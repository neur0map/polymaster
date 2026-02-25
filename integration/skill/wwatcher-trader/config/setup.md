# Guided Setup — wwatcher-trader

## Role
Walk the user through first-time configuration of the wwatcher-trader skill. Collect credentials, preferences, and risk limits through a conversational flow. Write the final config to memory.

## Files to Write
- `~/.openclaw/memory/wwatcher_trading/config.json` — Final config (copy from templates/config.json, fill in user values)
- `~/.openclaw/memory/wwatcher_trading/portfolio.json` — Zero-state portfolio (copy from templates/portfolio.json)
- `~/.openclaw/memory/wwatcher_trading/improvements.md` — Empty rulebook (copy from templates/improvements.md)
- `~/.openclaw/memory/wwatcher_trading/pipeline.jsonl` — Empty (copy from templates/pipeline.jsonl)
- `~/.openclaw/memory/wwatcher_trading/trades.jsonl` — Empty (copy from templates/trades.jsonl)
- `~/.openclaw/memory/wwatcher_trading/lessons.jsonl` — Empty (copy from templates/lessons.jsonl)

## Conversation Flow

### Step 1: Kalshi Account
Ask if the user has a Kalshi account with API access.

**If no:**
> Go to kalshi.com, create an account, then navigate to Profile > API Keys.
> Generate an RSA key pair. You'll get an API Key ID and a private key PEM file.
> Paste your API Key ID when ready.

**If yes:**
> Paste your Kalshi API Key ID.

Validate format: should be a non-empty string. Store as `kalshi.api_key_id`.

### Step 2: Private Key Path
> Where is your Kalshi private key PEM file stored? Give me the full path.
> Example: `~/.kalshi/private_key.pem`

Validate: run `test -f <path>` to confirm the file exists. Store as `kalshi.private_key_path`.

**CRITICAL**: Never read the PEM file contents. Never embed the key in JSON. Only store the filesystem path.

### Step 3: Verify API Connection
Run a test call to confirm credentials work:

```bash
# Generate timestamp and signature, then call Kalshi balance endpoint
node -e "
const crypto = require('crypto');
const fs = require('fs');
const key = fs.readFileSync('<private_key_path>', 'utf8');
const ts = Math.floor(Date.now() / 1000);
const method = 'GET';
const path = '/trade-api/v2/portfolio/balance';
const body = '';
const message = ts + method + path + body;
const sig = crypto.sign('sha256', Buffer.from(message), {
  key: key,
  padding: crypto.constants.RSA_PKCS1_PSS_PADDING,
  saltLength: crypto.constants.RSA_PSS_SALTLENGTH_DIGEST
}).toString('base64');
console.log(JSON.stringify({ ts: ts.toString(), sig }));
"
```

Then use the output to call:
```bash
curl -s -X GET "https://api.elections.kalshi.com/trade-api/v2/portfolio/balance" \
  -H "KALSHI-ACCESS-KEY: <api_key_id>" \
  -H "KALSHI-ACCESS-SIGNATURE: <sig>" \
  -H "KALSHI-ACCESS-TIMESTAMP: <ts>" \
  -H "Content-Type: application/json"
```

**If success (HTTP 200):** Report balance and mark `kalshi.verified: true`.
**If failure:** Show error, ask user to double-check credentials. Retry from Step 1.

### Step 4: Categories
> What markets interest you? Pick from:
> - **crypto** — Bitcoin/ETH price targets, crypto events
> - **politics** — Elections, policy, legislation
> - **weather** — Temperature records, precipitation, storms
> - **sports** — Game outcomes, player stats, season results

Store selected categories in `categories` array.

### Step 5: Risk Limits
> Let's set your risk limits. I recommend starting conservative:
> - **Max $50 per trade** — largest single position
> - **Max $200 per day** — total daily spending cap
> - **Max 3 open positions** — concurrent position limit
>
> Use these defaults or customize?

If customize: ask for each value individually. Validate: all must be positive numbers.

### Step 6: Execution Tiers
> Auto-execution tiers control when I trade autonomously vs ask you first:
> - **Auto-execute**: 80%+ confidence AND under $25 → I trade immediately
> - **Need approval**: 60-79% confidence OR over $25 → I ask you first
> - **Skip**: below 60% confidence → never trade
>
> Sound good, or want to adjust thresholds?

### Step 7: Payout Multiplier
> Minimum payout multiplier — this filters out low-reward trades.
> On Kalshi, every contract pays $1 if correct. So:
> - YES at $0.55 → 1.82x return ✓
> - YES at $0.40 → 2.50x return ✓
> - YES at $0.70 → 1.43x return ✗ skip
>
> I recommend **1.8x minimum** — only trade when you can nearly double your money.
> Use 1.8x default or set your own?

### Step 8: Dry Run Mode
> **Important safety note:** I'll start in **dry-run mode**.
> This means I'll scan markets, research opportunities, and make decisions,
> but I won't place real trades until you tell me "enable live trading".
>
> This lets you see how the system works before risking real money.
> You can enable live trading anytime with: "enable live trading"

Set `dry_run: true`.

### Step 9: Write Config & Initialize Memory
1. Create directory: `mkdir -p ~/.openclaw/memory/wwatcher_trading/`
2. Write `config.json` with all collected values
3. Copy template files for portfolio.json, improvements.md, pipeline.jsonl, trades.jsonl, lessons.jsonl
4. Set `setup_complete: true`, `last_updated` to current ISO timestamp

### Step 10: Confirm
> All set! Here's your configuration:
> - **Categories**: {list}
> - **Risk limits**: ${max_trade}/trade, ${daily_limit}/day, {max_positions} positions
> - **Auto-execute**: {min_confidence}%+ confidence under ${max_trade_size}
> - **Min multiplier**: {multiplier}x
> - **Mode**: Dry run (no real trades)
> - **Kalshi**: Verified ✓
>
> Starting market scanner now. I'll notify you when I find opportunities.

## Error Handling
- If PEM file not found: ask user to verify path, offer to search common locations (`~/.kalshi/`, `~/Downloads/`, `~/.ssh/`)
- If API verification fails with 401: credentials are wrong, retry from Step 1
- If API verification fails with network error: ask user to check internet, retry
- If user wants to skip verification: warn that trading won't work, allow skip but set `verified: false`
