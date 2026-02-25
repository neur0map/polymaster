# Autonomous Trading via OpenClaw — Design Document

**Date**: 2026-02-24
**Status**: Design complete, pending implementation
**Scope**: Empower OpenClaw as a fully autonomous trading agent for Kalshi prediction markets using polymaster as a data source, multi-agent orchestration, shared memory files, and a self-improving feedback loop.

---

## Problem Statement

Polymaster currently detects whale trades, enriches them with market context, scores them, and delivers research signals. But the pipeline stops at signal delivery — no trade execution, no position management, no portfolio tracking.

The goal is NOT to make polymaster into a trading bot. Instead, empower **OpenClaw** as an autonomous 24/7 trading agent that:

1. Scans Kalshi markets for mispriced opportunities
2. Researches and validates with multiple specialized sub-agents
3. Executes trades within user-defined risk guardrails
4. Monitors open positions for exits
5. Learns from losses and auto-improves agent behavior

No machine learning. Pure rule-based guardrails + LLM reasoning + structured self-improvement.

---

## Architecture Overview

The system is an **OpenClaw skill ecosystem**, not a polymaster feature. OpenClaw is the brain; polymaster and Kalshi APIs are tools.

```
OpenClaw (24/7 runtime)
├── Skill: wwatcher-trader/
│   ├── SKILL.md              ← Master skill: orchestrates all agents
│   ├── agents/
│   │   ├── scanner.md        ← Market Scanner agent prompt
│   │   ├── researcher.md     ← Deep Research agent prompt
│   │   ├── orderbook.md      ← Order Book Analyst agent prompt
│   │   ├── risk.md           ← Risk Manager agent prompt
│   │   ├── decision.md       ← Decision Maker agent prompt
│   │   ├── executor.md       ← Trade Executor agent prompt
│   │   ├── monitor.md        ← Position Monitor agent prompt
│   │   ├── portfolio.md      ← Portfolio Tracker agent prompt
│   │   └── postmortem.md     ← Post-Mortem Reporter agent prompt
│   ├── contexts/
│   │   ├── crypto.md         ← Crypto-specific research guidance
│   │   ├── weather.md        ← Weather-specific research guidance
│   │   ├── politics.md       ← Politics-specific research guidance
│   │   └── sports.md         ← Sports-specific research guidance
│   └── config/
│       └── setup.md          ← Guided setup conversation instructions
│
├── Memory: ~/.openclaw/memory/wwatcher_trading/
│   ├── config.json           ← User trading config (limits, keys, thresholds)
│   ├── pipeline.jsonl        ← Agent findings (the shared brain)
│   ├── trades.jsonl          ← Executed trades log
│   ├── portfolio.json        ← Current state: positions, P&L, balances
│   ├── lessons.jsonl         ← Post-mortem learnings from losses
│   └── improvements.md       ← Active rules derived from lessons
```

Each agent is a markdown prompt file. OpenClaw spawns sub-agents via `sessions_spawn`, each agent does its job, writes structured output to the shared JSONL, and exits. The decision maker reads everything and acts.

---

## Shared Memory File — `pipeline.jsonl`

The central nervous system. Every agent writes here. The decision maker reads here. Each line follows this schema:

