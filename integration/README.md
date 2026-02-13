# wwatcher-ai Integration

AI agent integration for wwatcher whale alert monitoring. Includes:
- **CLI tool** for OpenClaw and shell-based agents
- **MCP server** for MCP-compatible clients
- **Alert scoring** with weighted factors (whale rank, win rate, trade size, order book)
- **Context-aware research** with structured signals (bullish/bearish, confidence, factors)
- **Prediction market data** from Polymarket + Kalshi public APIs (no keys needed)
- **User preferences** for filtering alerts by win rate, rank, platform, category
- **RapidAPI integration** for contextual market data (crypto, sports, weather, news)

## Quick Start

```bash
cd integration
npm install
npm run build
```

## Configuration

1. Copy `.env.example` to `.env`
2. Add your Perplexity key: `PERPLEXITY_API_KEY=your-key` (required for research)
3. Add your RapidAPI key: `RAPIDAPI_KEY=your-key` (optional, for market data enrichment)

| Key | Required | Purpose | Get it at |
|-----|----------|---------|-----------|
| `PERPLEXITY_API_KEY` | For research | Web-based analysis | [perplexity.ai/settings/api](https://perplexity.ai/settings/api) |
| `RAPIDAPI_KEY` | Optional | Market data | [rapidapi.com](https://rapidapi.com) |

Prediction market data (related markets, cross-platform matching) requires **no API keys**.

---

## Option 1: CLI (OpenClaw / Shell)

For OpenClaw agents or any shell-based automation.

### Commands

```bash
# Health check
node dist/cli.js status

# Query alerts (returns enriched data: whale profile, order book, tags)
node dist/cli.js alerts --limit=10 --min=50000
node dist/cli.js alerts --platform=polymarket --type=WHALE_ENTRY

# Aggregate stats (avg whale rank, avg bid depth)
node dist/cli.js summary

# Search alerts (searches titles, outcomes, and tags)
node dist/cli.js search "bitcoin"

# Score an alert — returns tier (high/medium/low) + factors
node dist/cli.js score '<alert_json>'

# Full research — context-aware with structured signal
node dist/cli.js research "Bitcoin above 100k" --context='<alert_json>'

# Full research — generic (no alert context)
node dist/cli.js research "Bitcoin above 100k" --category=crypto

# Fetch RapidAPI market data only
node dist/cli.js fetch "Bitcoin price above 100k"
node dist/cli.js fetch "Lakers vs Celtics" --category=sports

# Single Perplexity search
node dist/cli.js perplexity "What are Bitcoin ETF inflows?"

# Show preference schema
node dist/cli.js preferences show
```

### CLI Options

**alerts:**
| Option | Description |
|--------|-------------|
| `--limit=N` | Max alerts to return (default: 20) |
| `--platform=X` | Filter: polymarket, kalshi |
| `--type=X` | Filter: WHALE_ENTRY, WHALE_EXIT |
| `--min=N` | Minimum USD value |
| `--since=ISO` | Alerts after timestamp |

**research:**
| Option | Description |
|--------|-------------|
| `--context=JSON` | Full alert JSON for context-aware research (auto-scores, targeted queries, structured signal) |
| `--category=X` | Override category: crypto, sports, weather, news, politics |
| `--queries=N` | Number of Perplexity queries (default: 3 with context, 5 without) |

**fetch:**
| Option | Description |
|--------|-------------|
| `--category=X` | Override: weather, crypto, sports, news |

### Alert Scoring

The `score` command analyzes an alert and returns a tier with factors:

```bash
node dist/cli.js score '{"platform":"polymarket","action":"BUY","value":150000,...}'
```

```json
{
  "score": 80,
  "tier": "high",
  "factors": [
    "Top 50 leaderboard trader (#45)",
    "Strong win rate (73%)",
    "Large portfolio ($2.3M)",
    "Heavy actor (6 txns/24h)",
    "Large trade ($150k)"
  ]
}
```

### Context-Aware Research

When `--context` is provided, the research command:
1. Scores the alert automatically
2. Generates 3 targeted Perplexity queries based on score factors
3. Fetches prediction market data (related markets, cross-platform match)
4. Returns a structured signal:

```json
{
  "signal": {
    "direction": "bullish",
    "confidence": "high",
    "factors": ["Top 50 trader (#45)", "Strong win rate (73%)", ...],
    "whale_quality": "Rank #45, 73% win rate, $2.3M portfolio",
    "market_pressure": "Bid pressure (64/36 bid/ask)",
    "research_summary": "Bitcoin ETF inflows hit $500M this week..."
  }
}
```

### OpenClaw Skill Installation

```bash
mkdir -p ~/.openclaw/skills/wwatcher-ai
cp skill/SKILL.md ~/.openclaw/skills/wwatcher-ai/SKILL.md
```

The skill supports:
- Automatic alert parsing and scoring
- User preferences via OpenClaw memory (`wwatcher_preferences` key)
- Natural language filter management ("only 60%+ win rate", "skip under $100k")
- Structured signal delivery

---

## Option 2: MCP Server

For MCP-compatible clients.

### Setup

Add to your MCP client config:

```json
{
  "mcpServers": {
    "wwatcher": {
      "command": "node",
      "args": ["/absolute/path/to/integration/dist/index.js"],
      "env": {
        "PERPLEXITY_API_KEY": "your-key",
        "RAPIDAPI_KEY": "your-key"
      }
    }
  }
}
```

### Start MCP Server

```bash
npm run start:mcp
# or
node dist/index.js
```

### MCP Tools

| Tool | Description |
|------|-------------|
| `get_recent_alerts` | Query alert history with filters (platform, type, value, tags, win rate, rank) |
| `get_alert_summary` | Aggregate stats: volume, top markets, whale counts, avg rank, avg bid depth |
| `search_alerts` | Text search in market titles, outcomes, and tags |
| `fetch_market_data` | Pull RapidAPI data based on market keywords |
| `get_wwatcher_status` | Health check |

All alert tools return enriched data: whale profile, order book, top holders, market context, tags.

### Modes

- `--mode=realtime` (default) — watches for new alerts in real-time
- `--mode=snapshot` — loads existing history only

---

## Providers

Providers are organized by category in the `providers/` directory:

```
providers/
├── README.md                # Full documentation for adding providers
├── crypto.json              # Coinranking (BTC, ETH, SOL prices)
├── sports.json              # NBA API (games, scores)
├── weather.json             # Meteostat (forecasts)
├── news.json                # Cryptocurrency News
└── prediction-markets.json  # Polymarket + Kalshi (related markets, cross-platform)
```

### Adding a Provider

**Option 1**: Add to an existing category file (e.g., `providers/crypto.json`)

**Option 2**: Create a new category file (e.g., `providers/politics.json`)

```json
{
  "provider_key": {
    "name": "Display Name",
    "category": "politics",
    "rapidapi_host": "api.p.rapidapi.com",
    "env_key": "RAPIDAPI_KEY",
    "keywords": ["election", "president", "vote"],
    "endpoints": {
      "markets": {
        "method": "GET",
        "path": "/v1/markets",
        "description": "What it returns",
        "params": {}
      }
    }
  }
}
```

The system automatically loads all `*.json` files from the `providers/` directory.

See [`providers/README.md`](./providers/README.md) for the complete schema and examples.

---

## For AI Agents

See [`instructions_for_ai_agent.md`](../instructions_for_ai_agent.md) for complete agent instructions including:
- Scoring system and tier thresholds
- Context-aware research workflow
- Structured signal format
- User preference management
- Pattern detection
