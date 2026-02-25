# wwatcher-trader — Autonomous Kalshi Trading Agent

**Name**: wwatcher-trader
**Description**: Autonomous prediction market trading on Kalshi. Scans for mispriced opportunities, researches with AI, validates risk, executes trades, monitors positions, and learns from losses.
**Version**: 1.0.0

---

## Trigger

### On Install
Run guided setup: load `config/setup.md` and walk the user through Kalshi API credentials, category selection, risk limits, and verification.

### On Startup (after setup complete)
1. Read `~/.openclaw/memory/wwatcher_trading/config.json`
2. Verify `setup_complete: true`
3. Start **Scanner Loop** and **Monitor Loop** concurrently

### On User Message
Match against these patterns and route accordingly:

| Pattern | Action |
|---------|--------|
| "approve {trade_id}" | Spawn executor for the queued trade |
| "reject {trade_id}" | Mark as rejected in pipeline.jsonl |
| "show my portfolio" / "how are we doing" | Read portfolio.json, summarize |
| "what's pending" | List QUEUE-status trades from pipeline.jsonl |
| "show my lessons" / "what have we learned" | Read improvements.md, summarize |
| "show my config" / "show settings" | Read config.json, summarize |
| "pause trading" | Stop scanner loop, keep monitor running |
| "resume trading" | Restart scanner loop |
| "enable live trading" | Set config.dry_run = false, confirm with user |
| "enable dry run" | Set config.dry_run = true |
| "kill all positions" | Emergency exit — spawn executor to sell all open positions at market |
| Config update (natural language) | Parse intent, update config.json, confirm change |

### On Trade Resolution (Loss)
When a trade resolves with negative PnL → spawn post-mortem agent.

---

## Memory Paths

All runtime data lives in `~/.openclaw/memory/wwatcher_trading/`:

| File | Purpose | Who writes | Who reads |
|------|---------|------------|-----------|
| `config.json` | User configuration | Setup, user commands | All agents |
| `pipeline.jsonl` | Shared agent findings | All agents | Decision maker, orchestrator |
| `trades.jsonl` | Executed trade log | Executor | Portfolio, post-mortem |
| `portfolio.json` | Current portfolio state | Portfolio tracker | Scanner, risk, monitor, decision |
| `lessons.jsonl` | Post-mortem learnings | Post-mortem | Post-mortem (regeneration), researcher |
| `improvements.md` | Active improvement rules | Post-mortem | Scanner, researcher, decision |

### Initialization
On first setup, copy template files from the skill directory:
```
~/.openclaw/skills/wwatcher-trader/templates/config.json     → ~/.openclaw/memory/wwatcher_trading/config.json
~/.openclaw/skills/wwatcher-trader/templates/portfolio.json   → ~/.openclaw/memory/wwatcher_trading/portfolio.json
~/.openclaw/skills/wwatcher-trader/templates/improvements.md  → ~/.openclaw/memory/wwatcher_trading/improvements.md
~/.openclaw/skills/wwatcher-trader/templates/pipeline.jsonl   → ~/.openclaw/memory/wwatcher_trading/pipeline.jsonl
~/.openclaw/skills/wwatcher-trader/templates/trades.jsonl     → ~/.openclaw/memory/wwatcher_trading/trades.jsonl
~/.openclaw/skills/wwatcher-trader/templates/lessons.jsonl    → ~/.openclaw/memory/wwatcher_trading/lessons.jsonl
```

---

## Loops

### Scanner Loop

**Frequency**: Every `config.scanner.interval_minutes` minutes (default: 10)
**Active when**: `config.setup_complete == true` AND trading is not paused