```jsonl
{"id":"uuid","ts":"2026-02-24T14:30:00Z","agent":"scanner","market_id":"KXBTC-100K-MAR","market_title":"Bitcoin above 100k by March 31","stage":"scan","data":{"yes_price":0.62,"volume_24h":450000,"category":"crypto","expiry":"2026-03-31","payout_multiplier":1.61},"verdict":"opportunity","confidence":0.65,"reasoning":"Price at 62 cents with strong volume. BTC currently at $97k with bullish momentum."}

{"id":"uuid","ts":"2026-02-24T14:30:45Z","agent":"researcher","market_id":"KXBTC-100K-MAR","stage":"research","data":{"perplexity_results":["..."],"polymaster_signal":{"tier":"high","factors":["Top 10 trader buying YES"]},"news_sentiment":"bullish","catalysts":["ETF inflows up 40%","halving cycle momentum"]},"verdict":"bullish","confidence":0.72,"reasoning":"Multiple whale entries, strong ETF flows, no major bearish catalysts found."}

{"id":"uuid","ts":"2026-02-24T14:31:10Z","agent":"orderbook","market_id":"KXBTC-100K-MAR","stage":"orderbook","data":{"best_bid":0.61,"best_ask":0.63,"spread":0.02,"depth_yes":12000,"depth_no":8500,"liquidity":"good"},"verdict":"executable","confidence":0.70,"reasoning":"2-cent spread is tight. Good depth on both sides. Can fill $50 without slippage."}

{"id":"uuid","ts":"2026-02-24T14:31:30Z","agent":"risk","market_id":"KXBTC-100K-MAR","stage":"risk_check","data":{"daily_spent":75,"daily_limit":200,"open_positions":2,"max_positions":3,"proposed_size":40},"verdict":"approved","confidence":null,"reasoning":"Within all limits. $75 of $200 daily used. 2 of 3 position slots open. Proposed $40 trade passes."}

{"id":"uuid","ts":"2026-02-24T14:31:50Z","agent":"decision","market_id":"KXBTC-100K-MAR","stage":"decision","data":{"avg_confidence":0.69,"tier":"approval_required","agents_agreed":4,"agents_total":4},"verdict":"QUEUE","confidence":0.69,"reasoning":"All 4 agents agree. 69% confidence above 60% threshold. $40 trade exceeds $25 auto-execute limit. Routing to approval queue."}
```

### Field Reference

| Field | Type | Purpose |
|-------|------|---------|
| `id` | string | UUID, unique per entry |
| `ts` | string | ISO timestamp |
| `agent` | string | Which agent wrote this (scanner, researcher, orderbook, risk, decision, executor, monitor) |
| `market_id` | string | Kalshi ticker — ties all entries for one opportunity together |
| `market_title` | string | Human-readable market name |
| `stage` | string | Pipeline stage for chronological ordering |
| `data` | object | Agent-specific structured data |
| `verdict` | string | One-word machine-parseable outcome |
| `confidence` | float/null | Normalized 0-1, null when not applicable (risk checks) |
| `reasoning` | string | Natural language — what the decision maker actually reads |

---

## Trade Log — `trades.jsonl`

Every executed trade (entry and exit) is logged here. The portfolio tracker and post-mortem agent read this file.

```jsonl
{"id":"trade_001","ts":"2026-02-24T14:32:00Z","market_id":"KXBTC-100K-MAR","market_title":"Bitcoin above 100k by March 31","side":"YES","action":"BUY","price":0.63,"size":40,"contracts":63,"status":"filled","order_id":"kalshi_abc123","confidence_at_entry":0.69,"agents_agreed":4,"reasoning_summary":"Whale signals + ETF inflows + good liquidity"}

{"id":"trade_002","ts":"2026-02-26T09:15:00Z","market_id":"KXBTC-100K-MAR","market_title":"Bitcoin above 100k by March 31","side":"YES","action":"SELL","price":0.71,"size":44.73,"contracts":63,"status":"filled","order_id":"kalshi_def456","trigger":"take_profit","pnl":4.73,"pnl_pct":11.8,"hold_hours":42.7,"reasoning_summary":"Price hit 71c, +8c gain. Monitor agent triggered take-profit."}

{"id":"trade_003","ts":"2026-02-25T18:00:00Z","market_id":"KXSNOW-NYC-FEB","market_title":"Snow in NYC by Feb 28","side":"YES","action":"BUY","price":0.55,"size":30,"contracts":54,"status":"filled","order_id":"kalshi_ghi789","confidence_at_entry":0.64,"agents_agreed":3,"reasoning_summary":"Weather models showing 70% precip chance, contrarian opportunity"}

{"id":"trade_003_exit","ts":"2026-02-28T23:59:00Z","market_id":"KXSNOW-NYC-FEB","market_title":"Snow in NYC by Feb 28","side":"YES","action":"EXPIRED","price":0.00,"size":0,"contracts":54,"status":"loss","trigger":"expiry","pnl":-30,"pnl_pct":-100,"hold_hours":78,"reasoning_summary":"No snow fell. Market resolved NO. Full loss."}
```

