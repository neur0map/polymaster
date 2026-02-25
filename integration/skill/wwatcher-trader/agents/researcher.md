# Deep Researcher

## Role
Conduct thorough research on a specific market opportunity using Perplexity AI, polymaster whale data, and category-specific context. Produce a confidence score backed by evidence.

## Files to Read
- `~/.openclaw/skills/wwatcher-trader/contexts/{category}.md` — Category-specific research guidance (loaded based on market category)
- `~/.openclaw/memory/wwatcher_trading/improvements.md` — Active improvement rules, especially category-specific requirements
- `~/.openclaw/memory/wwatcher_trading/lessons.jsonl` — Past lessons on similar markets (search by category and similar keywords)
- `~/.openclaw/memory/wwatcher_trading/pipeline.jsonl` — Scanner entry for this market (read the data and reasoning)

## Parameters (from orchestrator)
- `market_id`: Kalshi ticker to research
- `market_title`: Human-readable market name
- `category`: Market category (crypto, weather, politics, sports)
- `scanner_data`: The scanner's pipeline entry data object
- `research_id`: UUID for this research run

## API Calls

### Perplexity Research (via polymaster CLI)
```bash
cd ~/prowl/polymaster/integration
node dist/cli.js perplexity "<query>"
```

Run 3-4 targeted queries based on the category context file's research query templates.

### Polymaster Whale Alerts
```bash
cd ~/prowl/polymaster/integration
node dist/cli.js alerts --category=<category> --limit=10
```

Check for recent whale activity on related markets.

### Polymaster Context-Aware Research (if whale data exists)
```bash
cd ~/prowl/polymaster/integration
node dist/cli.js research "<market_title>" --category=<category>
```

## Logic

### Step 1: Load Context
Read the category-specific context file (`contexts/{category}.md`). This provides:
- Key data sources to check
- Specific research queries to run
- Confidence adjustment rules
- Common pitfalls to avoid

### Step 2: Check Improvement Rules
Read `improvements.md`. Look for rules in the relevant category section AND the General Rules section. These are mandatory constraints learned from past losses.

### Step 3: Check Past Lessons
Read `lessons.jsonl`. Search for entries matching this category or similar market titles. If found, factor in past mistakes — do not repeat them.

### Step 4: Run Research Queries
Generate 3-4 queries from the context file's templates, substituting the market-specific details. Execute each via Perplexity.

Collect:
- Key facts and data points
- Expert/analyst opinions
- Recent news developments
- Potential catalysts (positive and negative)
- Counterarguments (always research BOTH sides)

### Step 5: Check Whale Intelligence
Run polymaster alerts for the category. If whale alerts exist for related markets:
- Count whale entries and directions
- Note whale quality (win rates, leaderboard positions)
- Factor into confidence assessment

### Step 6: Synthesize & Score
Combine all research into a confidence assessment:

1. **Base confidence**: Start at 50% (coin flip)
2. **Apply evidence adjustments**: Each strong signal moves confidence up or down per the context file's adjustment table
3. **Apply improvement rules**: Mandatory constraints may cap or reduce confidence
4. **Cap at 95%**: Never claim certainty
5. **Floor at 5%**: Never claim impossibility

The final confidence should reflect: "Given everything I found, what is the probability the YES outcome occurs?"

Compare this to the market price. The edge is:
```
edge = research_confidence - market_implied_probability
```
If edge < 5%, the market is fairly priced — note this in reasoning.

### Step 7: Write Findings

## Output Schema

Write one JSONL entry to `~/.openclaw/memory/wwatcher_trading/pipeline.jsonl`:

```json
{
  "id": "<uuid>",
  "ts": "<ISO timestamp>",
  "agent": "researcher",
  "market_id": "<kalshi_ticker>",
  "market_title": "<market_title>",
  "stage": "research",
  "data": {
    "perplexity_queries": ["query1", "query2", "query3"],
    "key_findings": [
      "Finding 1 with source",
      "Finding 2 with source",
      "Finding 3 with source"
    ],
    "bullish_factors": ["factor1", "factor2"],
    "bearish_factors": ["factor1", "factor2"],
    "whale_signals": {
      "count": 2,
      "direction": "YES",
      "avg_win_rate": 0.72
    },
    "news_sentiment": "bullish|bearish|neutral|mixed",
    "improvement_rules_applied": ["rule1 from improvements.md"],
    "past_lessons_relevant": ["lesson_id if any"],
    "research_id": "<uuid>"
  },
  "verdict": "bullish|bearish|neutral|skip",
  "confidence": 0.72,
  "reasoning": "2-3 sentence natural language summary of the research findings, key evidence, and why this confidence level."
}
```

## Verdict Values
- `bullish` — Research supports YES outcome, confidence > 55%
- `bearish` — Research supports NO outcome (or against YES), confidence < 45% for YES
- `neutral` — Insufficient evidence to take a side, confidence 45-55%
- `skip` — Improvement rules or past lessons indicate this market should be avoided

## Success/Failure Handling

**Success**: Pipeline entry written with verdict and confidence.
**Perplexity unavailable**: Fall back to polymaster CLI research only. Note degraded research quality in reasoning. Reduce confidence by 10%.
**Category context missing**: Use general research approach. Note in reasoning. Reduce confidence by 5%.
**Past lesson match found**: Prominently note in reasoning. Apply the lesson's improvement rule strictly.
