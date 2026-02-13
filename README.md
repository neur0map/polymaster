# Whale Watcher

A Rust CLI tool that monitors large transactions on Polymarket and Kalshi prediction markets. Real-time alerts for significant market activity with built-in anomaly detection.

Repository: https://github.com/neur0map/polymaster

## DISCLAIMER
This tool is for informational and research purposes only. Use this data solely for informed decision-making and market analysis.

## Features

### Core Monitoring
- Real-time monitoring of Polymarket and Kalshi transactions
- **Kalshi WebSocket** for instant trade detection (HTTP polling fallback)
- **Polymarket server-side filtering** — API pre-filters whale trades, no wasted bandwidth
- Customizable threshold (default $25,000) and polling interval
- Audio alerts with triple beep for repeat actors
- Market category filtering (10 categories, 35+ subcategories)
- Platform selection (Polymarket only, Kalshi only, or both)

### Whale Intelligence (Polymarket)
- **Whale profiles** — portfolio value, leaderboard rank, profit, win rate, open positions
- **Leaderboard lookup** — checks if wallet is in top 500 Polymarket traders
- **12-hour wallet memory** — detects returning whales, position doubling, and whale flips
- **Top holders** — shows top 5 holders and their share concentration per market
- Profiles cached 30 min, leaderboard cached 1 hour

### Market Intelligence
- **Order book depth** for both platforms (Polymarket CLOB + Kalshi orderbook)
- **Market context** — YES/NO odds, spread, 24h volume, open interest, price change, liquidity
- **Native Kalshi categories** from API (more accurate than keyword matching)
- **Polymarket tags** extracted from Gamma API
- Anomaly detection for extreme bets, contrarian positions, and large capital

### Wallet Tracking
- Elevated alerts for repeat actors (2+ txns in 1 hour)
- High priority alerts for heavy actors (5+ txns in 24 hours)
- Tracks volume and transaction frequency per wallet
- Returning whale detection (doubling down, flipping positions)
- SQLite-backed persistent wallet memory across restarts

### Integrations
- **Webhook notifications** to n8n, Zapier, Make, Discord, or any endpoint
- Rich JSON payload with market context, whale profile, order book, and top holders
- **AI agent integration** with RapidAPI and Perplexity for deep market research

### Infrastructure
- SQLite database for alert history and wallet memory
- Configurable data retention (7, 30, 90 days, or forever)
- 6-step guided setup wizard
- No API keys required for basic functionality (all public data)
- Fast and efficient, built with Rust

## Quick Start

```bash
git clone https://github.com/neur0map/polymaster.git
cd polymaster
cargo install --path .
wwatcher watch
```

That's it. No API keys needed — all data comes from public endpoints. You'll see whale alerts in your terminal within seconds.

Run `wwatcher setup` to configure platforms, categories, thresholds, webhooks, and optional API keys.

See [QUICKSTART.md](QUICKSTART.md) for detailed setup instructions and webhook integration.

## Installation

### From Source

```bash
git clone https://github.com/neur0map/polymaster.git
cd polymaster
cargo build --release
```

The binary will be at `target/release/wwatcher`. Or install it system-wide:

```bash
cargo install --path .
```

## API Information

wwatcher uses **15 API endpoints** across 3 Polymarket APIs, 1 Kalshi REST API, and 1 Kalshi WebSocket. See [`docs/API_REFERENCE.md`](docs/API_REFERENCE.md) for complete endpoint documentation.

### Polymarket (3 APIs, no auth required)

| API | Base URL | Endpoints Used |
|-----|----------|---------------|
| Data API | `data-api.polymarket.com` | trades, value, positions, closed-positions, leaderboard, top-holders |
| Gamma API | `gamma-api.polymarket.com` | markets (context + tags) |
| CLOB API | `clob.polymarket.com` | book (order book depth) |

### Kalshi (REST + WebSocket)

| API | Base URL | Endpoints Used |
|-----|----------|---------------|
| REST API | `api.elections.kalshi.com/trade-api/v2` | markets/trades, markets/{ticker}, markets/{ticker}/orderbook |
| WebSocket | `wss://api.elections.kalshi.com/trade-api/ws/v2` | trade channel (real-time) |

All endpoints are public — no API keys needed. For enhanced Kalshi access, run `wwatcher setup`.

### Webhook Integration

wwatcher sends rich JSON payloads to any webhook URL with: market context, whale profile, order book depth, top holders, and wallet activity.

See [`docs/WEBHOOK_REFERENCE.md`](docs/WEBHOOK_REFERENCE.md) for:
- Complete payload schema with all fields
- n8n Telegram/Discord message templates
- Filter examples (heavy actors, top 100 whales, exits only)
- Computed fields (whale quality score, bid/ask imbalance)

## Alert Example

When a whale transaction is detected, you'll see enriched output:

```
[ALERT] LARGE TRANSACTION DETECTED - Polymarket
======================================================================
Question:   Will Bitcoin reach 100k by end of 2026?
Position:   BUYING 'Yes' shares
Prediction: Market believes 'Yes' has 65.0% chance

TRANSACTION DETAILS
Amount:     $50,000.00
Contracts:  76923.08 @ $0.6500 each
Action:     BUY shares
Timestamp:  2026-02-13T18:00:00Z

[WALLET ACTIVITY]
Wallet:   0x742d35...f0bEb
Txns (1h):  2
Txns (24h): 5
Volume (24h): $380,000.00
Status: HEAVY ACTOR (5+ transactions in 24h)

[MARKET CONTEXT]
Odds:          YES 65.0% | NO 35.0%
Spread:        $0.02 (tight)
24h Volume:    $450,000
Open Interest: $2,100,000
24h Move:      +3.2%
Liquidity:     $180,000
Tags:          crypto, bitcoin

[WHALE PROFILE]
Leaderboard:  #45 (TOP 50)
Profit:       +$890,000
Portfolio:    $2,340,000
Open Pos:     12
Win Rate:     73.0%
Markets:      195

[ORDER BOOK]
Best Bid:   $0.6400  |  Best Ask: $0.6600  |  Spread: $0.0200
Bid Depth:  $45,000 (12 levels)  |  Ask Depth: $38,000 (9 levels)
Imbalance:  54% bid / 46% ask (moderate bid pressure)

[TOP HOLDERS]
  #1: 0x742d...bEb — 150,000 shares (12.0%)
  #2: 0x8a3f...1D  — 120,000 shares (9.6%)
  #3: 0x5b9c...aF  — 95,000 shares (7.6%)
  Top 3 control 29.2% of shares
======================================================================
```

## Commands

```bash
wwatcher watch                        # Start monitoring (default: $25k threshold, 5s interval)
wwatcher watch -t 50000               # Set threshold to $50,000
wwatcher watch -t 10000 -i 10         # $10k threshold, 10s polling interval
wwatcher setup                        # 6-step guided configuration wizard
wwatcher status                       # View current configuration and DB stats
wwatcher history                      # View last 20 alerts
wwatcher history -l 50 -p polymarket  # Last 50 Polymarket alerts
wwatcher history --json               # Output alert history as JSON
wwatcher test-sound                   # Test alert sounds
wwatcher test-webhook                 # Send test webhook payloads
```

See [QUICKSTART.md](QUICKSTART.md) for detailed setup instructions.

## Configuration

Configuration is stored at `~/.config/wwatcher/config.json` (macOS/Linux) or `%APPDATA%\wwatcher\config.json` (Windows).

Run `wwatcher setup` for a guided 6-step wizard that configures:
1. **Platforms** — Polymarket, Kalshi, or both
2. **Categories** — Sports, Crypto, Politics, Economics, etc. with subcategory drill-down
3. **Threshold & Retention** — Minimum trade size and history retention period
4. **API Keys** — Optional Kalshi credentials
5. **AI Agent Mode** — Optional RapidAPI + Perplexity keys
6. **Save & Summary**

### Documentation

- [`docs/WEBHOOK_REFERENCE.md`](docs/WEBHOOK_REFERENCE.md) — Full webhook payload schema, n8n templates, filter examples
- [`docs/API_REFERENCE.md`](docs/API_REFERENCE.md) — All 15 API endpoints with request/response examples
- [`docs/plans/`](docs/plans/) — Design documents for each implementation phase

## Development

Built with:

- **Rust** — Systems programming language
- **Tokio** — Async runtime
- **Reqwest** — HTTP client
- **tokio-tungstenite** — WebSocket client (Kalshi real-time trades)
- **rusqlite** — SQLite database (alert history + wallet memory)
- **Clap** — CLI argument parsing
- **Serde** — JSON serialization
- **Colored** — Terminal color output

### Project Structure

```
src/
├── main.rs              # CLI entry point (clap)
├── config.rs            # Config loading/saving
├── db.rs                # SQLite database (schema, queries, migration)
├── categories.rs        # Market category system (10 categories, 35+ subcategories)
├── whale_profile.rs     # Whale intelligence (portfolio, leaderboard, win rate)
├── types.rs             # Shared types, wallet tracker
├── alerts/
│   ├── mod.rs           # AlertData struct, payload builder
│   ├── display.rs       # Terminal output (all display functions)
│   ├── anomaly.rs       # Anomaly detection
│   ├── history.rs       # SQLite alert history
│   ├── sound.rs         # Audio alerts
│   └── webhook.rs       # Webhook sender
├── commands/
│   ├── mod.rs
│   ├── watch.rs         # Main watch loop
│   ├── setup.rs         # 6-step setup wizard
│   ├── status.rs        # Status display
│   └── test.rs          # Sound + webhook tests
├── platforms/
│   ├── mod.rs
│   ├── polymarket.rs    # Polymarket API (trades, market context, order book, top holders)
│   └── kalshi.rs        # Kalshi API (trades, market context, order book, categories)
└── ws/
    ├── mod.rs
    └── kalshi.rs         # Kalshi WebSocket client (real-time trade stream)
```