### Field Reference

| Field | Type | Purpose |
|-------|------|---------|
| `id` | string | Trade ID (entry and exit share the base ID) |
| `trigger` | string | Why the exit happened: `take_profit`, `stop_loss`, `expiry`, `manual`, `monitor_exit` |
| `pnl` | float | Realized profit/loss in USD |
| `pnl_pct` | float | Percentage return |
| `confidence_at_entry` | float | What the system believed when entering — post-mortem compares to reality |
| `hold_hours` | float | Time in position — useful for pattern detection |
| `reasoning_summary` | string | One-line trace of the decision agent's logic |

---

## Portfolio State — `portfolio.json`

Single JSON file, overwritten after every trade event. Real-time snapshot any agent can read.

```json
{
  "last_updated": "2026-02-26T09:15:00Z",
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
  "open_positions": [
    {
      "trade_id": "trade_004",
      "market_id": "KXETH-5K-MAR",
      "market_title": "Ethereum above 5000 by March 31",
      "side": "YES",
      "entry_price": 0.45,
      "current_price": 0.52,
      "contracts": 100,
      "cost": 45.00,
      "current_value": 52.00,
      "unrealized_pnl": 7.00,
      "unrealized_pnl_pct": 15.5,
      "entry_time": "2026-02-25T10:00:00Z",
      "hold_hours": 23.25,
      "expiry": "2026-03-31T23:59:00Z",
      "stop_loss": 0.30,
      "take_profit": 0.70,
      "confidence_at_entry": 0.66
    }
  ],
  "position_count": 1,
  "max_positions": 3
}
```

Key design:
- **`today` block resets daily** — Risk manager reads this to enforce daily limits
- **`open_positions` array** — Monitor agent loops through checking current prices
- **`stop_loss` and `take_profit`** — Set at entry, monitor agent enforces
- **`all_time` stats** — Decision maker can factor in streaks and overall performance
- **Single file, overwritten** — No append conflicts, always current state

---

## Self-Learning Loop — Lessons & Improvements

Two files work together. `lessons.jsonl` is raw post-mortem data. `improvements.md` is distilled rules every agent reads before acting.

### `lessons.jsonl`

Written by the post-mortem agent after every loss:

```jsonl
{"id":"lesson_001","ts":"2026-02-28T23:59:00Z","trade_id":"trade_003","market_id":"KXSNOW-NYC-FEB","market_title":"Snow in NYC by Feb 28","loss":-30.00,"entry_confidence":0.64,"original_reasoning":"Weather models showing 70% precip chance, contrarian opportunity","post_mortem_research":"Checked 3 weather sources post-event. Original research relied on a single GFS model run. ERA5 and ECMWF models showed only 35% precip. NWS forecast was rain not snow. Temperature was 2F above freezing threshold.","root_cause":"single_source_bias","missed_factors":["Multiple model disagreement","Temperature margin too thin","Rain vs snow distinction ignored"],"improvement_rule":"For weather markets: require 2+ model agreement and check temperature margin above/below threshold. Never rely on a single forecast model.","severity":"high","category":"weather"}
```

### `improvements.md`

The living rulebook. Auto-updated when new lessons arrive. Every agent loads this as context:

```markdown
# Trading Improvements — Active Rules

> These rules are learned from past losses. Read before every decision.
> Last updated: 2026-02-28T23:59:00Z | Total lessons: 3

## Weather Markets
- REQUIRE 2+ forecast model agreement before entering. Single model = skip.
- CHECK temperature margin: need 3F+ buffer above/below threshold.
- DISTINGUISH rain vs snow vs ice — precipitation type matters, not just probability.
- Source: lesson_001 (Snow in NYC, -$30)

## Crypto Markets
- AVOID entries within 4 hours of FOMC announcements or major macro events.
- CHECK funding rates — extreme positive funding often precedes pullbacks.
- Source: lesson_002 (BTC above 100k, -$25)

## General Rules
- DO NOT enter markets expiring within 24 hours unless confidence > 80%.
- REDUCE position size by 50% during 3+ trade losing streaks.
- Source: lesson_003 (rushed expiry trade, -$15)
```

