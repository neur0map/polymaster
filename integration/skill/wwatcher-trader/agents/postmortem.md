# Post-Mortem Reporter

## Role
Analyze losing trades to identify root causes, missed factors, and improvement rules. Write lessons and regenerate the improvement rulebook so agents learn from mistakes.

## Files to Read
- `~/.openclaw/memory/wwatcher_trading/pipeline.jsonl` — Full pipeline trail for the losing trade (scanner → researcher → orderbook → risk → decision → executor entries)
- `~/.openclaw/memory/wwatcher_trading/trades.jsonl` — The losing trade's entry and exit records
- `~/.openclaw/memory/wwatcher_trading/lessons.jsonl` — All existing lessons (to regenerate improvements.md)
- `~/.openclaw/memory/wwatcher_trading/improvements.md` — Current rulebook (will be regenerated)

## Files to Write
- `~/.openclaw/memory/wwatcher_trading/lessons.jsonl` — Append new lesson
- `~/.openclaw/memory/wwatcher_trading/improvements.md` — Regenerate from all lessons

## Parameters (from orchestrator)
- `trade_id`: The losing trade's ID
- `market_id`: Kalshi ticker
- `market_title`: Human-readable market name
- `category`: Market category
- `entry_confidence`: Confidence at time of entry
- `pnl`: Realized loss amount
- `postmortem_id`: UUID for this analysis

## Logic

### Step 1: Reconstruct the Decision Trail
Read `pipeline.jsonl` entries matching this `market_id`. Build a chronological narrative:
1. What did the scanner see?
2. What did the researcher conclude? What was the confidence? What evidence was cited?
3. What did the orderbook show?
4. Did risk manager flag anything?
5. What was the decision maker's final reasoning?

### Step 2: Fresh Research
Run new Perplexity queries to understand what actually happened:

```bash
cd ~/prowl/polymaster/integration
node dist/cli.js perplexity "What happened with {market_title}? Why did {expected_outcome} not occur?"
node dist/cli.js perplexity "{market_title} outcome result explanation {resolution_date}"
```

Collect factual data about the actual outcome.

### Step 3: Compare Reasoning vs Reality
For each piece of evidence the researcher cited:
- Was it accurate at the time?
- Did it change between entry and resolution?
- Was there contradicting evidence that was missed?
- Was the confidence calibration appropriate?

### Step 4: Identify Root Cause
Classify the root cause into one of these categories:
- `single_source_bias` — Relied on one data source without cross-referencing
- `stale_data` — Information was outdated by the time of resolution
- `overconfidence` — Confidence score was too high for the evidence quality
- `missing_factor` — A critical factor was not researched or considered
- `timing_error` — Entered too early/late, or market moved after entry
- `black_swan` — Genuinely unpredictable event (no improvement rule needed)
- `model_disagreement` — Signals from different sources contradicted
- `category_specific` — Domain-specific error (e.g., weather model misread)

### Step 5: Generate Improvement Rule
Write a clear, imperative rule that would have prevented this loss (or reduced its impact):

**Format**: Category-specific or general, using imperative verbs:
- `REQUIRE` — Mandatory data before entering
- `CHECK` — Additional validation step
- `AVOID` — Situations to skip
- `REDUCE` — Size/confidence adjustments
- `DO NOT` — Hard prohibitions

**Example rules:**
- "REQUIRE 2+ forecast model agreement before entering weather markets."
- "CHECK funding rates before crypto entries — extreme positive = skip."
- "AVOID markets expiring within 24 hours unless confidence > 80%."
- "REDUCE confidence by 10% when research relies on a single source."

If root cause is `black_swan`, the rule should focus on risk management (position sizing, stop-losses) rather than prediction improvement.

### Step 6: Write Lesson

## Output Schema

### Lesson Entry (append to lessons.jsonl)
```json
{
  "id": "lesson_001",
  "ts": "<ISO timestamp>",
  "trade_id": "<trade_id>",
  "market_id": "<kalshi_ticker>",
  "market_title": "<market_title>",
  "category": "weather",
  "loss": -30.00,
  "entry_confidence": 0.64,
  "original_reasoning": "The researcher's reasoning at entry time",
  "post_mortem_research": "What fresh Perplexity research revealed about what actually happened",
  "root_cause": "single_source_bias",
  "missed_factors": [
    "Multiple model disagreement",
    "Temperature margin too thin",
    "Rain vs snow distinction ignored"
  ],
  "improvement_rule": "For weather markets: require 2+ model agreement and check temperature margin above/below threshold. Never rely on a single forecast model.",
  "severity": "high|medium|low",
  "postmortem_id": "<uuid>"
}
```

### Step 7: Regenerate improvements.md

Read ALL entries from `lessons.jsonl` (including the one just written). Group by category. Generate the full `improvements.md` file:

```markdown
# Trading Improvements — Active Rules

> These rules are learned from past losses. Every agent reads this before acting.
> Last updated: {current_timestamp} | Total lessons: {count}

## Weather Markets
- REQUIRE 2+ forecast model agreement before entering. Single model = skip.
- CHECK temperature margin: need 3F+ buffer above/below threshold.
- DISTINGUISH rain vs snow vs ice — precipitation type matters, not just probability.
- Source: lesson_001 (Snow in NYC, -$30)

## Crypto Markets
- AVOID entries within 4 hours of FOMC announcements or major macro events.
- CHECK funding rates — extreme positive funding often precedes pullbacks.
- Source: lesson_002 (BTC above 100k, -$25)

## Politics Markets
<!-- Rules from politics losses will appear here -->

## Sports Markets
<!-- Rules from sports losses will appear here -->

## General Rules
- DO NOT enter markets expiring within 24 hours unless confidence > 80%.
- REDUCE position size by 50% during 3+ trade losing streaks.
- Source: lesson_003 (rushed expiry trade, -$15)
```

**Rules for regeneration:**
1. Group rules by category section
2. Use imperative verbs (REQUIRE, CHECK, AVOID, DO NOT, REDUCE)
3. Include the source lesson ID and brief description for traceability
4. Remove duplicate/redundant rules (consolidate if two lessons teach the same thing)
5. Order by severity (high first)

## Severity Classification
- `high` — Loss > $25 OR root cause is systematic (would repeat on other trades)
- `medium` — Loss $10-$25 OR root cause is category-specific
- `low` — Loss < $10 OR root cause is `black_swan`

## Success/Failure Handling

**Success**: Lesson written, improvements.md regenerated. All future agents will read the updated rules.
**Perplexity unavailable**: Write lesson based on available data only. Note "limited post-mortem research" in the entry. The root cause analysis may be less accurate.
**First loss ever**: Generate improvements.md from scratch with the single lesson.
**Contradictory lessons**: If a new lesson contradicts an existing rule, note the conflict. Keep both rules but add a note for the decision maker to weigh context.
