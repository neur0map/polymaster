# Prowl Bot — Prediction Market Trading Bot Design

Date: 2026-02-25

## Overview

A conversational Telegram bot with a multi-agent prediction market team. Combines:
- **ZeroClaw's** Telegram integration, gateway, soul/memory/skills system (extracted, adapted to Rust)
- **CrewAI's** hierarchical multi-agent orchestration (translated to Rust)
- **wwatcher's** whale alert data + scoring as the signal source
- **Continual learning** via Self-SFT and GRPO on resolved market outcomes

The bot lives as a new binary crate inside the polymaster workspace.

### Mental Model

```
You (CEO) ←→ Telegram ←→ Manager Bot (personality, memory, skills)
                              │
                    ┌─────────┼─────────┐
                    ▼         ▼         ▼
                Researcher  Analyst    Risk
                soul.md     soul.md   soul.md
                skills/     skills/   skills/
                memory/     memory/   memory/
                    │         │         │
                    └─────────┼─────────┘
                              ▼
                      Manager synthesizes
                      presents to you
                              │
                      You: "go ahead" / "pass"
                              │
                              ▼
                          Executor
                          soul.md
                          skills/
                          memory/
                              │
                    ┌─────────┴─────────┐
                    ▼                   ▼
                 Kalshi            Polymarket
```

You talk to the manager. The manager delegates to the team. The team analyzes. The manager synthesizes and presents a decision with reasoning. You approve or reject. The executor places the order.

---

## Infrastructure

### Split Architecture: Server + Mac

The bot runs on two machines with different roles:

```
┌─────────────────────────────────────┐  ┌──────────────────────────────────┐
│  SERVER (always on, no GPU)          │  │  MAC (intermittent, Apple Silicon)│
│                                      │  │                                  │
│  • prowl-bot daemon                  │  │  • Training pipeline             │
│    - Telegram polling                │  │    - MLX LoRA fine-tuning (SFT)  │
│    - Gateway (webhook receiver)      │  │    - GRPO via Unsloth (or cloud) │
│    - Cloud LLM calls (OpenRouter)    │  │    - GGUF export → Ollama        │
│  • wwatcher (whale monitoring)       │  │                                  │
│  • Data collection (predictions.db)  │  │  • Optional local inference      │
│  • Market resolution tracker         │  │    - Ollama (embeddings, cheap)  │
│                                      │  │    - ONNX models (sentiment, etc)│
│  Runs 24/7, cloud-only inference     │  │                                  │
│  ~168 whale alerts/day               │  │  Run when available, portable    │
└─────────────────────────────────────┘  └──────────────────────────────────┘
```

**Server mode (default):** All agent reasoning via cloud (OpenRouter). No local model dependency. Embeddings via OpenRouter embedding endpoint or a lightweight cloud option. The bot is always available.

**Mac mode (training):** Clone repo, copy `data/` folder from server, run training, export GGUF, push trained model back. Training is decoupled from the live bot.

**Portable data directory:**
```
bot/data/
├── brain.db              # agent memories (SQLite)
├── predictions.db        # collected predictions + outcomes (SQLite)
├── training/
│   ├── sft/
│   │   ├── train.jsonl   # generated from resolved predictions
│   │   ├── valid.jsonl
│   │   └── test.jsonl
│   └── grpo/
│       └── prompts.jsonl # prompts + outcomes for RL training
└── models/
    └── prowl-v1.gguf     # fine-tuned model (after training)
```

Copy `data/` to Mac → train → copy trained model back. Or use git LFS / rsync.

---

## Architecture

### Agent Context Model

Every LLM call is stateless. The illusion of continuity comes from what we inject into the context window each time:

```
┌─────────────────────────────────────────────────────────┐
│  System Prompt (rebuilt every call)                       │
│                                                           │
│  1. soul.md      → "who I am" (personality, role)        │
│  2. skills/      → "what I know how to do" (instructions)│
│  3. core memory  → "what I always know" (persistent)     │
│  4. recalled     → "what's relevant now" (hybrid search) │
│     memories                                              │
│  5. tools        → "what I can call" (function manifest) │
│  6. task         → "what to do now" (from manager)       │
└─────────────────────────────────────────────────────────┘
```

### Three-Runtime Model Layer

```
┌─────────────────────────────────────────────────────────────┐
│                     Provider Registry                        │
│                                                              │
│  ┌─────────────┐  ┌──────────────┐  ┌───────────────────┐ │
│  │ OpenRouter   │  │   Ollama     │  │  ONNX Runtime     │ │
│  │ (cloud)      │  │  (local LLM) │  │  (local ML)       │ │
│  │              │  │              │  │                    │ │
│  │ • Claude     │  │ • Llama 3.2  │  │ • FinBERT         │ │
│  │ • GPT-4o    │  │ • Mistral    │  │   (sentiment)      │ │
│  │ • Gemini    │  │ • Qwen       │  │ • bge-reranker     │ │
│  │ • DeepSeek  │  │ • Phi-4      │  │   (memory search)  │ │
│  │              │  │ • nomic-embed│  │ • chronos          │ │
│  │              │  │   (embed)    │  │   (time series)    │ │
│  │              │  │ • prowl-v1   │  │                    │ │
│  │              │  │   (fine-tuned│  │                    │ │
│  └──────┬──────┘  └──────┬───────┘  └────────┬──────────┘ │
│         │                │                    │             │
│         └────────────────┼────────────────────┘             │
│                          ▼                                   │
│                   Unified Router                             │
│            (picks runtime by model tag)                      │
└─────────────────────────────────────────────────────────────┘
```

