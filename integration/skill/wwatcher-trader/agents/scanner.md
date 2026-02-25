# Market Scanner

## Role
Scan Kalshi public markets for mispriced opportunities that pass category, volume, multiplier, and spread filters. Rank and write top candidates to the pipeline.

## Files to Read
- `~/.openclaw/memory/wwatcher_trading/config.json` — Categories, min volume, min multiplier, spread limit, scanner interval
- `~/.openclaw/memory/wwatcher_trading/portfolio.json` — Check available position slots (`position_count` < `max_positions`)
- `~/.openclaw/memory/wwatcher_trading/improvements.md` — Check for "AVOID" rules that should exclude specific market types
- `~/.openclaw/memory/wwatcher_trading/pipeline.jsonl` — Check for markets already in the pipeline (avoid duplicate scans)

## Parameters (from orchestrator)
- `scan_id`: UUID assigned by the orchestrator for this scan run
- `timestamp`: ISO string of when this scan was triggered

## API Calls

### List Open Markets
```
GET https://api.elections.kalshi.com/trade-api/v2/markets?status=open&limit=200
```

No authentication required for public market data.

**Response fields used:**
- `ticker` — Market ID (e.g., "KXBTC-100K-MAR")
- `title` — Human-readable name
- `category` — Market category tag
- `yes_price` / `no_price` — Current prices (cents, divide by 100)
- `volume_24h` — 24-hour volume
- `close_time` — Expiry timestamp
- `status` — Must be "open"

### Pagination
If `cursor` is present in response, fetch next page:
```
GET https://api.elections.kalshi.com/trade-api/v2/markets?status=open&limit=200&cursor={cursor}
```

Continue until no more cursor or enough candidates found.

## Logic

### Step 1: Check Position Availability
Read `portfolio.json`. If `position_count >= max_positions`, write a single pipeline entry:
```json
{"agent":"scanner","stage":"scan","verdict":"no_slots","reasoning":"All position slots filled"}
```
Then exit.

### Step 2: Fetch Markets
Call the Kalshi markets endpoint. Collect all open markets across pages.

### Step 3: Filter
For each market, apply these filters in order (skip on first failure):

1. **Category match**: Market category must be in `config.categories` array
2. **Volume floor**: `volume_24h >= config.scanner.min_market_volume_24h`
3. **Spread check**: `abs(yes_price - no_price) <= config.scanner.max_spread` (for meaningful bid/ask, if both sides have prices)
4. **Days to expiry**: `(close_time - now) / 86400000 >= config.scanner.min_days_to_expiry`
5. **Payout multiplier**: `1.0 / min(yes_price, 1 - no_price) >= config.risk_limits.min_payout_multiplier`
   - Check both YES and NO sides — pick the better multiplier
6. **Improvement rules**: Check `improvements.md` for any "AVOID" rules matching this market's category or characteristics
7. **Not already in pipeline**: Check `pipeline.jsonl` — skip markets that already have a recent scanner entry (within last hour)

### Step 4: Rank
Score remaining candidates by:
- **Volume weight** (40%): Normalized `volume_24h` relative to median volume
- **Multiplier weight** (30%): Higher multiplier = more reward = higher rank
- **Expiry proximity** (20%): Markets expiring in 2-7 days rank highest (soon enough for resolution, not too rushed)
- **Whale signals** (10%): If polymaster has recent alerts for this market's topic, boost rank

### Step 5: Select Top N
Take the top 3 candidates (or fewer if less than 3 pass filters). These become opportunities for the next pipeline stage.

## Output Schema

Write one JSONL entry per opportunity to `~/.openclaw/memory/wwatcher_trading/pipeline.jsonl`:

```json
{
  "id": "<uuid>",
  "ts": "<ISO timestamp>",
  "agent": "scanner",
  "market_id": "<kalshi_ticker>",
  "market_title": "<market_title>",
  "stage": "scan",
  "data": {
    "yes_price": 0.55,
    "no_price": 0.45,
    "volume_24h": 125000,
    "category": "crypto",
    "expiry": "2026-03-31T23:59:00Z",
    "days_to_expiry": 35,
    "payout_multiplier": 1.82,
    "best_side": "YES",
    "rank_score": 0.78,
    "scan_id": "<scan_uuid>"
  },
  "verdict": "opportunity",
  "confidence": null,
  "reasoning": "Price at 55 cents with strong volume ($125k/24h). Multiplier 1.82x on YES side. Expires in 35 days."
}
```

## Success/Failure Handling

**Success**: One or more opportunities written to pipeline. Report count to orchestrator.
**No opportunities**: Write a single entry with `verdict: "no_opportunities"` and reasoning explaining filters that eliminated candidates.
**API error**: Report the HTTP status and error message. Do not write to pipeline. The orchestrator should retry on next scan interval.
**Rate limited (429)**: Report to orchestrator. Suggest increasing `scanner.interval_minutes`.
