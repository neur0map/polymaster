# wwatcher AI Agent Instructions

## How It Works

```
┌─────────────┐     ┌─────────┐     ┌──────────┐     ┌──────────┐     ┌────────────┐
│  wwatcher   │────▶│   n8n   │────▶│ Telegram │────▶│ AI Agent │────▶│ Prediction │
│  (Rust CLI) │     │ webhook │     │ message  │     │ research │     │   Report   │
└─────────────┘     └─────────┘     └──────────┘     └──────────┘     └────────────┘
```

1. **wwatcher** monitors Polymarket/Kalshi for whale transactions
2. **Webhook** fires when whale exceeds threshold (e.g., $35K)
3. **n8n** receives webhook, sends alert to your Telegram chat
4. **AI Agent** sees the message, runs full research workflow
5. **Prediction Report** delivered with recommendation

---

## Setup Guide

### Step 1: Install & Build wwatcher

```bash
# Clone the repo
git clone https://github.com/neur0map/polymaster.git
cd polymaster

# Build the Rust CLI
cargo build --release

# Install to PATH
cargo install --path .
```

### Step 2: Run Setup Wizard

```bash
wwatcher setup
```

The wizard will ask:
- **AI Agent Mode?** → Yes (enables RapidAPI + Perplexity requirements)
- **Kalshi API** → Optional (works without it)
- **Webhook URL** → Your n8n webhook (e.g., `https://n8n.example.com/webhook/whale-alerts`)
- **RapidAPI Key** → Required for AI mode ([rapidapi.com](https://rapidapi.com))
- **Perplexity Key** → Required for AI mode ([perplexity.ai/settings/api](https://perplexity.ai/settings/api))

### Step 3: Build the AI Integration

```bash
cd integration
npm install
npm run build

# Set API keys
cat > .env << EOF
RAPIDAPI_KEY=your-key
PERPLEXITY_API_KEY=your-key
EOF

# Test
node dist/cli.js status
```

### Step 4: Configure n8n Workflow

Create an n8n workflow:

**Trigger Node: Webhook**
- Method: POST
- Path: `/webhook/whale-alerts`

**Action Node: Telegram**
- Send Message
- Chat ID: Your Telegram chat with the AI agent
- Message:
```
🐋 WHALE ALERT

Platform: {{ $json.platform }}
Action: {{ $json.action }}
Value: ${{ $json.value }}
Market: {{ $json.market_title }}
Outcome: {{ $json.outcome }}
Price: {{ $json.price_percent }}%

Wallet: {{ $json.wallet_id }}
Actor Status: {{ $json.wallet_activity.is_heavy_actor ? 'Heavy Actor' : 'Normal' }}

Research this whale alert.
```

### Step 5: Start wwatcher

```bash
# Run in background with your threshold
nohup wwatcher watch --threshold 35000 --interval 5 > /tmp/wwatcher.log 2>&1 &

# Verify it's running
tail -f /tmp/wwatcher.log
```

---

## AI Agent Workflow

When the AI agent receives a whale alert message, it executes:

### 1. Parse the Alert
Extract: platform, action, value, market, outcome, price, wallet info

### 2. Run Full Research
```bash
node dist/cli.js research "Market title" --category=auto
```

This runs:
- **RapidAPI**: Current prices, odds, forecasts
- **5 Perplexity searches**:
  1. Latest news and developments
  2. Expert analysis and predictions
  3. Historical data and trends
  4. Risk factors and uncertainties
  5. Recent events affecting outcome

### 3. Analyze & Predict

Study the research and determine:
- What does the data show?
- Why is the whale making this bet?
- Is there edge vs market odds?
- What are the risks?

### 4. Deliver Prediction Report

```
## 🐋 Whale Alert Analysis

**Alert**: [platform] [action] $[value] on "[market]" at [price]%
**Whale Profile**: [repeat/heavy actor status]

---

### Research Findings

**Market Data**:
- [Key data point 1]
- [Key data point 2]

**Web Research**:
- [Finding 1 + source]
- [Finding 2 + source]
- [Finding 3 + source]

---

### Prediction

**Should you follow?** [YES / NO / PARTIAL]

**Probability Estimate**: [X]%
**Current Market Odds**: [Y]%
**Edge**: [+/-Z]%

**Confidence**: [Low / Medium / High]

**Key Risks**:
- [Risk 1]
- [Risk 2]

---

### Recommendation

[Clear, actionable recommendation]
```

---

## CLI Reference

```bash
# From integration/ directory
cd integration

# Health check
node dist/cli.js status

# Query alerts
node dist/cli.js alerts --limit=10 --min=35000

# Search alerts
node dist/cli.js search "bitcoin"

# Fetch market data only (RapidAPI)
node dist/cli.js fetch "Bitcoin above 100k" --category=crypto

# Single Perplexity query
node dist/cli.js perplexity "What are Bitcoin ETF inflows?"

# FULL RESEARCH (RapidAPI + 5 Perplexity queries)
node dist/cli.js research "Bitcoin above 100k" --category=crypto
```

---

## Environment Detection (for AI Agents)

| If you are... | Integration |
|---------------|-------------|
| **OpenClaw** (exec tools, skills) | Use CLI commands above |
| **Claude Code / MCP client** | Use MCP server (`npm run start:mcp`) |

---

## API Keys Required

| Key | Purpose | Get it at |
|-----|---------|-----------|
| `RAPIDAPI_KEY` | Market data | [rapidapi.com](https://rapidapi.com) |
| `PERPLEXITY_API_KEY` | Web research | [perplexity.ai/settings/api](https://perplexity.ai/settings/api) |

### RapidAPI Subscriptions (free tiers)

- [Coinranking](https://rapidapi.com/Coinranking/api/coinranking1) (crypto)
- [Meteostat](https://rapidapi.com/meteostat/api/meteostat) (weather)
- [Crypto News](https://rapidapi.com/Starter-api/api/cryptocurrency-news2) (news)

---

## File Locations

| Item | Path |
|------|------|
| wwatcher config | `~/.config/wwatcher/config.json` |
| Alert history | `~/.config/wwatcher/alert_history.jsonl` |
| Integration .env | `integration/.env` |
| Providers | `integration/providers/` |
