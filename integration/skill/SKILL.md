# wwatcher-ai — Whale Alert Research Agent

## Trigger

**When you see a message containing "🐋 WHALE ALERT" or "Research this whale alert":**
1. Parse the alert details from the message
2. Run the full research workflow below
3. Reply with a prediction report

---

## Quick Reference

```bash
cd /home/neur0map/polymaster-test/integration

# Full research (use this for whale alerts)
node dist/cli.js research "Market title from alert" --category=crypto

# Other commands
node dist/cli.js status                    # Health check
node dist/cli.js alerts --limit=5          # Recent alerts
node dist/cli.js fetch "query"             # RapidAPI only
node dist/cli.js perplexity "query"        # Single search
```

---

## Research Workflow

### Step 1: Parse the Alert

From the incoming message, extract:
- **Platform**: Polymarket or Kalshi
- **Action**: BUY/SELL, YES/NO
- **Value**: Dollar amount
- **Market**: Market title
- **Outcome**: What they're betting on
- **Price**: Current odds (%)
- **Wallet**: Actor status (repeat/heavy)

### Step 2: Run Research

```bash
node dist/cli.js research "Market title" --category=crypto
```

Categories: `crypto`, `sports`, `weather`, `politics`, `news`

This fetches:
- RapidAPI market data
- 5 Perplexity web searches

### Step 3: Deliver Prediction

```
## 🐋 Whale Alert Analysis

**Alert**: [platform] [action] $[value] on "[market]" at [price]%
**Whale**: [wallet status]

---

### Research Findings

**Market Data**:
- [Current price/odds]
- [Trend/momentum]

**Web Research**:
- [Key finding 1]
- [Key finding 2]
- [Key finding 3]

---

### Prediction

**Follow the whale?** [YES / NO / PARTIAL]

**My Estimate**: [X]%
**Market Odds**: [Y]%
**Edge**: [+/-Z]%

**Confidence**: [Low/Medium/High]

**Risks**:
- [Risk 1]
- [Risk 2]

---

### Recommendation

[One clear sentence]
```

---

## Category Guide

| Category | RapidAPI Data | Research Focus |
|----------|---------------|----------------|
| crypto | Coinranking prices | ETF flows, whale activity, technicals |
| sports | Game data | Injuries, odds movement, matchups |
| weather | Meteostat forecast | Model confidence, patterns |
| politics | — | Polls, demographics, news |

---

## API Keys

Both required in `integration/.env`:
```
RAPIDAPI_KEY=xxx
PERPLEXITY_API_KEY=xxx
```