**Cloud (OpenRouter):** Smart reasoning — manager orchestration, researcher analysis, risk assessment. Always available. This is the primary runtime on the server.

**Local LLM (Ollama/llama.cpp):** Cheap tasks — embedding generation, context compression, learning extraction, memory hygiene. Cost: $0. Available when Mac is online, or via cloud fallback.

**Local ML (ONNX Runtime):** Specialized pretrained models — FinBERT sentiment classification, BGE reranker for memory search, Chronos time series prediction. Rust-native via `ort` crate, no Python dependency. Optional — bot works without them.

### Cloud Fallback Strategy

When local models are unavailable (Mac offline), the bot degrades gracefully:

| Task | With Mac | Without Mac (server-only) |
|------|----------|--------------------------|
| Agent reasoning | Cloud | Cloud (same) |
| Embeddings | Ollama (nomic-embed) | OpenRouter embed endpoint |
| Compression | Ollama (llama3.2) | Skip — use larger context window budget |
| Reranking | ONNX (bge-reranker) | Skip — use hybrid search only |
| Sentiment | ONNX (FinBERT) | Skip — LLM does sentiment in-context |
| Memory hygiene | Ollama | Defer until Mac is online |

The bot is fully functional cloud-only. Local models are performance/cost optimizations, not requirements.

### Local vs Cloud Model Responsibilities

| Task | Runtime | Cost |
|------|---------|------|
| Manager orchestration | Cloud (Claude/GPT) | Per-token |
| Researcher deep analysis | Cloud | Per-token |
| Analyst pattern interpretation | Cloud | Per-token |
| Risk assessment | Cloud | Per-token |
| Embedding generation | Local (Ollama) or Cloud fallback | $0 or minimal |
| Context compression | Local (Ollama) | $0 |
| Learning extraction after tasks | Local (Ollama) | $0 |
| Memory hygiene (dedup, consolidate) | Local (Ollama) | $0 |
| Core memory promotion decisions | Local (Ollama) | $0 |
| Sentiment analysis | Local (ONNX) | $0 |
| Memory reranking | Local (ONNX) | $0 |
| Time series prediction | Local (ONNX) | $0 |

---

## Agent System

### Agent Struct

Every agent (manager, researcher, analyst, risk, executor) is the same struct configured differently:

```rust
struct Agent {
    id: AgentId,              // "manager", "researcher", etc.
    soul: String,             // loaded from souls/{id}.md
    skills: Vec<Skill>,       // loaded from skills/{id}/
    memory: AgentMemory,      // partition in brain.db filtered by agent_id
    provider: Box<dyn Provider>,
    model: String,            // orchestrator picks this per spawn
    temperature: f32,
    max_iterations: u8,       // tool-call loop cap
}
```

### Execution Loop

On every call, the agent assembles its context and runs an agentic tool-calling loop:

1. Build system prompt: soul + skills + core memories + recalled memories
2. Add task from manager as user message
3. Call LLM
4. If LLM returns tool calls → execute tools, feed results back, loop
5. If LLM returns text → done, save learnings to memory
6. If LLM returns escalation → kick back to manager
7. If max iterations reached → return partial result

### Manager's Special Power: Delegation

The manager gets a `delegate` tool that other agents don't. It spawns subagents as concurrent tokio tasks:

```rust
delegate({
    agent: "researcher",
    task: "Gather context on BTC 100k market",
    model: "anthropic/claude-sonnet-4-5",
    thinking: "high"
})
```

The manager decides which model and thinking power per subagent based on task complexity.

### Communication Pattern: Fire-and-Collect + Escalation

Default: manager sends task to subagent, waits for final answer. One round trip.

Exception: subagent can escalate back to manager if stuck or confidence is low. Manager decides whether to provide more context, adjust the task, or reassign.

### Team Composition

| Agent | Role | Key Capabilities |
|-------|------|-----------------|
| **Manager** | Orchestrator | Talks to you, delegates tasks, synthesizes results, manages approval flow |
| **Researcher** | Data gatherer | Web search, market data, news, sentiment analysis (ONNX) |
| **Analyst** | Signal interpreter | Reads wwatcher.db whale history, scores alerts, detects patterns, time series (ONNX) |
| **Risk** | Position evaluator | Position sizing, bankroll management, liquidity analysis, downside scenarios |
| **Executor** | Trade placer | Kalshi API orders, Polymarket CLOB orders, position tracking, P&L |

---

## Memory Architecture

### The Problem

Cloud LLMs are stateless. They have a fixed context window. When context gets too long, you must truncate — which means forgetting. Local models have smaller windows but no API costs.

### The Solution: Three Tiers of Memory

#### Tier 1: Working Memory (in-conversation)

Message history during a single agent execution. Grows with every tool call.

When working memory hits 60% of the context window limit, older messages are compressed into a summary by a **local model** (Ollama). On server-only mode, skip compression and use a larger context budget instead.

```
[soul + skills + memories]  →  [summary of earlier work]  →  [recent messages]
     fixed (~2.5k tok)          compressed (~500 tok)         live (~5k-30k tok)
```

#### Tier 2: Episodic Memory (cross-conversation, per agent)

