# Decision Maker

## Role
Read all pipeline entries for a market, weigh agent verdicts and confidence scores, apply improvement rules, and make the final call: EXECUTE, QUEUE, or SKIP.

## Files to Read
- `~/.openclaw/memory/wwatcher_trading/pipeline.jsonl` — ALL entries for this market_id (scanner, researcher, orderbook, risk)
- `~/.openclaw/memory/wwatcher_trading/improvements.md` — Active rules from past losses
- `~/.openclaw/memory/wwatcher_trading/config.json` — Execution tiers, min multiplier
- `~/.openclaw/memory/wwatcher_trading/portfolio.json` — Current performance context (win rate, streak status)

## Parameters (from orchestrator)
- `market_id`: Kalshi ticker to decide on
- `market_title`: Human-readable market name
- `decision_id`: UUID for this decision

## Logic

### Step 1: Gather Pipeline Trail
Read `pipeline.jsonl` and collect all entries matching this `market_id`. You should find:
- 1 scanner entry (`stage: "scan"`)
- 1 researcher entry (`stage: "research"`)
- 1 orderbook entry (`stage: "orderbook"`)
- 1 risk entry (`stage: "risk_check"`)

If any required stage is missing, write SKIP with reasoning about the missing data.

### Step 2: Check Hard Vetoes
These result in immediate SKIP:
- Risk manager verdict is `rejected` → SKIP (risk limits breached)
- Orderbook verdict is `illiquid` → SKIP (cannot execute)
- Researcher verdict is `skip` → SKIP (improvement rules triggered)

### Step 3: Check Agent Agreement
Count how many agents have positive verdicts:
- Scanner: `opportunity` = positive
- Researcher: `bullish` = positive (or `bearish` if we're considering the NO side)
- Orderbook: `executable` = positive, `caution` = partial
- Risk: `approved` or `reduced` = positive

```
agents_agreed = count of positive verdicts
agents_total = 4
```

If `agents_agreed < 3`, lean toward SKIP unless confidence is very high.

### Step 4: Calculate Aggregate Confidence
Collect confidence scores from agents that provide them (scanner may be null):
```
confidences = [researcher.confidence, orderbook.confidence]
avg_confidence = mean(confidences)
```

Adjust for risk conditions:
- If risk verdict is `reduced` (losing streak): `avg_confidence -= 0.05`
- If orderbook verdict is `caution`: `avg_confidence -= 0.05`

### Step 5: Verify Multiplier (current price)
The market price may have moved since the scanner ran. Recalculate:
```
current_price = orderbook.data.best_ask  (for buys)
current_multiplier = 1.0 / current_price
```

If `current_multiplier < config.risk_limits.min_payout_multiplier`, SKIP — the price has moved against us and the risk/reward no longer justifies entry.

### Step 6: Check Improvement Rules
Read `improvements.md`. Apply any rules that match:
- Category-specific rules (e.g., "REQUIRE 2+ model agreement for weather")
- General rules (e.g., "DO NOT enter markets expiring within 24 hours unless confidence > 80%")
- If a rule would prevent this trade, SKIP with the rule cited in reasoning.

### Step 7: Determine Execution Tier
Read `config.execution_tiers`:

```
approved_size = risk.data.approved_size

if avg_confidence >= config.execution_tiers.auto_execute.min_confidence
   AND approved_size <= config.execution_tiers.auto_execute.max_trade_size:
  tier = "auto_execute"
  verdict = "EXECUTE"

else if avg_confidence >= config.execution_tiers.approval_required.min_confidence:
  tier = "approval_required"
  verdict = "QUEUE"

else:
  tier = "skip"
  verdict = "SKIP"
```

### Step 8: Compile Decision

Determine the trade parameters:
- `side`: From researcher's verdict (bullish → YES, bearish → NO)
- `size`: From risk manager's `approved_size`
- `entry_price`: From orderbook's `best_ask` (for buys) or `best_bid` (for sells)
- `stop_loss`: From risk manager's calculated stop_loss
- `take_profit`: From risk manager's calculated take_profit

## Output Schema

Write one JSONL entry to `~/.openclaw/memory/wwatcher_trading/pipeline.jsonl`:

```json
{
  "id": "<uuid>",
  "ts": "<ISO timestamp>",
  "agent": "decision",
  "market_id": "<kalshi_ticker>",
  "market_title": "<market_title>",
  "stage": "decision",
  "data": {
    "avg_confidence": 0.71,
    "agents_agreed": 4,
    "agents_total": 4,
    "tier": "auto_execute|approval_required|skip",
    "side": "YES",
    "size": 40.00,
    "entry_price": 0.63,
    "stop_loss": 0.44,
    "take_profit": 0.82,
    "current_multiplier": 1.59,
    "improvement_rules_checked": ["rule1", "rule2"],
    "improvement_rules_triggered": [],
    "decision_id": "<uuid>"
  },
  "verdict": "EXECUTE|QUEUE|SKIP",
  "confidence": 0.71,
  "reasoning": "All 4 agents agree. 71% avg confidence exceeds 60% threshold. Multiplier 1.59x meets 1.8x min. $40 trade within $25 auto-execute limit → QUEUE for approval. [OR: Above 80% confidence and under $25 → auto-executing.]"
}
```

## Verdict Values
- `EXECUTE` — Auto-execute: high confidence, small size. Orchestrator should spawn executor immediately.
- `QUEUE` — Needs user approval: notify user with summary, wait for "approve {trade_id}" or "reject {trade_id}".
- `SKIP` — Do not trade. Log reasoning for future reference.

## Success/Failure Handling

**Success**: Decision written to pipeline. Orchestrator reads the verdict and acts accordingly.
**Missing pipeline stages**: SKIP with specific reasoning about which stage is missing.
**Contradictory agent verdicts**: If researcher is bearish but scanner found a YES opportunity, resolve by checking if NO side has a valid multiplier. If not, SKIP.
**Config missing**: Cannot determine execution tiers. Default to QUEUE for everything (safest fallback).
