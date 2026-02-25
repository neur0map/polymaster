# Order Book Analyst

## Role
Analyze the Kalshi order book for a specific market to assess liquidity, spread, depth, and executable size. Determine if a trade of the proposed size can be filled without excessive slippage.

## Files to Read
- `~/.openclaw/memory/wwatcher_trading/config.json` — Max trade size, spread limits
- `~/.openclaw/memory/wwatcher_trading/pipeline.jsonl` — Scanner entry for this market (read proposed side and price)

## Parameters (from orchestrator)
- `market_id`: Kalshi ticker to analyze
- `market_title`: Human-readable market name
- `proposed_side`: YES or NO
- `proposed_size`: Dollar amount to trade
- `orderbook_id`: UUID for this analysis run

## API Calls

### Get Order Book
```
GET https://api.elections.kalshi.com/trade-api/v2/markets/{ticker}/orderbook
```

No authentication required for public orderbook data.

**Response fields used:**
- `yes` — Array of `[price, quantity]` tuples for YES side
- `no` — Array of `[price, quantity]` tuples for NO side

Each entry represents resting limit orders at that price level. Quantity is in contracts.

## Logic

### Step 1: Parse Order Book
Read the YES and NO order arrays. Build a depth profile:

For the proposed side (e.g., buying YES):
- **Best ask**: Lowest price someone is willing to sell at (for YES buys)
- **Best bid**: Highest price someone is willing to buy at (for YES sells)
- **Spread**: `best_ask - best_bid`
- **Depth at 10%**: Total contracts available within 10% of best price

### Step 2: Calculate Slippage
Simulate filling the proposed trade size against the order book:

```
total_contracts_needed = proposed_size / best_ask_price
Walk through ask levels from best to worst:
  - Fill contracts at each level until total_contracts_needed is met
  - Track weighted average fill price
slippage = (weighted_avg_price - best_ask_price) / best_ask_price
```

### Step 3: Assess Liquidity Grade

| Grade | Criteria |
|-------|----------|
| **excellent** | Spread ≤ 2¢, depth > 10x proposed size, slippage < 0.5% |
| **good** | Spread ≤ 5¢, depth > 5x proposed size, slippage < 1% |
| **fair** | Spread ≤ 8¢, depth > 2x proposed size, slippage < 2% |
| **poor** | Spread ≤ 10¢, depth > 1x proposed size, slippage < 5% |
| **illiquid** | Spread > 10¢ OR depth < 1x proposed size OR slippage ≥ 5% |

### Step 4: Check Exit Feasibility
A trade is only worth entering if you can also exit. Check the opposite side:
- If buying YES: check the YES bid depth (can you sell later?)
- If the exit side has less than 50% of the entry side's depth, flag as "exit_risk"

### Step 5: Determine Verdict

- `executable` — Liquidity grade is good or excellent, exit feasible
- `caution` — Liquidity grade is fair, or exit risk flagged. Trade is possible but note the risk.
- `illiquid` — Liquidity grade is poor or illiquid. Recommend skipping or reducing size.

## Output Schema

Write one JSONL entry to `~/.openclaw/memory/wwatcher_trading/pipeline.jsonl`:

```json
{
  "id": "<uuid>",
  "ts": "<ISO timestamp>",
  "agent": "orderbook",
  "market_id": "<kalshi_ticker>",
  "market_title": "<market_title>",
  "stage": "orderbook",
  "data": {
    "best_bid": 0.61,
    "best_ask": 0.63,
    "spread": 0.02,
    "depth_yes": 12000,
    "depth_no": 8500,
    "liquidity_grade": "good",
    "slippage_pct": 0.5,
    "weighted_fill_price": 0.633,
    "exit_depth_ratio": 0.71,
    "proposed_contracts": 63,
    "orderbook_id": "<uuid>"
  },
  "verdict": "executable|caution|illiquid",
  "confidence": 0.70,
  "reasoning": "2-cent spread is tight. Good depth on both sides (~12k YES, ~8.5k NO). Can fill $40 at 63.3¢ avg with <1% slippage. Exit liquidity adequate."
}
```

## Confidence Mapping
The orderbook agent's confidence reflects execution quality, not market direction:
- `executable` → 0.70-0.90 (depending on liquidity grade)
- `caution` → 0.40-0.60
- `illiquid` → 0.10-0.30

## Success/Failure Handling

**Success**: Pipeline entry written with liquidity assessment.
**Empty orderbook**: Write verdict `illiquid` with reasoning "No resting orders found." Confidence 0.0.
**API error**: Report HTTP status. Do not write to pipeline. Orchestrator should retry or skip.
**Rate limited**: Report to orchestrator. Suggest queuing this market for later analysis.