After each task completes, a **local model** extracts key learnings and saves them to SQLite with embeddings. Next time the agent is called, relevant memories are recalled via hybrid search.

Examples:
- "Whale 0x742d has bought BTC YES 3 times this week — consistent bull"
- "Kalshi BTC markets have thin liquidity after midnight UTC"

#### Tier 3: Core Memory (persistent, always loaded)

Things that should never be forgotten. Your preferences, accumulated wisdom, proven patterns. Stored in the `core` category, always injected (not search-dependent, up to 20 entries).

Core memories are **promoted** — when an episodic memory is recalled 3+ times across different sessions, it upgrades to core. The local model also extracts obvious preferences from conversation:

```
You: "Never put more than $500 on a single trade"
Manager saves → core memory: user_max_position = $500
```

### Memory Recall Pipeline

```
Query: "BTC whale behavior this week"
  ↓
Vector Search (Ollama nomic-embed-text or cloud embed) → 20 candidates
  ↓
BM25 Keyword Search (SQLite FTS5) → 20 candidates
  ↓
Hybrid Merge (0.7 vector / 0.3 keyword) → top 20
  ↓
ONNX Reranker (bge-reranker-v2, if available) → top 5
  ↓
Injected into agent context (~1k tokens)
```

### Memory Storage

All agents share one `brain.db`, partitioned by `agent_id`:

```sql
CREATE TABLE memories (
    id INTEGER PRIMARY KEY,
    agent_id TEXT NOT NULL,
    key TEXT NOT NULL,
    content TEXT NOT NULL,
    category TEXT NOT NULL,  -- "core", "conversation", "learning"
    embedding BLOB,
    created_at TEXT,
    updated_at TEXT
);

CREATE VIRTUAL TABLE memories_fts USING fts5(key, content);
```

### Memory Hygiene (Background, Local Model)

Runs periodically (daily or on startup, deferred if local models unavailable):

- **Deduplication** — merge near-duplicate memories (cosine similarity > 0.95)
- **Consolidation** — repeated observations become single memories with count
- **Decay** — episodic memories older than 30 days with no recalls get archived
- **Promotion** — frequently recalled episodic → core
- **Snapshot** — export core memories to `MEMORY_SNAPSHOT.md` for disaster recovery

---

## Soul & Skills System

Extracted from zeroclaw's dual-identity architecture.

### Soul (personality)

Each agent has a `souls/{agent_id}.md` file defining personality, role, voice, behavior. Loaded once per execution, injected at the top of the system prompt.

```
bot/souls/
├── manager.md      # delegation style, how to present to CEO, synthesis approach
├── researcher.md   # research methodology, source evaluation, thoroughness
├── analyst.md      # analysis framework, pattern recognition, scoring interpretation
├── risk.md         # risk philosophy, position sizing principles, conservative vs aggressive
└── executor.md     # execution discipline, order verification, confirmation protocols
```

### Skills (capabilities)

Per-agent skill directories. Each agent only loads skills from its own directory. Skills are SKILL.md files (instruction prompts) or SKILL.toml files (structured with optional tool definitions). Injected into system prompt after soul.

```
bot/skills/
├── shared/                    # cross-agent skills
├── manager/
│   └── delegation/
│       └── SKILL.md           # how to delegate, synthesize, present decisions
├── researcher/
│   ├── market-signals/
│   │   └── SKILL.md           # reading market microstructure
│   └── whale-tracking/
│       └── SKILL.md           # interpreting whale behavior patterns
├── analyst/
│   ├── alert-scoring/
│   │   └── SKILL.md           # using wwatcher scoring system
│   └── pattern-detection/
│       └── SKILL.md           # spotting recurring whale patterns
├── risk/
│   ├── position-sizing/
│   │   └── SKILL.md           # Kelly criterion, bankroll management
│   └── market-risk/
│       └── SKILL.md           # liquidity risk, spread analysis
└── executor/
    ├── kalshi-trading/
    │   └── SKILL.md           # Kalshi API order placement
    └── polymarket-trading/
        └── SKILL.md           # Polymarket CLOB order placement
```

Two injection modes:
- **Full** (default): complete skill instructions inlined in system prompt
- **Compact**: name/description/location only, agent reads file on demand

Security audit on load: blocks symlinks, script files, oversized files, path traversal.

---

## Telegram & Gateway

Two entry points into the bot. Extracted from zeroclaw.

### Telegram (bidirectional conversation)

Full zeroclaw-grade Telegram support:

| Feature | Description |
|---------|-------------|
| Long polling | `getUpdates` loop, no webhooks needed |
| Text messaging | Send/receive with 4096-char smart splitting |
| Media support | Photos, documents, voice messages |
| Emoji reactions | Random acknowledgment on message receive |
| Streaming drafts | Edit message in-place as response generates |
| Allowlist | `allowed_users` numeric ID check |
| Mention-only mode | Require @botname in group chats |
| /bind command | Self-authentication for new users |

### Gateway (HTTP webhook receiver)