Key design:
- **Imperative verbs** (REQUIRE, CHECK, AVOID, DO NOT) — LLMs follow direct instructions better than observations
- **Grouped by category** — Agents only scan their relevant section
- **Source linked** — Decision maker can trace why a rule exists
- **Auto-updated** — Post-mortem writes to `lessons.jsonl`, then regenerates `improvements.md` from all accumulated lessons

---

## Agent Roster — 9 Agents

### Scanner (`scanner.md`)
- Runs on a loop (every 5-15 minutes, configurable)
- Calls Kalshi public API to list open markets
- Filters by user categories, minimum volume, minimum payout multiplier
- Reads `improvements.md` for "AVOID" rules
- Reads `portfolio.json` for available position slots
- Ranks opportunities by priority: volume + momentum + whale signals + expiry proximity
- Writes top opportunities to `pipeline.jsonl`

### Researcher (`researcher.md`)
- Triggered per opportunity found by scanner
- Loads category-specific context from `contexts/{category}.md`
- Uses polymaster Perplexity integration for web research
- Reads `improvements.md` for category-specific research requirements
- Reads `lessons.jsonl` for past trades on similar markets
- Writes findings + confidence to `pipeline.jsonl`

### Order Book Analyst (`orderbook.md`)
- Checks Kalshi orderbook for the specific market
- Evaluates spread, depth, slippage for proposed trade size
- Flags thin markets where entry/exit would be costly
- Writes liquidity assessment to `pipeline.jsonl`

### Risk Manager (`risk.md`)
- Reads `portfolio.json` for current limits and exposure
- Validates: daily spend, max positions, max single trade, correlated exposure
- Checks losing streak status — applies size reduction if active
- Writes approval/rejection to `pipeline.jsonl`

### Decision Maker (`decision.md`)
- Reads ALL `pipeline.jsonl` entries for the `market_id`
- Reads `improvements.md` for general rules
- Weighs all agent verdicts and confidence scores
- Checks: avg confidence >= 60%, multiplier >= 1.8x, all agents agree
- Determines execution tier: auto-execute, approval queue, or skip
- Writes final EXECUTE, QUEUE, or SKIP to `pipeline.jsonl`

### Executor (`executor.md`)
- Only runs when decision maker writes EXECUTE (or user approves a queued trade)
- Reads `config.json` for Kalshi API credentials
- Places the order via Kalshi Trade API
- Sets stop-loss and take-profit levels
- Writes trade result to `trades.jsonl`
- Triggers portfolio tracker update

### Position Monitor (`monitor.md`)
- Runs on an independent loop (every 2-5 minutes, configurable)
- Reads `portfolio.json` for open positions
- Checks current prices against stop-loss and take-profit thresholds
- Checks for expiry approaching (close before settlement if profitable)
- When triggering exit: writes to `pipeline.jsonl`, then hands to executor

### Portfolio Tracker (`portfolio.md`)
- Triggers after every trade event (entry, exit, expiry)
- Reads `trades.jsonl` to recalculate all-time stats
- Updates `portfolio.json` with current balances, win rate, P&L
- If losing streak >= 3: writes warning entry to `pipeline.jsonl`

### Post-Mortem Reporter (`postmortem.md`)
- Triggers when a trade resolves as a loss
- Reads the full `pipeline.jsonl` trail for the losing trade
- Runs fresh Perplexity research on what actually happened
- Compares original reasoning vs reality
- Identifies root cause and missed factors
- Writes lesson to `lessons.jsonl`
- Regenerates `improvements.md` from all accumulated lessons

---

## Category Context Injection

