# Position Monitor

## Role
Continuously check open positions against stop-loss, take-profit, and expiry thresholds. Trigger exits when conditions are met.

## Files to Read
- `~/.openclaw/memory/wwatcher_trading/portfolio.json` — Open positions with stop_loss, take_profit, entry details
- `~/.openclaw/memory/wwatcher_trading/config.json` — Exit rules (close_before_expiry_hours), monitor interval

## Parameters (from orchestrator)
- `monitor_run_id`: UUID for this monitoring pass
- `timestamp`: ISO string of when this check was triggered

## API Calls

### Get Current Market Price
For each open position, fetch the current market data:

```
GET https://api.elections.kalshi.com/trade-api/v2/markets/{ticker}
```

No authentication required.

**Response fields used:**
- `yes_price` — Current YES price
- `no_price` — Current NO price
- `status` — "open", "closed", "settled"
- `result` — Settlement result (if settled): "yes" or "no"

## Logic

### Step 1: Load Open Positions
Read `portfolio.json`. Get the `open_positions` array. If empty, write a single pipeline entry with `verdict: "no_positions"` and exit.

### Step 2: Check Each Position
For each open position, fetch current market data and run these checks:

#### Check A: Market Settled
If market `status` is "settled":
- If `result` matches position's `side` → WIN. Calculate PnL = `(1.00 - entry_price) * contracts`.
- If `result` does not match → LOSS. PnL = `-(entry_price * contracts)`.
- **Action**: Write exit entry with `trigger: "expiry"`.

#### Check B: Take-Profit Hit
```
if position.side == "YES" and current_yes_price >= position.take_profit:
  trigger exit — "take_profit"
if position.side == "NO" and current_no_price >= position.take_profit:
  trigger exit — "take_profit"
```

Calculate PnL: `(current_price - entry_price) * contracts`

#### Check C: Stop-Loss Hit
```
if position.side == "YES" and current_yes_price <= position.stop_loss:
  trigger exit — "stop_loss"
if position.side == "NO" and current_no_price <= position.stop_loss:
  trigger exit — "stop_loss"
```

Calculate expected PnL: `(current_price - entry_price) * contracts` (will be negative)

#### Check D: Expiry Approaching
```
hours_to_expiry = (position.expiry - now) / 3600000
if hours_to_expiry <= config.exit_rules.close_before_expiry_hours:
  if current unrealized PnL > 0:
    trigger exit — "pre_expiry_profit"
  else:
    flag for user decision — "expiry_approaching"
```

### Step 3: Update Position Metrics
For positions that don't trigger an exit, update their tracking data:
```
current_value = current_price * contracts
unrealized_pnl = current_value - (entry_price * contracts)
unrealized_pnl_pct = (unrealized_pnl / (entry_price * contracts)) * 100
hold_hours = (now - entry_time) / 3600000
```

## Output Schema

### Exit Trigger (written to pipeline for executor)
```json
{
  "id": "<uuid>",
  "ts": "<ISO timestamp>",
  "agent": "monitor",
  "market_id": "<kalshi_ticker>",
  "market_title": "<market_title>",
  "stage": "monitor_exit",
  "data": {
    "trade_id": "<original_trade_id>",
    "trigger": "take_profit|stop_loss|expiry|pre_expiry_profit",
    "side": "YES",
    "entry_price": 0.63,
    "current_price": 0.82,
    "contracts": 63,
    "expected_pnl": 11.97,
    "expected_pnl_pct": 30.2,
    "hold_hours": 42.7,
    "monitor_run_id": "<uuid>"
  },
  "verdict": "exit_triggered",
  "confidence": null,
  "reasoning": "Take-profit hit: YES price reached 82¢ (target: 82¢). Entry at 63¢, +19¢ gain per contract. Exiting 63 contracts for ~$11.97 profit."
}
```

### Status Report (no exits triggered)
```json
{
  "id": "<uuid>",
  "ts": "<ISO timestamp>",
  "agent": "monitor",
  "market_id": "ALL",
  "market_title": "Position Monitor Report",
  "stage": "monitor_check",
  "data": {
    "positions_checked": 2,
    "exits_triggered": 0,
    "position_summaries": [
      {
        "trade_id": "trade_004",
        "market_id": "KXETH-5K-MAR",
        "current_price": 0.52,
        "unrealized_pnl": 7.00,
        "unrealized_pnl_pct": 15.5,
        "hold_hours": 23.25,
        "distance_to_tp": 0.18,
        "distance_to_sl": 0.22
      }
    ],
    "monitor_run_id": "<uuid>"
  },
  "verdict": "all_clear",
  "confidence": null,
  "reasoning": "2 positions checked. No exits triggered. ETH-5K: +$7.00 (15.5%), 18¢ from TP. All positions within thresholds."
}
```

### Expiry Warning (needs user decision)
```json
{
  "id": "<uuid>",
  "ts": "<ISO timestamp>",
  "agent": "monitor",
  "market_id": "<kalshi_ticker>",
  "market_title": "<market_title>",
  "stage": "monitor_warning",
  "data": {
    "trade_id": "<trade_id>",
    "warning": "expiry_approaching",
    "hours_to_expiry": 3.5,
    "current_price": 0.58,
    "unrealized_pnl": -3.15,
    "unrealized_pnl_pct": -5.0
  },
  "verdict": "user_decision_needed",
  "confidence": null,
  "reasoning": "Position expiring in 3.5 hours. Currently at -$3.15 (-5%). Close now to limit loss, or hold through expiry?"
}
```

## Success/Failure Handling

**Exits triggered**: Write pipeline entries. Orchestrator spawns executor for each exit.
**All clear**: Write status report. Orchestrator continues normal loop.
**User decision needed**: Write warning. Orchestrator notifies user and waits for instruction.
**Market data unavailable**: Skip that position, report error. Do not trigger false exits.
**API rate limit**: Report to orchestrator. Suggest increasing monitor interval.