## Troubleshooting

### No configuration found warning

This is normal. The tool works without configuration using public APIs. Run `wwatcher setup` only if you want to add Kalshi authentication.

### API errors

- Polymarket: Public endpoint should work without issues
- Kalshi: Public endpoint works without auth, but rate limits may apply

### Rate Limiting

If you're getting rate limited:

- Increase the `--interval` to poll less frequently
- For Kalshi: Add API credentials via `wwatcher setup`

## Contributors

Thanks to these contributors for their ideas and improvements:

- [@fuzmik](https://github.com/fuzmik) - Suggested alert history logging feature

## AI Agent Integration

wwatcher includes an AI integration layer that turns whale alerts into actionable research with structured signals. When a whale alert arrives, the agent scores it, runs context-aware research using Perplexity + prediction market data, and delivers a structured signal (bullish/bearish, confidence, key factors).

### Quick Start

```bash
cd integration
npm install
npm run build

# Configure API keys
cat > .env << EOF
PERPLEXITY_API_KEY=your-key-here
RAPIDAPI_KEY=your-key-here
EOF

# Test it works
node dist/cli.js status
node dist/cli.js preferences show
```

### CLI Commands

```bash
node dist/cli.js status                           # Health check
node dist/cli.js alerts --limit=10 --min=50000    # Query alerts (enriched data)
node dist/cli.js summary                          # Aggregate stats
node dist/cli.js search "bitcoin"                 # Search alerts by title/tags
node dist/cli.js score '<alert_json>'             # Score alert → tier + factors
node dist/cli.js research "BTC above 100k"        # Full research (generic)
node dist/cli.js research "BTC above 100k" --context='<alert_json>'  # Context-aware research
node dist/cli.js fetch "BTC price above 100k"     # RapidAPI data only
node dist/cli.js preferences show                 # Show preference schema
```

### Alert Scoring

Every alert is scored based on whale profile, trade size, order book imbalance, and position type:

| Tier | Score | Meaning |
|------|-------|---------|
| **High** | >= 60 | Top trader, large bet, strong signals |
| **Medium** | >= 35 | Known trader or significant trade |
| **Low** | < 35 | Unknown trader, smaller trade |

### User Preferences (OpenClaw)

Users can set natural language filters that OpenClaw stores in memory:

- "Only alert me on whales with 60%+ win rate"
- "Skip anything under $100k"
- "Only crypto and politics markets"
- "Top 100 leaderboard traders only"

See [`integration/skill/SKILL.md`](./integration/skill/SKILL.md) for the full preference schema.

### Modular Provider System

Providers are organized by category in `integration/providers/`:

```
providers/
├── crypto.json              # Coinranking (BTC, ETH, SOL prices)
├── sports.json              # NBA API (games, scores)
├── weather.json             # Meteostat (forecasts)
├── news.json                # Cryptocurrency News
├── prediction-markets.json  # Polymarket + Kalshi (related markets, cross-platform)
└── README.md                # How to add more providers
```

Prediction market data (related markets, cross-platform matching, price history) requires **no API keys** — it calls public Polymarket and Kalshi endpoints directly.

### OpenClaw Skill Installation

```bash
mkdir -p ~/.openclaw/skills/wwatcher-ai
cp integration/skill/SKILL.md ~/.openclaw/skills/wwatcher-ai/SKILL.md
```

### MCP Server

```bash
npm run start:mcp
```

Add to your MCP config:
```json
{
  "mcpServers": {
    "wwatcher": {
      "command": "node",
      "args": ["/path/to/integration/dist/index.js"]
    }
  }
}
```

MCP tools return full enriched data (whale profile, order book, top holders, tags) and support filtering by win rate, leaderboard rank, and tags.

### RapidAPI Setup (Optional)

Your single API key works for all subscribed services. Subscribe to these (free tiers available):

| Category | API | Link |
|----------|-----|------|
| Crypto | Coinranking | [rapidapi.com/Coinranking/api/coinranking1](https://rapidapi.com/Coinranking/api/coinranking1) |
| Sports | NBA API | [rapidapi.com/api-sports/api/nba-api-free-data](https://rapidapi.com/api-sports/api/nba-api-free-data) |
| Weather | Meteostat | [rapidapi.com/meteostat/api/meteostat](https://rapidapi.com/meteostat/api/meteostat) |
| News | Crypto News | [rapidapi.com/Starter-api/api/cryptocurrency-news2](https://rapidapi.com/Starter-api/api/cryptocurrency-news2) |

### Documentation

- [`integration/skill/SKILL.md`](./integration/skill/SKILL.md) — OpenClaw skill (scoring, preferences, workflow)
- [`integration/providers/README.md`](./integration/providers/README.md) — Adding custom providers
- [`instructions_for_ai_agent.md`](./instructions_for_ai_agent.md) — Complete agent instructions
- [`integration/README.md`](./integration/README.md) — CLI and MCP server details

## License

This tool is for educational and monitoring purposes. Review the terms of service for Polymarket and Kalshi APIs.