Axum HTTP server for receiving wwatcher alerts and external integrations:

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/health` | GET | Health check (public) |
| `/pair` | POST | 6-digit pairing code → bearer token |
| `/webhook` | POST | Receive wwatcher alert JSON |

Security: pairing auth, rate limiting (sliding window per-IP), HMAC webhook secret (optional).

### Two Conversation Modes

**Mode 1: You're chatting** (Telegram message)

You send a message → Manager receives it as conversation → Manager decides if it needs the team or can answer directly → If team needed, spawns subagents → Synthesizes and replies.

**Mode 2: Alert comes in** (webhook from wwatcher)

wwatcher fires webhook → Manager auto-spawns full team (Researcher + Analyst + Risk, concurrent) → Collects results → Sends synthesized report to Telegram with approve/pass/adjust buttons → You approve → Executor places order → Confirmation in Telegram.

---

## Data Collection & Market Resolution

### The Data Pipeline

Every analysis the bot produces is logged automatically. This serves two purposes:
1. Track the bot's predictions against actual outcomes (performance measurement)
2. Build training data for continual learning (SFT and GRPO)

### Predictions Database

Separate from brain.db — this is the training data pipeline:

```sql
-- predictions.db

CREATE TABLE predictions (
    id INTEGER PRIMARY KEY,
    market_id TEXT NOT NULL,            -- "polymarket_0x..." or "kalshi_..."
    platform TEXT NOT NULL,             -- "polymarket" or "kalshi"
    question TEXT NOT NULL,             -- market title / question
    prediction_date TEXT NOT NULL,      -- when the bot made the prediction
    close_date TEXT,                    -- when market is scheduled to close
    market_price REAL,                  -- market odds at prediction time
    predicted_prob REAL,                -- bot's predicted probability
    analysis TEXT,                      -- full reasoning chain (<think>...</think>)
    agent TEXT,                         -- which agent produced this
    model TEXT,                         -- which LLM model was used
    alert_id TEXT,                      -- link to whale alert that triggered this
    version TEXT NOT NULL               -- model/bot version
);

CREATE TABLE outcomes (
    id INTEGER PRIMARY KEY,
    market_id TEXT NOT NULL UNIQUE,
    resolution INTEGER NOT NULL,        -- 1 = YES, 0 = NO
    resolved_at TEXT NOT NULL,
    brier_score REAL,                   -- computed: -(predicted_prob - resolution)^2
    profit_loss REAL                    -- if trade was placed
);

CREATE TABLE training_pairs (
    id INTEGER PRIMARY KEY,
    prediction_id INTEGER REFERENCES predictions(id),
    outcome_id INTEGER REFERENCES outcomes(id),
    split TEXT DEFAULT 'train',         -- "train", "valid", "test"
    exported INTEGER DEFAULT 0,         -- 1 = already exported to JSONL
    quality_score REAL                  -- Brier score, lower = better
);
```

### Market Resolution Tracker

A background task that periodically checks if markets from past predictions have resolved:

```
Every 6 hours (configurable):
  1. Query predictions.db for unresolved market_ids
  2. Check Polymarket API: GET /markets/{condition_id}
  3. Check Kalshi API: GET /markets/{ticker}
  4. If resolved: INSERT into outcomes, compute Brier score
  5. If training_pairs count crosses threshold → flag for export
```

This runs on the server alongside the bot. No GPU needed — just HTTP calls to public APIs.

### Current Data Inventory

As of 2026-02-25, wwatcher has collected:
- **~3,150 whale alerts** over 17 days (~168/day)
  - 2,864 in live SQLite (wwatcher.db), 286 in archived JSONL
  - 2,571 WHALE_ENTRY / 293 WHALE_EXIT
  - 2,314 Polymarket / 550 Kalshi
  - 90 unique wallets tracked
- These are raw whale trades, not predictions — they link to markets that may have already resolved
- **First action:** scan all market_ids from existing alerts, check resolution status, build initial outcomes table

### Training Data Export

When enough resolved predictions accumulate, export to JSONL for training:

```bash
prowl-bot export-training --format sft --output data/training/sft/
prowl-bot export-training --format grpo --output data/training/grpo/
```

**SFT format** (MLX chat format):
```jsonl
{"messages": [{"role": "system", "content": "You are a prediction market analyst..."}, {"role": "user", "content": "Market: Will BTC reach 100k? Price: 0.65. Headlines: [...]"}, {"role": "assistant", "content": "<think>Based on whale activity...</think>\n<prediction>0.72</prediction>"}]}
```

**GRPO format** (prompts + outcomes for reward computation):
```jsonl
{"prompt": "Market: Will BTC reach 100k? Price: 0.65. Headlines: [...]", "outcome": 1}
```

---

## Continual Learning

### Reference: "Outcome-based RL to Predict the Future"

[Turtel et al., TMLR 2025](https://arxiv.org/abs/2505.17989) — trained a 14B model (DeepSeek-R1-Distill-Qwen-14B) with modified GRPO on Polymarket data. Matched o1 accuracy, beat it on calibration, ~10% ROI in trading simulation. Key techniques we adopt:

1. **Modified GRPO**: Remove std_dev division from advantage calculation
   - Standard: `A = (reward - mean) / std_dev`
   - Modified: `A = reward - mean`
   - Preserves raw magnitude of large forecast errors

2. **Brier score reward**: `R = -(predicted_prob - outcome)^2`
   - Range: [-1.0, 0.0] where 0.0 = perfect prediction
   - Training penalty: -1.0 for unparseable output
   - Eval penalty: -0.25 for malformed output

3. **Guardrails**: Non-English detection, gibberish filtering, reasoning block requirement, character truncation, format validation

4. **Single-pass chronological training**: Each question seen once, in time order — prevents overfitting to stale patterns

5. **Median prediction sampling**: 7 forward passes at inference, take median — ensemble without extra models

**Their dataset:** 10k real Polymarket questions + 100k synthetic. Test set (1,265 questions) available at [HuggingFace](https://huggingface.co/datasets/LightningRodLabs/outcome-rl-test-dataset).

**Their hardware:** 8x H100 GPUs, ~3 days per run (for 14B model). We'll use smaller models (7B) on Mac with MLX/Unsloth — much less compute needed.

### Training Strategy

Two complementary approaches, run on Mac when available:

#### Self-SFT (Supervised Fine-Tuning)

**When:** 500+ resolved predictions accumulated
**Where:** Mac (Apple Silicon) via MLX
**What:** LoRA fine-tune a 7B model on the bot's best predictions

```bash
# On Mac, after copying data/ from server
cd bot
mlx_lm.lora \
  --model Qwen/Qwen2.5-7B \
  --train \
  --data data/training/sft/ \
  --iters 400 \
  --batch-size 1 \
  --num-layers 4 \
  --grad-checkpoint