```
REPEAT every {interval} minutes:
  1. Spawn scanner agent
     - Prompt: agents/scanner.md
     - Model guidance: cheap, no extended thinking (deterministic filtering)
     - Timeout: 60s
     - Context: config.json + portfolio.json + improvements.md

  2. Read pipeline.jsonl for new scanner entries (stage: "scan", verdict: "opportunity")

  3. FOR EACH opportunity:
     a. Spawn researcher + orderbook IN PARALLEL
        - Researcher:
          - Prompt: agents/researcher.md + contexts/{category}.md
          - Model guidance: moderate, some thinking (synthesis work)
          - Timeout: 120s
          - Context: improvements.md + lessons.jsonl + pipeline entry
        - Orderbook:
          - Prompt: agents/orderbook.md
          - Model guidance: cheap, no thinking (API call + math)
          - Timeout: 30s
          - Context: config.json + pipeline entry

     b. WAIT for both to complete

     c. Spawn risk manager
        - Prompt: agents/risk.md
        - Model guidance: cheap, no thinking (rule checking)
        - Timeout: 30s
        - Context: config.json + portfolio.json + pipeline entries

     d. WAIT for completion

     e. Spawn decision maker
        - Prompt: agents/decision.md
        - Model guidance: best available, extended thinking (high-stakes reasoning)
        - Timeout: 180s
        - Context: ALL pipeline entries for this market + improvements.md + config.json

     f. Read decision verdict:
        - EXECUTE → Spawn executor → On success, spawn portfolio tracker
        - QUEUE → Notify user with trade summary, await approval
        - SKIP → Log, continue to next opportunity
```

### Monitor Loop

**Frequency**: Every `config.monitor.interval_minutes` minutes (default: 3)
**Active when**: `config.setup_complete == true` (always runs, even when scanner is paused)

```
REPEAT every {interval} minutes:
  1. Spawn monitor agent
     - Prompt: agents/monitor.md
     - Model guidance: cheap, no thinking (price checks + comparisons)
     - Timeout: 30s
     - Context: portfolio.json + config.json

  2. Read pipeline.jsonl for monitor results:
     - verdict: "exit_triggered" → Spawn executor (sell) → Spawn portfolio tracker
       - If exit was a loss → Spawn post-mortem
     - verdict: "user_decision_needed" → Notify user, await instruction
     - verdict: "all_clear" → Continue
```

### Post-Mortem (event-driven, not a loop)

```
ON loss detected:
  1. Spawn post-mortem agent
     - Prompt: agents/postmortem.md
     - Model guidance: best available, extended thinking (deep analysis)
     - Timeout: 180s
     - Context: pipeline trail + trades.jsonl + lessons.jsonl + improvements.md

  2. Post-mortem writes lesson to lessons.jsonl and regenerates improvements.md

  3. All subsequent agent spawns will read the updated improvements.md
```

---

## Agent Roster

| Agent | File | Frequency | Model Guidance | Purpose |
|-------|------|-----------|----------------|---------|
| Scanner | `agents/scanner.md` | Every 10min | Cheap, no thinking | Find opportunities |
| Researcher | `agents/researcher.md` | Per opportunity | Moderate, some thinking | Deep research |
| Orderbook | `agents/orderbook.md` | Per opportunity | Cheap, no thinking | Liquidity check |
| Risk | `agents/risk.md` | Per opportunity | Cheap, no thinking | Limit enforcement |
| Decision | `agents/decision.md` | Per opportunity | Best, deep thinking | Final call |
| Executor | `agents/executor.md` | On EXECUTE/exit | Cheap, no thinking | Place trades |
| Monitor | `agents/monitor.md` | Every 3min | Cheap, no thinking | Watch positions |
| Portfolio | `agents/portfolio.md` | After each trade | Cheap, no thinking | Update stats |
| Post-mortem | `agents/postmortem.md` | On each loss | Best, deep thinking | Learn from mistakes |

### Category Contexts

Loaded as additional context for the researcher agent based on market category:

| Category | File | Key Focus |
|----------|------|-----------|
| Crypto | `contexts/crypto.md` | Funding rates, on-chain flows, macro calendar, ETF data |
| Weather | `contexts/weather.md` | Multi-model agreement, temp margins, precip type |
| Politics | `contexts/politics.md` | Polling averages, demographics, methodology |
| Sports | `contexts/sports.md` | Injury reports, line movement, sharp money, rest days |

New categories: create `contexts/{category}.md` and add to `config.categories`. No agent changes needed.

---

## Model Selection Guidance

OpenClaw decides which model and thinking level to use. These are guidelines, not rigid rules:

**Principles:**
- **CHEAP & FAST** for deterministic work — API calls, math, filtering, rule checking
- **MODERATE** for synthesis — combining multiple data sources, summarizing research
- **EXPENSIVE & DEEP** only for high-stakes reasoning — trade decisions, loss analysis
- **MATCH** frequency to cost — agents running every 3 minutes must be cheap
- **SCALE** thinking to uncertainty — clear decisions need no reasoning, ambiguous signals need deep thinking

**Adaptive upgrades:**
- Losing streak active → upgrade researcher to deeper thinking
- High-value trade (>$40) → upgrade decision maker
- System idle with budget → run deeper research passes
- Many opportunities simultaneously → use cheaper models to parallelize

---

## User Commands

### Portfolio & Status
- **"show my portfolio"** → Read portfolio.json, display balances, open positions, all-time stats
- **"how are we doing"** → Summarize all-time performance: win rate, total PnL, best/worst trades
- **"what's pending"** → List trades with QUEUE status from pipeline.jsonl

### Trade Management
- **"approve {trade_id}"** → Spawn executor for the queued trade
- **"reject {trade_id}"** → Mark as rejected in pipeline.jsonl, log reason
- **"kill all positions"** → Emergency: spawn executor to sell ALL open positions immediately

### Configuration
- **"show my config"** → Read config.json, display formatted summary
- **"raise my daily limit to $X"** → Update `risk_limits.daily_limit`
- **"add {category} to my categories"** → Append to `categories` array
- **"remove {category}"** → Remove from `categories` array
- **"set min multiplier to X"** → Update `risk_limits.min_payout_multiplier`
- **"only auto-execute above X% confidence"** → Update `execution_tiers.auto_execute.min_confidence`
- **"set max trade size to $X"** → Update `risk_limits.max_trade_size`

### Operational Control
- **"pause trading"** → Stop scanner loop. Monitor loop continues watching positions.
- **"resume trading"** → Restart scanner loop.
- **"enable live trading"** → Set `config.dry_run = false`. Confirm with user: "Live trading enabled. Real money will be used."
- **"enable dry run"** → Set `config.dry_run = true`. Confirm.

### Learning & Improvement
- **"show my lessons"** → Read improvements.md, display the active rulebook
- **"what have we learned"** → Summarize lessons.jsonl: count, categories, key rules
- **"run post-mortem on {trade_id}"** → Manually trigger post-mortem for any trade

---

## Safety Guardrails

1. **Dry run default**: `config.dry_run: true` on setup. No real trades until user explicitly says "enable live trading".
2. **Private key isolation**: PEM file path stored in config, never the key contents. Key is read only by the Node.js signing one-liner at execution time.
3. **Limit orders only**: Executor always uses limit orders, never market orders. Price protection built in.
4. **Daily spending cap**: Risk manager enforces `daily_limit` on every trade.
5. **Position cap**: Risk manager enforces `max_open_positions` limit.
6. **Losing streak protection**: Automatic size reduction after consecutive losses.
7. **Expiry protection**: Monitor closes profitable positions before expiry settlement.
8. **Self-improvement**: Post-mortem writes rules that ALL agents read. System gets smarter after each loss.

---

## Pipeline Data Contract

Every agent writes JSONL entries to `pipeline.jsonl` with this schema:

```json
{
  "id": "uuid",
  "ts": "ISO-8601 timestamp",
  "agent": "scanner|researcher|orderbook|risk|decision|executor|monitor|portfolio|postmortem",
  "market_id": "KALSHI-TICKER",
  "market_title": "Human readable name",
  "stage": "scan|research|orderbook|risk_check|decision|execution|monitor_check|monitor_exit|monitor_warning|portfolio_warning",
  "data": {},
  "verdict": "string",
  "confidence": 0.0,
  "reasoning": "Natural language explanation"
}
```

Agents tie their entries together via `market_id`. The decision maker reads ALL entries for a market to make the final call.

---

## Notification Events

When `config.notifications` flags are true:

| Event | Trigger | Content |
|-------|---------|---------|
| `on_trade_executed` | Executor fills an order | Market, side, size, price, confidence |
| `on_approval_needed` | Decision maker returns QUEUE | Market summary, confidence, recommended action |
| `on_stop_loss_hit` | Monitor triggers stop-loss exit | Market, loss amount, reasoning |
| `on_daily_summary` | End of trading day | Day's trades, P&L, portfolio snapshot |
