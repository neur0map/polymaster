# wwatcher Integration Instructions

## How It Works

```
┌─────────────┐     ┌─────────┐     ┌──────────┐     ┌──────────┐     ┌────────────┐
│  wwatcher   │────▶│   n8n   │────▶│ Telegram │────▶│  Agent   │────▶│  Scored    │
│  (Rust CLI) │     │ webhook │     │ message  │     │ score +  │     │  Analysis  │
└─────────────┘     └─────────┘     └──────────┘     │ analyze  │     └────────────┘
                                                      └──────────┘
```

1. **wwatcher** monitors Polymarket/Kalshi for whale transactions
2. **Webhook** fires with enriched payload (whale profile, order book, market context, top holders)
3. **n8n** receives webhook, sends alert to your messaging platform
4. **Agent** scores the alert, checks user preferences, analyzes the data
5. **Scored Analysis** delivered with tier, factors, and whale quality assessment

---

## Setup Guide

### Step 1: Install & Build wwatcher

```bash
git clone https://github.com/neur0map/polymaster.git
cd polymaster
cargo install --path .
```

### Step 2: Run Setup Wizard

```bash
wwatcher setup
```

The wizard configures:
- **Platforms** — Polymarket, Kalshi, or both
- **Categories** — Sports, Crypto, Politics, Economics, etc.
- **Threshold & Retention** — Minimum trade size and history retention
- **API Keys** — Optional Kalshi credentials
- **Webhook URL** — Your n8n endpoint

### Step 3: Build the Integration

```bash
cd integration
npm install
npm run build
```

### Step 4: Configure n8n Workflow

Create an n8n workflow:

**Trigger Node: Webhook**
- Method: POST
- Path: `/webhook/whale-alerts`

**Action Node: Telegram/Discord/Slack**
- Send the full alert JSON to your agent's chat

### Step 5: Start wwatcher

```bash
wwatcher watch --threshold 35000 --interval 5
```

---

## Agent Workflow

When the agent receives a whale alert, it executes this workflow:

### 1. Parse the Alert

Extract the full enriched payload including:
- **Core**: platform, action, value, price, market_title, outcome, timestamp
- **Wallet**: wallet_id, wallet_activity (repeat/heavy actor, txn counts)
- **Whale Profile**: leaderboard_rank, win_rate, portfolio_value, positions_count
- **Market Context**: yes_price, no_price, spread, volume_24h, open_interest, tags
- **Order Book**: best_bid, best_ask, bid_depth_10pct, ask_depth_10pct
- **Top Holders**: top 5 holders with shares and percentages

### 2. Check User Preferences

Use the `check_preferences` MCP tool. If the alert doesn't pass, silently skip (unless `debug: true`, then log the skip reason).

Preference fields (all optional):
- `min_value` — Minimum trade size (e.g., 100000)
- `min_win_rate` — Minimum whale win rate (e.g., 0.6)
- `max_leaderboard_rank` — Maximum rank (e.g., 100)
- `max_odds` — Skip if the side being bought has odds above this (e.g., 0.80). Filters out near-certainties where you need massive capital for small gains.
- `platforms` — Platform filter (e.g., ["polymarket"])
- `categories` — Category filter (e.g., ["crypto", "politics"])
- `directions` — Action filter (e.g., ["buy"])
- `tier_filter` — Minimum tier ("high" or "medium")
- `debug` — If true, log skip reasons instead of silent skip (default: false)

**Odds filter logic:** When `max_odds` is set, check the price of the side being bought:
- BUY YES action → check `yes_price` <= max_odds
- BUY NO action → check `no_price` <= max_odds
- SELL actions → no odds filter (exiting position)

Natural language examples:
- "Only alert me on whales with 60%+ win rate" → `{ "min_win_rate": 0.6 }`
- "Skip anything under $100k" → `{ "min_value": 100000 }`
- "Top 100 leaderboard traders only" → `{ "max_leaderboard_rank": 100 }`
- "Only high tier alerts" → `{ "tier_filter": "high" }`
- "Skip near-certainties over 80% odds" → `{ "max_odds": 0.80 }`
- "Show me why alerts are being skipped" → `{ "debug": true }`

### 3. Score the Alert

Use the `score_alert` MCP tool. Pass the full alert JSON. Returns tier + factors.

### 4. Deliver Analysis

Present the scored alert with whale quality assessment, order book analysis, and key factors.

---

## Alert Scoring

Every alert is scored based on whale profile, trade size, order book, and position type:

| Factor | Signal | Score |
|--------|--------|-------|
| Leaderboard top 10 | Elite trader | +30 |
| Leaderboard top 50 | Strong trader | +25 |
| Leaderboard top 100 | Known trader | +20 |
| Leaderboard top 500 | Ranked trader | +10 |
| Win rate >= 80% | Elite accuracy | +20 |
| Win rate >= 70% | Strong accuracy | +15 |
| Win rate >= 60% | Above average | +10 |
| Heavy actor (5+ txns/24h) | High conviction | +15 |
| Repeat actor (2+ txns/1h) | Active trader | +10 |
| Trade >= $250k | Massive bet | +20 |
| Trade >= $100k | Large bet | +15 |
| Trade >= $50k | Significant bet | +10 |
| Bid imbalance >= 65% | Directional pressure | +10 |
| Contrarian position | Against consensus | +15 |
| Portfolio >= $1M | Whale portfolio | +10 |

**Tier thresholds:**
- **High**: score >= 60 — Top trader, large bet, strong signals
- **Medium**: score >= 35 — Known trader or significant trade
- **Low**: score < 35 — Unknown trader, smaller trade

---

## MCP Tools

| Tool | Input | Output |
|------|-------|--------|
| `score_alert` | `{ alert: "<json>" }` | `{ score, tier, factors }` |
| `check_preferences` | `{ alert: "<json>", preferences: "<json>" }` | `{ passes: boolean }` |

---

## File Locations

| Item | Path |
|------|------|
| wwatcher config | `~/.config/wwatcher/config.json` |
| Alert database | `~/.config/wwatcher/wwatcher.db` |