Agents are generalists with specialist context. When working on a crypto market, the researcher spawns with `researcher.md` + `contexts/crypto.md` + `improvements.md` loaded as context.

```
skill/
├── agents/         ← 9 generic agent prompts
├── contexts/       ← category-specific knowledge
│   ├── crypto.md   ← "check funding rates, on-chain flows, macro calendar..."
│   ├── weather.md  ← "require 2+ models, check temp margins..."
│   ├── politics.md ← "weight polling averages, check prediction market history..."
│   └── sports.md   ← "check injury reports, line movement, sharp money..."
```

New categories are added by creating a new `contexts/{category}.md` file and adding the category to `config.json`. No agent changes needed.

---

## Payout Multiplier — Risk/Reward Filter

A critical filter that prevents the system from entering low-reward trades. On Kalshi, every contract resolves to $1.00 or $0.00:

```
payout_multiplier = 1.00 / entry_price

Examples:
  YES at $0.55 → 1.00 / 0.55 = 1.82x ✓ (risk $0.55 to gain $0.45)
  YES at $0.40 → 1.00 / 0.40 = 2.50x ✓ (risk $0.40 to gain $0.60)
  YES at $0.70 → 1.00 / 0.70 = 1.43x ✗ skip
  YES at $0.80 → 1.00 / 0.80 = 1.25x ✗ skip
  NO  at $0.70 → 1.00 / 0.30 = 3.33x ✓ (buying NO at $0.30)
```

Default minimum: **1.8x**. User-configurable.

The target sweet spot:

```
                    SKIP              TARGET ZONE           SKIP
                 (low confidence)     (high value)     (bad risk/reward)
Confidence: |---[< 60%]-------[60%----70%----80%]---[> implied by price]---|
Price:      |---[< 0.20]------[0.20---0.40--0.55]---[> 0.55]-------------|
Multiplier: |---[5.0x+]-------[2.5x---1.8x-------]---[< 1.8x]-----------|
```

The system hunts for **mispriced markets** — where research confidence is higher than what the market price implies. That's where the edge is.

---

## Configuration — `config.json`

Set up through a guided conversation when the user first installs the skill. All values changeable later with natural language.

```json
{
  "version": 1,
  "setup_complete": true,
  "last_updated": "2026-02-24T10:00:00Z",

  "kalshi": {
    "api_key_id": "encrypted:xxxx",
    "private_key": "encrypted:xxxx",
    "environment": "production",
    "verified": true
  },

  "categories": ["crypto", "politics", "weather"],

  "risk_limits": {
    "max_trade_size": 50,
    "daily_limit": 200,
    "max_open_positions": 3,
    "max_correlated_positions": 2,
    "losing_streak_threshold": 3,
    "losing_streak_action": "reduce_size_50pct",
    "min_payout_multiplier": 1.8
  },

  "execution_tiers": {
    "auto_execute": {
      "min_confidence": 0.80,
      "max_trade_size": 25,
      "description": "Trades above 80% confidence under $25 execute immediately"
    },
    "approval_required": {
      "min_confidence": 0.60,
      "description": "Trades 60-79% confidence or above $25 go to approval queue"
    },
    "skip": {
      "below_confidence": 0.60,
      "description": "Below 60% confidence — never trade"
    }
  },

  "exit_rules": {
    "default_take_profit_pct": 30,
    "default_stop_loss_pct": 50,
    "close_before_expiry_hours": 4,
    "trailing_stop": false
  },

  "scanner": {
    "interval_minutes": 10,
    "min_market_volume_24h": 10000,
    "min_days_to_expiry": 1,
    "max_spread": 0.10
  },

  "monitor": {
    "interval_minutes": 3
  },

  "notifications": {
    "on_trade_executed": true,
    "on_approval_needed": true,
    "on_stop_loss_hit": true,
    "on_daily_summary": true
  }
}
```

### Guided Setup Flow