# Fuse adapters into base model
mlx_lm.fuse --model Qwen/Qwen2.5-7B --adapter-path adapters --save-path fused/

# Convert to GGUF for Ollama
python llama.cpp/convert_hf_to_gguf.py fused/ --outfile data/models/prowl-v1.gguf --outtype q4_k_m

# Import into Ollama
ollama create prowl-v1 -f Modelfile
```

**Data filtering:** Keep only predictions where Brier score < 0.25. Quality > quantity.

**Preventing catastrophic forgetting:**
- LoRA (only modifies adapter matrices, base weights frozen)
- Mix 15% general-purpose data into training set
- Low learning rate (1e-5 to 5e-6)
- Short training (200-600 iterations)
- Evaluate on held-out general benchmark after each cycle

#### GRPO (Reinforcement Learning with Verifiable Rewards)

**When:** 2,000+ resolved predictions accumulated
**Where:** Mac with Unsloth (if GPU sufficient) or rented cloud GPU ($1-2/hr on vast.ai/runpod)
**What:** RL fine-tune for calibrated probability estimation

```python
# Using Unsloth + TRL GRPOTrainer
from unsloth import FastLanguageModel
from trl import GRPOConfig, GRPOTrainer

model, tokenizer = FastLanguageModel.from_pretrained("Qwen/Qwen2.5-7B", load_in_4bit=True)
model = FastLanguageModel.get_peft_model(model, r=16, target_modules=["q_proj", "v_proj"])

def brier_reward(completions, outcomes):
    """Modified reward without std_dev normalization"""
    rewards = []
    for completion, outcome in zip(completions, outcomes):
        prob = extract_probability(completion)
        if prob is None:
            rewards.append(-1.0)  # unparseable penalty
        else:
            rewards.append(-((prob - outcome) ** 2))
    return rewards

config = GRPOConfig(
    learning_rate=1e-6,
    num_generations=4,        # group size
    max_completion_length=512,
    per_device_train_batch_size=1,
    gradient_accumulation_steps=4,
)

trainer = GRPOTrainer(model=model, config=config, reward_funcs=[brier_reward], ...)
trainer.train()

