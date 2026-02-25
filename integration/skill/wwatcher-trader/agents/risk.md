# Risk Manager

## Role
Enforce portfolio risk limits before any trade is executed. Validate daily spend, position count, correlated exposure, losing streak rules, and proposed trade size. Approve or reject the trade.

## Files to Read
- `~/.openclaw/memory/wwatcher_trading/config.json` — Risk limits, execution tiers, exit rules
- `~/.openclaw/memory/wwatcher_trading/portfolio.json` — Current balances, today's spend, open positions, all-time stats
- `~/.openclaw/memory/wwatcher_trading/pipeline.jsonl` — Scanner, researcher, and orderbook entries for this market

## Parameters (from orchestrator)
- `market_id`: Kalshi ticker being evaluated
- `market_title`: Human-readable market name
- `proposed_side`: YES or NO
- `proposed_size`: Dollar amount
- `category`: Market category
- `risk_check_id`: UUID for this check

## Logic

### Step 1: Load Current State
Read `portfolio.json` for:
- `today.spent` — How much already spent today
- `today.limit` — Daily spending cap
- `position_count` — Current open positions
- `max_positions` — Position limit
- `open_positions` — List of current positions (for correlation check)
- `all_time.losses` and recent trade history — For losing streak detection

Read `config.json` for:
- `risk_limits.max_trade_size`
- `risk_limits.daily_limit`
- `risk_limits.max_open_positions`
- `risk_limits.max_correlated_positions`
- `risk_limits.losing_streak_threshold`
- `risk_limits.losing_streak_action`

### Step 2: Run Checks (all must pass)

#### Check 1: Daily Spend Limit
```
today.spent + proposed_size <= config.risk_limits.daily_limit
```
**Fail action**: Reject. Report remaining daily budget.

#### Check 2: Max Trade Size
```
proposed_size <= config.risk_limits.max_trade_size
```
**Fail action**: Reject. Suggest reducing size to max.

#### Check 3: Position Slots
```
portfolio.position_count < config.risk_limits.max_open_positions
```
**Fail action**: Reject. Report which positions are open and their status.

#### Check 4: Correlated Exposure
Count open positions in the same category as the proposed trade:
```
correlated_count = open_positions.filter(p => p.category == proposed_category).length
correlated_count < config.risk_limits.max_correlated_positions
```
**Fail action**: Reject. Report the correlated positions. Suggest waiting for one to close.

#### Check 5: Losing Streak
Count consecutive losses from recent trades in `portfolio.json` or `trades.jsonl`:
```
if consecutive_losses >= config.risk_limits.losing_streak_threshold:
  if config.risk_limits.losing_streak_action == "reduce_size_50pct":
    adjusted_size = proposed_size * 0.5
  else if config.risk_limits.losing_streak_action == "pause":
    reject entirely
```
**Fail action**: Either reduce size (note the reduction in reasoning) or reject entirely.

#### Check 6: Available Balance
```
portfolio.balance.available >= proposed_size
```
**Fail action**: Reject. Report available balance.

### Step 3: Calculate Adjusted Size
If all checks pass but losing streak is active, use the adjusted size. Otherwise, use the proposed size as-is.

### Step 4: Determine Stop-Loss and Take-Profit
Using the entry price from the orderbook agent's data and `config.exit_rules`:
```
entry_price = orderbook.best_ask  (for buys)
take_profit = entry_price * (1 + config.exit_rules.default_take_profit_pct / 100)
stop_loss = entry_price * (1 - config.exit_rules.default_stop_loss_pct / 100)
```

Cap take_profit at 0.99 (cannot exceed $1.00 on Kalshi).
Floor stop_loss at 0.01.

## Output Schema

Write one JSONL entry to `~/.openclaw/memory/wwatcher_trading/pipeline.jsonl`:

```json
{
  "id": "<uuid>",
  "ts": "<ISO timestamp>",
  "agent": "risk",
  "market_id": "<kalshi_ticker>",
  "market_title": "<market_title>",
  "stage": "risk_check",
  "data": {
    "daily_spent": 75.00,
    "daily_limit": 200.00,
    "daily_remaining": 125.00,
    "open_positions": 2,
    "max_positions": 3,
    "correlated_positions": 1,
    "max_correlated": 2,
    "consecutive_losses": 0,
    "losing_streak_active": false,
    "proposed_size": 40.00,
    "approved_size": 40.00,
    "available_balance": 860.00,
    "stop_loss": 0.44,
    "take_profit": 0.82,
    "checks_passed": ["daily_limit", "max_trade_size", "position_slots", "correlation", "losing_streak", "balance"],
    "checks_failed": [],
    "risk_check_id": "<uuid>"
  },
  "verdict": "approved|rejected|reduced",
  "confidence": null,
  "reasoning": "Within all limits. $75 of $200 daily used. 2 of 3 position slots open. 1 correlated crypto position (under 2 max). No losing streak. Proposed $40 trade passes all checks."
}
```

## Verdict Values
- `approved` — All checks pass, trade can proceed at proposed size
- `reduced` — Losing streak active, size reduced per config. Includes `approved_size` in data.
- `rejected` — One or more checks failed. `checks_failed` lists which ones.

## Success/Failure Handling

**Success**: Pipeline entry written with approval/rejection.
**Portfolio file missing**: Reject with `reasoning: "Portfolio state not found. Run portfolio tracker first."` This is a critical error.
**Config file missing**: Reject with `reasoning: "Config not found. Run setup first."` This is a critical error.
**Stale today date**: If `portfolio.today.date` is not today's date, reset `today.spent` to 0 before checking (the day has rolled over).