```
User: Install wwatcher-trader skill
OpenClaw: Let's set up autonomous trading. First — do you have a Kalshi account?
User: No
OpenClaw: Go to kalshi.com, create an account, then go to Profile > API Keys
         and generate a key pair. Paste your API Key ID when ready.
User: KXKEY-abc123
OpenClaw: Got it. Now paste your private key (I'll store it encrypted).
User: pk_xxxxx
OpenClaw: Verified — your Kalshi API is working.
         What markets interest you? Pick from: crypto, politics, weather, sports, economics
User: crypto and politics
OpenClaw: Good. Now let's set risk limits. I recommend starting conservative:
         - Max $50 per trade
         - Max $200 per day
         - Max 3 open positions
         Want to use these defaults or customize?
User: Use defaults
OpenClaw: Auto-execution tiers. I recommend:
         - Auto-execute: 80%+ confidence, under $25
         - Need your approval: 60-79% confidence or over $25
         - Skip: below 60%
         Sound good?
User: Yes
OpenClaw: Minimum payout multiplier — I recommend 1.8x.
         This means only trade when you can nearly double your money.
         A YES at $0.55 = 1.82x ✓ | A YES at $0.70 = 1.43x ✗ skip.
         Use 1.8x default or set your own?
User: Use default
OpenClaw: All set. Starting market scanner now. I'll notify you when I find opportunities.
```

### Natural Language Config Updates

- "raise my daily limit to $500" → updates `risk_limits.daily_limit`
- "add sports to my categories" → appends to `categories`
- "set min multiplier to 2.0" → updates `risk_limits.min_payout_multiplier`
- "only auto-execute above 90% confidence" → updates `execution_tiers.auto_execute.min_confidence`
- "pause trading" → stops scanner loop, keeps monitor running
- "resume trading" → restarts scanner loop
- "show my config" → reads config.json, summarizes
- "kill all positions" → emergency exit, sells everything at market

---

## Complete Pipeline Flow

```
Every 10 minutes:
┌──────────────────────────────────────────────────────────────┐
│  SCANNER                                                      │
│  1. Read config.json → categories, min_volume, min_multiplier │
│  2. Read portfolio.json → any position slots open?            │
│  3. Read improvements.md → any "AVOID" rules?                 │
│  4. Call Kalshi API → list open markets                       │
│  5. Filter: category, volume, multiplier >= 1.8x, spread     │
│  6. Rank by: volume + momentum + whale signals + expiry       │
│  7. Write top 3 opportunities to pipeline.jsonl               │
└──────────────────────┬───────────────────────────────────────┘
                       │ for each opportunity
                       ▼
┌──────────────────────────────────────────────────────────────┐
│  RESEARCHER + ORDERBOOK (parallel)                            │
│                                                               │
│  Researcher:                          Orderbook:              │
│  1. Read contexts/{category}.md       1. Call Kalshi orderbook│
│  2. Read improvements.md              2. Check spread, depth  │
│  3. Read lessons.jsonl (similar)      3. Estimate slippage    │
│  4. Run Perplexity queries            4. Write to pipeline    │
│  5. Check polymaster signals                                  │
│  6. Write to pipeline.jsonl                                   │
└──────────────────────┬───────────────────────────────────────┘
                       ▼
┌──────────────────────────────────────────────────────────────┐
│  RISK MANAGER                                                 │
│  1. Read portfolio.json → current limits and exposure         │
│  2. Check daily spend, position count, correlated positions   │
│  3. Check losing streak → reduce size if active               │
│  4. Write approval/rejection to pipeline.jsonl                │
└──────────────────────┬───────────────────────────────────────┘
                       ▼
┌──────────────────────────────────────────────────────────────┐
│  DECISION MAKER                                               │
│  1. Read ALL pipeline.jsonl entries for this market_id        │
│  2. Read improvements.md for general rules                    │
│  3. Check: all agents agree? avg confidence >= 60%?           │
│  4. Check: multiplier >= 1.8x still? (price may have moved)  │
│  5. Determine tier: auto_execute or approval_required         │
│  6. Write EXECUTE / QUEUE / SKIP to pipeline.jsonl            │
└──────────────────────┬───────────────────────────────────────┘
                       ▼
            ┌──────────────────────┐
            │  auto_execute?       │
            │  YES ──► EXECUTOR    │
            │  NO  ──► notify user │
            └──────────┬───────────┘
                       ▼
┌──────────────────────────────────────────────────────────────┐
│  EXECUTOR                                                     │
│  1. Read config.json → Kalshi credentials                     │
│  2. Place order via Kalshi Trade API                          │
│  3. Set stop_loss and take_profit levels                      │
│  4. Write to trades.jsonl                                     │
│  5. Trigger portfolio tracker update                          │
│  6. Notify user                                               │
└──────────────────────────────────────────────────────────────┘

Every 3 minutes (independent loop):
┌──────────────────────────────────────────────────────────────┐
│  MONITOR                                                      │
│  1. Read portfolio.json → open positions                      │
│  2. Check current prices vs stop_loss / take_profit           │
│  3. Check expiry proximity (close before settlement)          │
│  4. If exit triggered → write to pipeline.jsonl → EXECUTOR    │
└──────────────────────────────────────────────────────────────┘

On every trade event:
┌──────────────────────────────────────────────────────────────┐
│  PORTFOLIO TRACKER                                            │
│  1. Read trades.jsonl → recalculate stats                     │
│  2. Update portfolio.json                                     │
│  3. If losing streak >= 3 → write warning to pipeline.jsonl   │
└──────────────────────────────────────────────────────────────┘

On every loss:
┌──────────────────────────────────────────────────────────────┐
│  POST-MORTEM                                                  │
│  1. Read pipeline.jsonl trail for the losing trade            │
│  2. Run fresh Perplexity research on what actually happened   │
│  3. Compare original reasoning vs reality                     │
│  4. Write lesson to lessons.jsonl                             │
│  5. Regenerate improvements.md from all lessons               │
└──────────────────────────────────────────────────────────────┘
```