# Export to GGUF
model.save_pretrained_gguf("prowl-grpo-v1", tokenizer, quantization_method="q4_k_m")
```

**Key modification from paper:** Remove std_dev division in advantage calculation. Unsloth/TRL support custom advantage computation.

#### Training Cadence

| Data Volume | SFT Cadence | GRPO Cadence |
|-------------|-------------|--------------|
| < 500 resolved | Not yet | Not yet |
| 500 - 2,000 | Monthly on Mac | Not yet |
| 2,000 - 5,000 | Bi-weekly | Monthly (cloud GPU) |
| 5,000+ | Weekly | Bi-weekly |

**Critical:** Markets must resolve before data is usable. Short-duration markets (1-7 days) give faster feedback. The bot should prefer analyzing short-horizon markets when building training data.

#### The Self-Improving Loop

```
┌──────────────────────────────────────────────────────────┐
│                  Continual Learning Loop                   │
│                                                            │
│  1. Bot analyzes whale alerts (cloud LLM)                 │
│     └→ Stores prediction + probability in predictions.db  │
│                                                            │
│  2. Resolution tracker checks market outcomes (background)│
│     └→ Computes Brier score, stores in outcomes table     │
│                                                            │
│  3. When threshold hit: export training JSONL              │
│     └→ SFT: filter best predictions (Brier < 0.25)       │
│     └→ GRPO: all predictions with outcomes                │
│                                                            │
│  4. Copy data/ to Mac, run training                       │
│     └→ MLX LoRA for SFT (local, free)                    │
│     └→ Unsloth for GRPO (local or cloud GPU)             │
│                                                            │
│  5. Export GGUF → Ollama                                  │
│     └→ Fine-tuned model replaces generic for some tasks  │
│     └→ Analyst uses prowl-v2 instead of base Qwen        │
│                                                            │
│  6. Evaluate on held-out recent markets                   │
│     └→ If Brier improved → deploy new version            │
│     └→ If degraded → rollback, adjust filtering          │
│                                                            │
│  Repeat. Each cycle the model gets better at YOUR markets.│
└──────────────────────────────────────────────────────────┘
```

Over time, the fine-tuned local model can handle more tasks that currently need cloud — reducing costs and latency. The analyst agent could eventually run on `prowl-v3` locally instead of Claude, for routine whale alert analysis.

---

## Crate Structure

```
polymaster/
├── Cargo.toml              # workspace root
├── src/                    # wwatcher binary (existing, unchanged)
├── integration/            # scoring MCP server (existing, unchanged)
└── bot/
    ├── Cargo.toml
    ├── config.example.toml
    ├── souls/
    │   ├── manager.md
    │   ├── researcher.md
    │   ├── analyst.md
    │   ├── risk.md
    │   └── executor.md
    ├── skills/
    │   ├── shared/
    │   ├── manager/
    │   ├── researcher/
    │   ├── analyst/
    │   ├── risk/
    │   └── executor/
    ├── data/                # portable data directory
    │   ├── brain.db
    │   ├── predictions.db
    │   ├── training/
    │   └── models/          # ONNX + fine-tuned GGUF (gitignored)
    └── src/
        ├── main.rs
        ├── config.rs
        ├── daemon.rs        # starts telegram + gateway + background tasks
        │
        ├── telegram/
        │   ├── mod.rs       # TelegramChannel, polling loop
        │   ├── parse.rs     # message parsing
        │   ├── send.rs      # send, split, reactions, streaming
        │   └── auth.rs      # allowlist, /bind
        │
        ├── gateway/
        │   ├── mod.rs       # Axum server
        │   ├── api.rs       # endpoints
        │   └── rate_limit.rs
        │
        ├── agents/
        │   ├── mod.rs       # Agent struct, AgentResult, spawn
        │   ├── manager.rs
        │   ├── researcher.rs
        │   ├── analyst.rs
        │   ├── risk.rs
        │   └── executor.rs
        │
        ├── providers/
        │   ├── mod.rs       # Provider trait
        │   ├── openrouter.rs
        │   └── ollama.rs
        │
        ├── memory/
        │   ├── mod.rs       # Memory trait
        │   ├── sqlite.rs    # brain.db
        │   ├── recall.rs    # hybrid search + reranker
        │   ├── compress.rs  # working memory compression
        │   └── hygiene.rs
        │
        ├── skills/
        │   ├── mod.rs       # Skill struct, loader
        │   └── audit.rs
        │
        ├── runtime/
        │   ├── mod.rs       # ModelRuntime trait, router
        │   └── onnx.rs      # ONNX wrapper
        │
        ├── markets/
        │   ├── mod.rs       # Market trait
        │   ├── kalshi.rs    # auth, orders, positions
        │   └── polymarket.rs # wallet, signing, orders
        │
        ├── collector/
        │   ├── mod.rs       # prediction logging
        │   ├── resolver.rs  # market resolution tracker (background)
        │   └── exporter.rs  # training data JSONL export
        │
        └── tools/
            └── mod.rs       # Tool trait, registry
```

### Key Dependencies

```toml
# Async / web
tokio = { version = "1", features = ["full"] }
axum = "0.8"
reqwest = { version = "0.12", features = ["json", "rustls-tls"], default-features = false }

# Data
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
rusqlite = { version = "0.32", features = ["bundled", "vtab"] }

# ML (optional, feature-gated)
ort = { version = "2", features = ["load-dynamic"], optional = true }
tokenizers = { version = "0.20", optional = true }

# Utilities
chrono = "0.4"
tracing = "0.1"
tracing-subscriber = "0.3"

[features]
default = []
local-ml = ["ort", "tokenizers"]  # enable ONNX models
```

Bot reads wwatcher's `wwatcher.db` directly via rusqlite (read-only). No shared library crate needed.

---

## Configuration

Single TOML file at `~/.config/prowl-bot/config.toml`.

```toml
[telegram]
bot_token = "123456:ABC-DEF..."
allowed_users = ["987654321"]
mention_only = false
draft_updates = true
reaction_enabled = true

[gateway]
bind = "127.0.0.1:42617"
pairing_enabled = true

[providers.cloud]
kind = "openrouter"
api_key = "sk-or-..."

[providers.local]
kind = "ollama"
base_url = "http://localhost:11434"
# If Ollama unavailable, cloud is used as fallback for embeddings

[memory]
db_path = "~/.config/prowl-bot/data/brain.db"
embedder = "local"                    # "local" (ollama) or "cloud" (openrouter)
embed_model = "nomic-embed-text"
compressor = "local"
compressor_model = "llama3.2"
reranker = "onnx"                     # optional, skipped if unavailable
reranker_model = "data/models/bge-reranker-v2.onnx"
recall_limit = 5
min_relevance = 0.4
hygiene_interval_hours = 24

[skills]
prompt_mode = "full"                  # "full" or "compact"

[agents.manager]
provider = "cloud"
model = "anthropic/claude-sonnet-4-5"
temperature = 0.7
max_iterations = 15

[agents.researcher]
provider = "cloud"
model = "anthropic/claude-sonnet-4-5"
temperature = 0.5
max_iterations = 10

[agents.analyst]
provider = "cloud"
model = "anthropic/claude-sonnet-4-5"
temperature = 0.3
max_iterations = 10

[agents.risk]
provider = "cloud"
model = "openai/gpt-4o"
temperature = 0.2
max_iterations = 8

[agents.executor]
provider = "cloud"
model = "anthropic/claude-sonnet-4-5"
temperature = 0.1
max_iterations = 5

