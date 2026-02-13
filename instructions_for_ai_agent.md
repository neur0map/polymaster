# wwatcher AI Agent Instructions

## How It Works

```
┌─────────────┐     ┌─────────┐     ┌──────────┐     ┌──────────┐     ┌────────────┐
│  wwatcher   │────▶│   n8n   │────▶│ Telegram │────▶│ AI Agent │────▶│ Structured │
│  (Rust CLI) │     │ webhook │     │ message  │     │ score +  │     │   Signal   │
└─────────────┘     └─────────┘     └──────────┘     │ research │     └────────────┘
                                                      └──────────┘
```

1. **wwatcher** monitors Polymarket/Kalshi for whale transactions
2. **Webhook** fires with enriched payload (whale profile, order book, market context, top holders)
3. **n8n** receives webhook, sends alert to your messaging platform
4. **AI Agent** scores the alert, checks user preferences, runs context-aware research
5. **Structured Signal** delivered with direction, confidence, and key factors

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
- **AI Agent Mode** — Optional RapidAPI + Perplexity keys
- **Webhook URL** — Your n8n endpoint

### Step 3: Build the AI Integration

```bash
cd integration
npm install
npm run build

# Set API keys
cat > .env << EOF
PERPLEXITY_API_KEY=your-key
RAPIDAPI_KEY=your-key
EOF

# Test
node dist/cli.js status
```

### Step 4: Configure n8n Workflow

Create an n8n workflow:

**Trigger Node: Webhook**
- Method: POST
- Path: `/webhook/whale-alerts`

**Action Node: Telegram/Discord/Slack**
- Send the full alert JSON to your AI agent's chat
- Include "Research this whale alert." to trigger the skill

### Step 5: Start wwatcher

```bash
wwatcher watch --threshold 35000 --interval 5
```

---

## AI Agent Workflow

When the AI agent receives a whale alert, it executes this workflow:

### 1. Parse the Alert

Extract the full enriched payload including:
- **Core**: platform, action, value, price, market_title, outcome, timestamp
- **Wallet**: wallet_id, wallet_activity (repeat/heavy actor, txn counts)
- **Whale Profile**: leaderboard_rank, win_rate, portfolio_value, positions_count
- **Market Context**: yes_price, no_price, spread, volume_24h, open_interest, tags
- **Order Book**: best_bid, best_ask, bid_depth_10pct, ask_depth_10pct
- **Top Holders**: top 5 holders with shares and percentages

### 2. Check User Preferences

Load preferences from memory key `wwatcher_preferences`. If any filter fails, silently skip.

Preference fields (all optional):
- `min_value` — Minimum trade size (e.g., 100000)
- `min_win_rate` — Minimum whale win rate (e.g., 0.6)
- `max_leaderboard_rank` — Maximum rank (e.g., 100)
- `platforms` — Platform filter (e.g., ["polymarket"])
- `categories` — Category filter (e.g., ["crypto", "politics"])
- `directions` — Action filter (e.g., ["buy"])
- `tier_filter` — Minimum tier ("high" or "medium")

Natural language examples:
- "Only alert me on whales with 60%+ win rate" → `{ "min_win_rate": 0.6 }`
- "Skip anything under $100k" → `{ "min_value": 100000 }`
- "Top 100 leaderboard traders only" → `{ "max_leaderboard_rank": 100 }`
- "Only high tier alerts" → `{ "tier_filter": "high" }`

### 3. Score + Context-Aware Research

```bash
node dist/cli.js research "Market title" --context='<full_alert_json>'
```

This single command:
1. **Scores** the alert (tier: high/medium/low + factors list)
2. **Generates 3 targeted Perplexity queries** based on score factors (not 5 generic ones)
3. **Fetches prediction market data** — related markets, cross-platform match (no API key needed)
4. **Fetches RapidAPI data** if relevant providers match the category
5. **Returns a structured signal** with direction, confidence, and factors

### 4. Deliver Structured Signal

```
WHALE SIGNAL: [market_title]

Direction: [BULLISH/BEARISH] | Confidence: [HIGH/MEDIUM/LOW]

Key Factors:
- [factor 1]
- [factor 2]
- [factor 3]

Whale: [whale_quality summary]
Book: [market_pressure summary]

Research: [2-3 sentence research_summary]

Cross-Platform: [if cross_platform match found, show title + price]
Related Markets: [if related markets found, list top 2-3]
```

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

## CLI Reference

```bash
cd integration

# Health check
node dist/cli.js status

# Query alerts (enriched with whale profile, order book, tags)
node dist/cli.js alerts --limit=10 --min=50000
node dist/cli.js alerts --platform=polymarket --type=WHALE_ENTRY

# Aggregate stats (avg whale rank, avg bid depth)
node dist/cli.js summary

# Search alerts (titles, outcomes, tags)
node dist/cli.js search "bitcoin"

# Score an alert — returns tier + factors
node dist/cli.js score '<alert_json>'

# Context-aware research — scoring + targeted queries + structured signal
node dist/cli.js research "Bitcoin above 100k" --context='<alert_json>'

# Generic research (no alert context)
node dist/cli.js research "Bitcoin above 100k" --category=crypto

# RapidAPI data only
node dist/cli.js fetch "Bitcoin price above 100k"

# Single Perplexity search
node dist/cli.js perplexity "What are Bitcoin ETF inflows?"

# Show user preference schema
node dist/cli.js preferences show
```

---

## Environment Detection (for AI Agents)

| If you are... | Integration |
|---------------|-------------|
| **OpenClaw** (exec tools, skills) | Use CLI commands above |
| **MCP client** (Claude Code, etc.) | Use MCP server (`npm run start:mcp`) |

---

## API Keys

| Key | Required | Purpose | Get it at |
|-----|----------|---------|-----------|
| `PERPLEXITY_API_KEY` | For research | Web-based analysis | [perplexity.ai/settings/api](https://perplexity.ai/settings/api) |
| `RAPIDAPI_KEY` | Optional | Market data enrichment | [rapidapi.com](https://rapidapi.com) |

Prediction market data (related markets, cross-platform matching) requires **no API keys**.

### RapidAPI Subscriptions (free tiers)

- [Coinranking](https://rapidapi.com/Coinranking/api/coinranking1) (crypto prices)
- [NBA API](https://rapidapi.com/api-sports/api/nba-api-free-data) (sports)
- [Meteostat](https://rapidapi.com/meteostat/api/meteostat) (weather)
- [Crypto News](https://rapidapi.com/Starter-api/api/cryptocurrency-news2) (news)

---

## Category Guide

| Category | RapidAPI Data | Research Focus |
|----------|---------------|----------------|
| crypto | Coinranking prices | On-chain data, institutional flows, technicals |
| sports | Game data | Injuries, odds movement, matchups |
| weather | Meteostat forecast | Model confidence, patterns |
| politics | — | Polls, demographics, developments |
| prediction-markets | Polymarket/Kalshi | Related markets, cross-platform, price history |

---

## File Locations

| Item | Path |
|------|------|
| wwatcher config | `~/.config/wwatcher/config.json` |
| Alert database | `~/.config/wwatcher/wwatcher.db` |
| Integration .env | `integration/.env` |
| Providers | `integration/providers/` |
| OpenClaw skill | `integration/skill/SKILL.md` |