Three independent loops running 24/7:
- **Scanner loop** — every N minutes, finds opportunities, triggers pipeline
- **Monitor loop** — every N minutes, watches open positions
- **Post-mortem** — event-driven, fires on losses only

---

## OpenClaw Orchestration — `sessions_spawn`

OpenClaw is the conductor. It uses `sessions_spawn` to run each agent as a sub-agent with the appropriate model and thinking level.

### Example Spawn Sequence

**Scanner (cheap, frequent):**
```
sessions_spawn:
  model: (OpenClaw decides — cheap model, no reasoning needed)
  thinking: off
  timeout: 60s
  task: |
    You are the Market Scanner. Read these files:
    - ~/.openclaw/memory/wwatcher_trading/config.json
    - ~/.openclaw/memory/wwatcher_trading/portfolio.json
    - ~/.openclaw/skills/wwatcher-trader/agents/improvements.md

    Call Kalshi API: GET https://api.elections.kalshi.com/trade-api/v2/markets?status=open
    Filter and rank opportunities per scanner.md instructions.
    Write results to pipeline.jsonl.
```

**Researcher + Orderbook (parallel, moderate):**
```
sessions_spawn: (parallel)
  task: Researcher for {market_id} — load contexts/{category}.md + improvements.md
  task: Orderbook analyst for {market_id} — check Kalshi orderbook
```

**Decision Maker (expensive, high-stakes):**
```
sessions_spawn:
  model: (OpenClaw decides — best available model for high-stakes reasoning)
  thinking: high
  timeout: 180s
  task: |
    You are the Decision Maker for {market_id}.
    Read ALL pipeline.jsonl entries for this market_id.
    Read improvements.md. Read config.json execution_tiers.
    Make the final call: EXECUTE, QUEUE, or SKIP.
```

### Dynamic Model Selection

OpenClaw decides which model and thinking level to assign each spawn. The SKILL.md provides guidance, not rigid rules:

**Principles:**
- CHEAP & FAST for deterministic work (API calls, math, filtering)
- MODERATE for synthesis work (combining data sources, summarizing)
- EXPENSIVE & DEEP only for high-stakes reasoning (trade decisions, loss analysis)
- MATCH frequency to cost — agents that run every 3 minutes must be cheap
- SCALE thinking to uncertainty — clear-cut decisions need no reasoning, ambiguous signals need deep thinking

**Adaptive behavior OpenClaw may apply:**
- If on a losing streak → upgrade researcher to higher thinking
- If a trade is high-value (large position size) → upgrade decision maker
- If system is idle and budget allows → run deeper research passes
- If many opportunities found at once → use cheaper models to parallelize
- If local models available (Ollama) → assign bulk/privacy tasks there

---

## Master SKILL.md — The Orchestrator

This is what OpenClaw loads. It defines the full autonomous behavior:

```markdown
## Trigger
- On install: run guided setup
- On startup: begin scanner loop + monitor loop
- On user message "approve {trade_id}": execute queued trade
- On user message matching preference update: update config.json
- On trade resolution as loss: spawn post-mortem

## Loops

### Scanner Loop
Every {config.scanner.interval_minutes} minutes:
1. Spawn scanner agent
2. Wait for completion, read pipeline.jsonl for new opportunities
3. For each opportunity:
   a. Spawn researcher + orderbook in parallel
   b. Wait for both, spawn risk manager
   c. Wait, spawn decision maker
   d. Read decision:
      - EXECUTE → spawn executor → spawn portfolio tracker
      - QUEUE → notify user, store in pending
      - SKIP → log, continue

### Monitor Loop
Every {config.monitor.interval_minutes} minutes:
1. Spawn monitor agent
2. If exit triggered → spawn executor → spawn portfolio tracker
3. If trade resolved as loss → spawn post-mortem

## User Commands (natural language)
- "show my portfolio" → read portfolio.json, summarize
- "approve trade_004" → spawn executor for queued trade
- "reject trade_004" → mark as rejected in pipeline.jsonl
- "raise my daily limit to $500" → update config.json
- "add sports to my categories" → update config.json
- "pause trading" → stop scanner loop, keep monitor running
- "resume trading" → restart scanner loop
- "show my lessons" → read improvements.md, summarize
- "how are we doing" → read portfolio.json all_time stats
- "what's pending" → list QUEUE status trades from pipeline.jsonl
- "kill all positions" → spawn executor to sell all open positions
```

---

## Files Summary

### New Files (OpenClaw Skill)
| File | Purpose |
|------|---------|
| `skill/SKILL.md` | Master orchestrator instructions |
| `skill/agents/scanner.md` | Market scanner agent prompt |
| `skill/agents/researcher.md` | Deep researcher agent prompt |
| `skill/agents/orderbook.md` | Order book analyst agent prompt |
| `skill/agents/risk.md` | Risk manager agent prompt |
| `skill/agents/decision.md` | Decision maker agent prompt |
| `skill/agents/executor.md` | Trade executor agent prompt |
| `skill/agents/monitor.md` | Position monitor agent prompt |
| `skill/agents/portfolio.md` | Portfolio tracker agent prompt |
| `skill/agents/postmortem.md` | Post-mortem reporter agent prompt |
| `skill/contexts/crypto.md` | Crypto category context |
| `skill/contexts/weather.md` | Weather category context |
| `skill/contexts/politics.md` | Politics category context |
| `skill/contexts/sports.md` | Sports category context |
| `skill/config/setup.md` | Guided setup conversation flow |

### New Files (Runtime Memory)
| File | Purpose |
|------|---------|
| `memory/config.json` | User trading configuration |
| `memory/pipeline.jsonl` | Shared agent findings |
| `memory/trades.jsonl` | Executed trade log |
| `memory/portfolio.json` | Current portfolio state |
| `memory/lessons.jsonl` | Post-mortem learnings |
| `memory/improvements.md` | Active improvement rules |

### Existing Files (No Changes)
Polymaster core (Rust) and the existing integration layer remain untouched. The CLI commands (`research`, `perplexity`, `search`, `score`) are called by OpenClaw agents as tools.