[alerts]
auto_analyze = true
require_approval = true
wwatcher_db = "~/.config/wwatcher/wwatcher.db"

[collector]
predictions_db = "~/.config/prowl-bot/data/predictions.db"
resolution_check_interval_hours = 6
training_export_threshold = 500       # auto-export when this many resolved

[markets.kalshi]
api_key_id = ""
private_key = ""

[markets.polymarket]
wallet_private_key = ""
```

### Setup Wizard

```
prowl-bot setup

🐋 Prowl Bot Setup
═══════════════════

Step 1/6: Telegram Bot
  Bot token (from @BotFather): ___
  Your Telegram user ID: ___

Step 2/6: Cloud Provider
  Provider: [OpenRouter]
  API key: ___

Step 3/6: Local Models (optional — skip for server-only)
  Ollama running? Checking localhost:11434... ✓ Found / ✗ Not found (cloud fallback)
  Pulling nomic-embed-text... ✓
  Pulling llama3.2... ✓

Step 4/6: ONNX Models (optional — skip for server-only)
  Download FinBERT sentiment model? [Y/n]
  Download BGE reranker? [Y/n]

Step 5/6: Trading Platforms (optional)
  Configure Kalshi API? [y/N]
  Configure Polymarket wallet? [y/N]

Step 6/6: wwatcher Integration
  Found wwatcher.db at ~/.config/wwatcher/wwatcher.db ✓
  Scanning existing alerts for market resolution data...
  Found 2,314 Polymarket markets, 550 Kalshi markets to track.

