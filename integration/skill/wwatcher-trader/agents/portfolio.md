# Portfolio Tracker

## Role
Recalculate portfolio statistics from the trade log after every trade event. Update the portfolio state file with current balances, positions, win rate, and P&L.

## Files to Read
- `~/.openclaw/memory/wwatcher_trading/trades.jsonl` — Complete trade history (entries and exits)
- `~/.openclaw/memory/wwatcher_trading/portfolio.json` — Current portfolio state (to get initial_deposit and carry forward stable values)
- `~/.openclaw/memory/wwatcher_trading/config.json` — Risk limits for max_positions, losing streak threshold

## Files to Write
- `~/.openclaw/memory/wwatcher_trading/portfolio.json` — Overwrite with recalculated state
- `~/.openclaw/memory/wwatcher_trading/pipeline.jsonl` — Write warning if losing streak detected

## Parameters (from orchestrator)
- `trigger`: What caused this update — "trade_entry", "trade_exit", "manual_refresh"
- `tracker_id`: UUID for this update run

## Logic

### Step 1: Read All Trades
Parse `trades.jsonl`. Each line is a trade record. Separate into:
- **Entry trades**: `action == "BUY"`
- **Exit trades**: `action == "SELL" or action == "EXPIRED"`

Match entries to exits by base `trade_id` (exits use `trade_id + "_exit"` or match `market_id`).

### Step 2: Calculate Open Positions
Open positions are entries without a matching exit:
```
open_positions = entries.filter(e => !exits.find(x => x.market_id == e.market_id && x.ts > e.ts))
```

For each open position, carry forward entry data:
- `trade_id`, `market_id`, `market_title`, `side`
- `entry_price`, `contracts`, `cost` (= entry_price * contracts)
- `entry_time`, `expiry`
- `stop_loss`, `take_profit`
- `confidence_at_entry`

Set `current_price`, `current_value`, `unrealized_pnl` to last known values (monitor will update these).

### Step 3: Calculate Closed Trade Stats
For matched entry-exit pairs:
```
pnl = exit.pnl  (already calculated by executor)
pnl_pct = exit.pnl_pct
```

Aggregate:
- `total_trades`: Count of closed trades
- `wins`: Count where `pnl > 0`
- `losses`: Count where `pnl <= 0`
- `win_rate`: `wins / total_trades` (0 if no trades)
- `total_pnl`: Sum of all PnL
- `total_pnl_pct`: `total_pnl / initial_deposit * 100`
- `best_trade`: Highest PnL trade `{ market, pnl, pnl_pct }`
- `worst_trade`: Lowest PnL trade `{ market, pnl, pnl_pct }`
- `avg_confidence_at_entry`: Mean of all entry confidences
- `avg_hold_hours`: Mean of all hold durations

### Step 4: Calculate Balances
```
total_cost_in_positions = sum(open_position.cost for each open position)
total_realized_pnl = sum of all closed trade PnLs
initial_deposit = portfolio.balance.initial_deposit  (from existing file, set during setup)

available = initial_deposit + total_realized_pnl - total_cost_in_positions
in_positions = total_cost_in_positions
total = available + in_positions
```

### Step 5: Calculate Today's Stats
```
today = current date (YYYY-MM-DD)
today_trades = trades where ts matches today
today_entries = today_trades.filter(action == "BUY")
today_exits = today_trades.filter(action == "SELL" or action == "EXPIRED")

today.spent = sum(today_entries.size)
today.trades_executed = today_entries.length
today.pnl = sum(today_exits.pnl)
today.limit = config.risk_limits.daily_limit
```

### Step 6: Detect Losing Streak
Look at the most recent N closed trades (where N = `config.risk_limits.losing_streak_threshold`):
```
recent_trades = closed_trades.sort_by(ts).slice(-threshold)
consecutive_losses = count trailing losses from the end
```

If `consecutive_losses >= config.risk_limits.losing_streak_threshold`:
- Write a warning entry to `pipeline.jsonl`

### Step 7: Write Portfolio State
Assemble the complete `portfolio.json` and overwrite the file.

## Output Schema

### portfolio.json
```json
{
  "last_updated": "<ISO timestamp>",
  "balance": {
    "available": 860.00,
    "in_positions": 140.00,
    "total": 1000.00,
    "initial_deposit": 1000.00
  },
  "today": {
    "date": "2026-02-26",
    "spent": 40.00,
    "limit": 200.00,
    "trades_executed": 1,
    "pnl": 4.73
  },
  "all_time": {
    "total_trades": 5,
    "wins": 3,
    "losses": 1,
    "pending": 1,
    "win_rate": 0.75,
    "total_pnl": 47.20,
    "total_pnl_pct": 4.72,
    "best_trade": { "market": "BTC above 95k", "pnl": 28.50, "pnl_pct": 57.0 },
    "worst_trade": { "market": "Snow in NYC", "pnl": -30.00, "pnl_pct": -100.0 },
    "avg_confidence_at_entry": 0.68,
    "avg_hold_hours": 36.4
  },
  "open_positions": [],
  "position_count": 0,
  "max_positions": 3
}
```

### Losing Streak Warning (pipeline entry)
```json
{
  "id": "<uuid>",
  "ts": "<ISO timestamp>",
  "agent": "portfolio",
  "market_id": "SYSTEM",
  "market_title": "Losing Streak Warning",
  "stage": "portfolio_warning",
  "data": {
    "consecutive_losses": 3,
    "threshold": 3,
    "recent_losses": [
      { "trade_id": "trade_005", "market": "Some market", "pnl": -15.00 },
      { "trade_id": "trade_006", "market": "Another market", "pnl": -20.00 },
      { "trade_id": "trade_007", "market": "Third market", "pnl": -10.00 }
    ],
    "action": "reduce_size_50pct",
    "tracker_id": "<uuid>"
  },
  "verdict": "losing_streak_active",
  "confidence": null,
  "reasoning": "3 consecutive losses detected (-$45 total). Activating size reduction: all new trades will be at 50% of normal size until a win breaks the streak."
}
```

## Success/Failure Handling

**Success**: Portfolio file updated. Orchestrator can read fresh state.
**Empty trades.jsonl**: Write zero-state portfolio (all zeros, no positions). This is normal for new accounts.
**Malformed trade entry**: Skip the malformed line, log a warning in reasoning. Process remaining trades.
**Initial deposit unknown**: If `initial_deposit` is 0 or null, set it to the Kalshi balance (orchestrator should query via executor's balance check).
