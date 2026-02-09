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

When whale alerts fire, you execute the full research workflow:

1. **Check preferences** → Does alert match user's interests?
2. **Investigate** → RapidAPI data + 5 Perplexity searches
3. **Analyze** → Study findings, compare to whale's position
4. **Predict** → Should user follow? Probability vs current odds
5. **Report** → Deliver summary with recommendation

**Don't just forward alerts** — add value through research and prediction.

---

## OpenClaw Setup

### Build & Configure

```bash
cd /home/neur0map/polymaster-test/integration
npm install && npm run build

# Set your API keys (BOTH required for full research)
cat > .env << EOF
RAPIDAPI_KEY=your-rapidapi-key
PERPLEXITY_API_KEY=your-perplexity-key
EOF

# Verify
node dist/cli.js status
```

### Install Skill

```bash
mkdir -p ~/.openclaw/skills/wwatcher-ai
cp skill/SKILL.md ~/.openclaw/skills/wwatcher-ai/SKILL.md
```

### CLI Commands

```bash
cd /home/neur0map/polymaster-test/integration

node dist/cli.js status                              # Health check
node dist/cli.js alerts --limit=10 --min=50000       # Query alerts
node dist/cli.js summary                             # Aggregate stats
node dist/cli.js search "bitcoin"                    # Search alerts
node dist/cli.js fetch "Bitcoin above 100k"          # RapidAPI data only
node dist/cli.js perplexity "BTC ETF inflows"        # Single Perplexity query
node dist/cli.js research "Bitcoin above 100k"       # FULL RESEARCH
```

---

## MCP Setup

### Configure MCP Client

```json
{
  "mcpServers": {
    "wwatcher": {
      "command": "node",
      "args": ["/home/neur0map/polymaster-test/integration/dist/index.js"],
      "env": {
        "RAPIDAPI_KEY": "your-key",
        "PERPLEXITY_API_KEY": "your-key"
      }
    }
  }
}
```

### MCP Tools

| Tool | Description |
|------|-------------|
| `get_recent_alerts` | Query alerts with filters |
| `get_alert_summary` | Aggregate stats |
| `search_alerts` | Text search |
| `fetch_market_data` | RapidAPI data |
| `get_wwatcher_status` | Health check |

---

## Full Research Workflow

### When You Receive an Alert

#### Step 1: Check User Preferences
Does this alert match what the user wants to track?
- Categories: crypto, sports, politics, weather
- Minimum value threshold
- Specific keywords

If NO match → skip or notify briefly
If MATCH → continue to full research

#### Step 2: Run Full Research
```bash
node dist/cli.js research "Market title" --category=crypto
```

This executes:
- **RapidAPI fetch**: Current prices, odds, forecasts
- **5 Perplexity searches**:
  1. Latest news and developments
  2. Expert analysis and predictions
  3. Historical data and trends
  4. Risk factors and uncertainties
  5. Recent events affecting outcome

#### Step 3: Analyze the Data

Study the research output:
- What does the current data show?
- What are experts saying?
- What risks exist?
- Why might the whale be making this bet?

#### Step 4: Generate Prediction Report

```
## 🐋 Whale Alert Analysis

**Alert**: [platform] [action] $[value] on "[market]" at [price]%
**Whale Profile**: [repeat actor? heavy actor?]

---

### Research Findings

**Market Data (RapidAPI)**:
- [Current price/odds/forecast]
- [Trend or momentum]

**Web Research (Perplexity)**:
- [Key finding 1 + source]
- [Key finding 2 + source]
- [Key finding 3 + source]
- [Key finding 4 + source]
- [Key finding 5 + source]

---

### Prediction

**Should you follow the whale?** [YES / NO / PARTIAL]

**Reasoning**:
[2-3 sentences explaining your analysis based on the research]

**Probability Estimate**: [X]%
**Current Market Odds**: [Y]%
**Edge**: [+/-Z]% — [whale sees higher/lower probability than market]

**Confidence**: [Low / Medium / High]

**Key Risk Factors**:
- [Risk 1]
- [Risk 2]

---

### Recommendation

[One clear, actionable sentence]
```

---

## User Preferences

Users can configure what to track:

```json
{
  "categories": ["crypto", "politics"],
  "keywords": ["bitcoin", "ethereum", "election", "trump"],
  "min_value": 25000,
  "platforms": ["polymarket"],
  "alert_on_all": false
}
```

When `alert_on_all` is false, only run full research on matching alerts.

---

## API Keys Required

| Key | Purpose | Required | Get it at |
|-----|---------|----------|-----------|
| `RAPIDAPI_KEY` | Market data | Yes (for AI) | [rapidapi.com](https://rapidapi.com) |
| `PERPLEXITY_API_KEY` | Web research | Yes (for AI) | [perplexity.ai/settings/api](https://perplexity.ai/settings/api) |

### RapidAPI Subscriptions (free tiers)

| Category | API |
|----------|-----|
| Crypto | [Coinranking](https://rapidapi.com/Coinranking/api/coinranking1) |
| Sports | [NBA API](https://rapidapi.com/api-sports/api/nba-api-free-data) |
| Weather | [Meteostat](https://rapidapi.com/meteostat/api/meteostat) |
| News | [Crypto News](https://rapidapi.com/Starter-api/api/cryptocurrency-news2) |

---

## File Locations

| Item | Path |
|------|------|
| Alert history | `~/.config/wwatcher/alert_history.jsonl` |
| Config | `~/.config/wwatcher/config.json` |
| API keys | `integration/.env` |
| Providers | `integration/providers/` |
| CLI | `integration/dist/cli.js` |
| MCP Server | `integration/dist/index.js` |