Config saved to ~/.config/prowl-bot/config.toml
Run: prowl-bot start
```

---

## Portability & Cost

### Training Cost

| Method | Where | Cost |
|--------|-------|------|
| SFT (MLX LoRA) | Mac (Apple Silicon, 32GB+) | Free |
| GRPO (Unsloth, 7B QLoRA) | Mac (16GB+ GPU memory) | Free |
| GRPO (Unsloth, 14B or Mac insufficient) | Cloud GPU (vast.ai, runpod) | $1-2/hr, ~$3-24 per cycle |
| Bot inference | Server via OpenRouter | Per-token (cloud LLM pricing) |
| Embeddings (Ollama) | Mac when available | Free |
| Embeddings (cloud fallback) | Server via OpenRouter | Minimal |

Training is free if your Mac can handle it. Cloud GPU only needed for larger models or GRPO if Mac RAM is insufficient.

### What Can Be Pushed to Git

| Artifact | Size | Push? |
|----------|------|-------|
| `predictions.db` | MBs | Yes |
| Training JSONL (`train.jsonl`, etc.) | MBs | Yes |
| `brain.db` (agent memories) | MBs | Yes |
| Base model (Qwen 7B, Llama 8B) | ~4GB | No — `ollama pull` on each machine |
| Fine-tuned GGUF (`prowl-v1.gguf`) | ~4GB | No — rsync or rebuild from adapters |
| LoRA adapters | 50-200MB | Optional (git LFS, or just retrain) |
| ONNX models (FinBERT, reranker) | 100-400MB each | No — download on setup |

```gitignore
# bot/.gitignore
data/models/*.gguf
data/models/*.onnx
adapters/
fused/
```

The portable data (predictions.db, training JSONL, brain.db) is small and can be pushed or rsync'd freely. Large model files are downloaded or rebuilt on each machine.

### Training on Any Machine

Clone the repo, copy the data folder, train:

```bash
# On any machine with MLX (Mac) or Unsloth (GPU)
git clone https://github.com/neur0map/polymaster.git
cd polymaster/bot

# Copy data from server (rsync, scp, USB, whatever)
rsync -av server:~/.config/prowl-bot/data/ ./data/

# Download base model (once per machine)
ollama pull qwen2.5:7b

# Export training data from predictions.db
prowl-bot export-training --format sft --output data/training/sft/

# Train (MLX on Mac)
mlx_lm.lora --model Qwen/Qwen2.5-7B --train --data data/training/sft/ \
  --iters 400 --batch-size 1 --num-layers 4 --grad-checkpoint

# Fuse + export GGUF
mlx_lm.fuse --model Qwen/Qwen2.5-7B --adapter-path adapters --save-path fused/ --de-quantize
python llama.cpp/convert_hf_to_gguf.py fused/ --outfile data/models/prowl-v1.gguf --outtype q4_k_m

# Import to Ollama and test
ollama create prowl-v1 -f Modelfile
ollama run prowl-v1 "Analyze: BTC 100k market, whale bought $150k YES at 65%"

# Copy trained model back to server (if server runs Ollama)
rsync data/models/prowl-v1.gguf server:~/.config/prowl-bot/data/models/
```

### Model Flexibility

You are NOT locked to one base model. Switch between training cycles:

```
Cycle 1: Qwen 2.5 7B → prowl-v1 (LoRA adapters tied to Qwen)
Cycle 2: Llama 3.2 8B → prowl-v2 (new adapters, same training data)
Cycle 3: Qwen 2.5 7B → prowl-v3 (back to Qwen, more data)
```

Your training data (predictions.db, JSONL) is model-agnostic. LoRA adapters are tied to a specific base, but the next cycle can use a different base entirely. Run multiple versions in Ollama simultaneously to A/B test.

Swap models in config with one line:

```toml
[agents.analyst]
model = "prowl-v3"     # fine-tuned
# model = "qwen2.5:7b" # or fall back to generic
```

---

## Implementation Phases

### Phase 1: Talking Bot + Data Collection

The manager talks to you via Telegram. No subagents, no trading. Conversational bot with personality, memory, and prediction logging from day one.

Build:
- Config system (TOML parsing, setup wizard)
- Telegram (extracted from zeroclaw — polling, send, split, reactions, streaming, media)
- Provider trait + OpenRouter implementation
- Provider trait + Ollama implementation (with cloud fallback)
- Agent struct (soul.md loading, skill loading)
- Memory (SQLite, embeddings, hybrid recall)
- Manager agent only — conversational, remembers you
- Predictions database schema + logging
- Market resolution tracker (background task)

Test: Talk to it in Telegram. It has personality. Tell it preferences. Close app. Reopen. It remembers. Check predictions.db is accumulating data.

### Phase 2: Team Spawning

Manager can delegate to subagents. No trading — analysis only.

Build:
- Agent spawning (tokio::spawn, concurrent execution)
- Researcher agent + Analyst agent + Risk agent
- Analyst reads wwatcher.db (whale alert history)
- Manager delegation tool + synthesis
- Escalation path (subagent → manager)
- Working memory compression (Ollama, or skip on server-only)
- Episodic memory (learning extraction after tasks)
- Prediction logging for all team analyses

Test: "What are whales doing on BTC?" → Manager spawns team, gets analysis, replies with synthesized report. Predictions logged.

### Phase 3: Alert Pipeline

wwatcher alerts auto-trigger the team.

Build:
- Gateway (Axum HTTP server, webhook endpoint)
- Pairing auth (6-digit code)
- Alert auto-analysis pipeline
- Approval flow in Telegram (approve / pass / adjust)
- Configure wwatcher webhook → bot gateway

Test: wwatcher detects whale → fires webhook → bot analyzes → sends report in Telegram → you approve or pass. Every analysis logged to predictions.db.

### Phase 4: Execution

Place trades after approval.

Build:
- Executor agent
- Kalshi API integration (auth, order placement, positions)
- Polymarket CLOB integration (wallet signing, orders)
- Position tracking + P&L reporting
- Executor memory (past trades, outcomes)
- Profit/loss tracking linked to predictions.db

Test: Full loop — alert → analysis → approval → order placed → confirmation in Telegram. Trade outcomes tracked.

### Phase 5: ML Models

Enhanced signals via ONNX. Feature-gated, optional.

Build:
- ONNX runtime wrapper (behind `local-ml` feature flag)
- Model download/management CLI
- FinBERT sentiment as researcher tool
- BGE reranker in memory recall pipeline
- Time series model as analyst tool
- Memory hygiene (dedup, promotion, decay)

Test: Improved recall quality, sentiment signals in reports, price predictions complement whale analysis.

### Phase 6: Continual Learning

Self-improving model via SFT and GRPO. Runs on Mac.

Build:
- Training data exporter (predictions.db → JSONL)
- SFT pipeline documentation + scripts (MLX LoRA)
- GRPO pipeline documentation + scripts (Unsloth/TRL)
- GGUF export → Ollama import pipeline
- Model versioning (prowl-v1, prowl-v2, ...)
- Evaluation framework (Brier score on held-out set)
- Config support for fine-tuned model as agent provider

**SFT (500+ resolved predictions):**
- Filter best predictions (Brier < 0.25)
- MLX LoRA fine-tune on Mac (7B Qwen, ~250 tok/s)
- Export GGUF → Ollama
- Cadence: monthly → bi-weekly as data grows

**GRPO (2,000+ resolved predictions):**
- Modified GRPO (no std_dev scaling) with Brier reward
- Unsloth on Mac or rented GPU ($1-2/hr)
- Single-pass chronological training
- Export GGUF → deploy
- Cadence: monthly

**Bootstrapping with existing data:**
- Scan 3,150 existing whale alerts for market resolution status
- Use [LightningRodLabs test dataset](https://huggingface.co/datasets/LightningRodLabs/outcome-rl-test-dataset) (1,265 resolved Polymarket questions) for initial evaluation baseline

Test: Fine-tuned model outperforms base model on held-out resolved markets. Analyst agent uses prowl-v1 for routine analysis instead of Claude.

---

## References

- [Outcome-based RL to Predict the Future](https://arxiv.org/abs/2505.17989) — Turtel et al., TMLR 2025. 14B model trained with modified GRPO on Polymarket data.
- [LightningRodLabs Test Dataset](https://huggingface.co/datasets/LightningRodLabs/outcome-rl-test-dataset) — 1,265 resolved Polymarket questions with predictions.
- [MLX LoRA Documentation](https://github.com/ml-explore/mlx-lm/blob/main/mlx_lm/LORA.md) — Apple Silicon fine-tuning.
- [Unsloth GRPO Tutorial](https://unsloth.ai/blog/r1-reasoning) — Local RL training with GGUF export.
- [HuggingFace TRL GRPOTrainer](https://huggingface.co/docs/trl/main/en/grpo_trainer) — Reference GRPO implementation.
- [ZeroClaw](https://github.com/zeroclaw-labs/zeroclaw) — Telegram integration, gateway, soul/memory/skills system (source for extraction).
- [CrewAI](https://github.com/crewAIInc/crewAI) — Multi-agent orchestration patterns (translated to Rust).
