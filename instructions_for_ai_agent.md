# wwatcher AI Agent Instructions

## Environment Detection

**Read this first to determine your integration path:**

| If you are... | Use this integration | Jump to section |
|---------------|---------------------|-----------------|
| **OpenClaw agent** (skill-based, exec tools) | CLI | [OpenClaw Setup](#openclaw-setup) |
| **Claude Code / Cursor / MCP client** | MCP Server | [MCP Setup](#mcp-setup) |
| **Other shell-based agent** | CLI | [OpenClaw Setup](#openclaw-setup) |

**How to tell:**
- If you have access to `exec` tool and run shell commands → Use **CLI**
- If you connect via MCP protocol with tool definitions → Use **MCP Server**

---

## What This Project Is

**wwatcher** (Whale Watcher) monitors large transactions ("whale bets") on prediction markets:
- **Polymarket** — crypto-native prediction market
- **Kalshi** — regulated US prediction market

Alerts are logged to `~/.config/wwatcher/alert_history.jsonl`.

## Your Role

When whale alerts fire, you:
1. **Investigate** — Pull contextual data (prices, odds, forecasts, news)
2. **Analyze** — Synthesize data against the whale's position
3. **Deliver insight** — Provide research-backed analysis with probability estimates

**Don't just forward alerts** — add value through research and synthesis.

---

## OpenClaw Setup

### Build & Configure

```bash
cd /home/neur0map/polymaster-test/integration
npm install && npm run build

# Set your API key
echo "RAPIDAPI_KEY=your-key" > .env

# Verify
node dist/cli.js status
```

### Install Skill

```bash
mkdir -p ~/.openclaw/skills/wwatcher-ai
cp skill/SKILL.md ~/.openclaw/skills/wwatcher-ai/SKILL.md
```

### CLI Commands

All commands run from the integration directory:

```bash
cd /home/neur0map/polymaster-test/integration

node dist/cli.js status                              # Health check
node dist/cli.js alerts --limit=10 --min=50000       # Query alerts
node dist/cli.js summary                             # Aggregate stats
node dist/cli.js search "bitcoin"                    # Search alerts
node dist/cli.js fetch "Bitcoin above 100k"          # Fetch market data
node dist/cli.js fetch "Lakers game" --category=sports
```

**Alert options:** `--limit`, `--platform`, `--type`, `--min`, `--since`
**Fetch options:** `--category` (weather, crypto, sports, news)

---

## MCP Setup

### Configure MCP Client

Add to your MCP client config (Claude Code, Cursor, etc.):

```json
{
  "mcpServers": {
    "wwatcher": {
      "command": "node",
      "args": ["/home/neur0map/polymaster-test/integration/dist/index.js"],
      "env": {
        "RAPIDAPI_KEY": "your-key"
      }
    }
  }
}
```

### Start Server

```bash
cd /home/neur0map/polymaster-test/integration
npm run start:mcp
```

### MCP Tools Available

| Tool | Description |
|------|-------------|
| `get_recent_alerts` | Query alerts with filters |
| `get_alert_summary` | Aggregate stats |
| `search_alerts` | Text search |
| `fetch_market_data` | Pull RapidAPI data |
| `get_wwatcher_status` | Health check |

---

## Research Workflow

### When You Receive an Alert

1. **Parse the alert**: platform, action, value, market, outcome, price
2. **Fetch relevant data**: 
   ```bash
   node dist/cli.js fetch "market title" --category=crypto
   ```
3. **Analyze**: Compare whale position to market data
4. **Deliver insight**:

```
🐋 Whale Alert: $50K on "Bitcoin above 100k" — YES at 65%

What the whale did: Bought $50K YES at 65% implied probability

What the data shows:
- BTC currently at $97,200, up 3% today
- 30-day trend: +15%
- Key resistance at $99,500

My take: Whale is betting on momentum continuation. With BTC 3% from target 
and strong trend, the 65% price looks reasonable. Risk is rejection at 
resistance — if it fails twice, expect pullback.

Probability estimate: 60-65% (aligns with market)
```

### Category-Specific Research

| Category | Data Source | Key Analysis |
|----------|-------------|--------------|
| Crypto | Coinranking | Price vs entry, momentum/contrarian |
| Sports | NBA API | Odds vs whale bet, injuries, form |
| Weather | Meteostat | Forecast vs threshold, confidence |
| News | Crypto News | Recent events, information edge |

---

## Providers

Providers are in `integration/providers/` (one JSON file per category):

```
providers/
├── crypto.json     # Coinranking
├── sports.json     # NBA API
├── weather.json    # Meteostat
└── news.json       # Crypto News
```

### Adding Providers

Edit existing category file or create new one:

```json
{
  "provider_key": {
    "name": "Display Name",
    "category": "crypto",
    "rapidapi_host": "api.p.rapidapi.com",
    "env_key": "RAPIDAPI_KEY",
    "keywords": ["bitcoin", "btc", "ethereum"],
    "endpoints": {
      "price": {
        "method": "GET",
        "path": "/v1/price",
        "description": "Get current price",
        "params": {}
      }
    }
  }
}
```

See `providers/README.md` for full schema.

---

## File Locations

| Item | Path |
|------|------|
| Alert history | `~/.config/wwatcher/alert_history.jsonl` |
| API key | `integration/.env` |
| Providers | `integration/providers/` |
| CLI | `integration/dist/cli.js` |
| MCP Server | `integration/dist/index.js` |
| Skill | `~/.openclaw/skills/wwatcher-ai/SKILL.md` |

---

## RapidAPI Subscriptions

Your single key works for all subscribed APIs (free tiers available):

| Category | API | Link |
|----------|-----|------|
| Crypto | Coinranking | [rapidapi.com/Coinranking/api/coinranking1](https://rapidapi.com/Coinranking/api/coinranking1) |
| Sports | NBA API | [rapidapi.com/api-sports/api/nba-api-free-data](https://rapidapi.com/api-sports/api/nba-api-free-data) |
| Weather | Meteostat | [rapidapi.com/meteostat/api/meteostat](https://rapidapi.com/meteostat/api/meteostat) |
| News | Crypto News | [rapidapi.com/Starter-api/api/cryptocurrency-news2](https://rapidapi.com/Starter-api/api/cryptocurrency-news2) |
