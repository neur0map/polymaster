# Trade Executor

## Role
Execute trades on Kalshi by signing API requests with RSA-PSS and placing orders. Log all trade results. This is the only agent that spends real money.

## Files to Read
- `~/.openclaw/memory/wwatcher_trading/config.json` — Kalshi API credentials (key ID + PEM path), dry_run flag, base URL
- `~/.openclaw/memory/wwatcher_trading/pipeline.jsonl` — Decision maker's EXECUTE entry for this market

## Files to Write
- `~/.openclaw/memory/wwatcher_trading/trades.jsonl` — Append trade execution record
- `~/.openclaw/memory/wwatcher_trading/pipeline.jsonl` — Append execution result entry

## Parameters (from orchestrator)
- `market_id`: Kalshi ticker to trade
- `market_title`: Human-readable market name
- `side`: YES or NO
- `size`: Dollar amount to spend
- `entry_price`: Target price (limit order)
- `stop_loss`: Stop-loss price level
- `take_profit`: Take-profit price level
- `confidence_at_entry`: Decision maker's confidence score
- `agents_agreed`: How many agents agreed
- `execution_id`: UUID for this execution

## CRITICAL: Dry Run Check

**BEFORE any API call that places an order**, read `config.dry_run`:

```
if config.dry_run == true:
  DO NOT call the Kalshi order placement API.
  Instead, write a simulated trade entry with status "simulated".
  Log everything as if the trade happened, but prefix reasoning with "[DRY RUN]".
  Return to orchestrator with the simulated result.
```

This is a hard safety gate. Dry run mode must be explicitly disabled by the user saying "enable live trading" before real orders are placed.

## API Authentication — RSA-PSS Signing

Kalshi's Trade API requires RSA-PSS signed requests. The private key is a PEM file on disk.

### Signing Process

Generate the signature using Node.js crypto (self-contained, no dependencies):

```bash
node -e "
const crypto = require('crypto');
const fs = require('fs');
const key = fs.readFileSync(process.argv[1], 'utf8');
const ts = Math.floor(Date.now() / 1000).toString();
const method = process.argv[2];
const path = process.argv[3];
const body = process.argv[4] || '';
const message = ts + method + path + body;
const sig = crypto.sign('sha256', Buffer.from(message), {
  key: key,
  padding: crypto.constants.RSA_PKCS1_PSS_PADDING,
  saltLength: crypto.constants.RSA_PSS_SALTLENGTH_DIGEST
}).toString('base64');
console.log(JSON.stringify({ ts, sig }));
" "<pem_path>" "<METHOD>" "<api_path>" "<body_json>"
```

**Parameters:**
- `pem_path`: From `config.kalshi.private_key_path`
- `METHOD`: HTTP method (GET, POST)
- `api_path`: API endpoint path (e.g., `/trade-api/v2/portfolio/orders`)
- `body_json`: Request body as JSON string (empty string for GET)

**Output:** JSON with `ts` (timestamp) and `sig` (base64 signature)

### Making Authenticated Requests

```bash
curl -s -X <METHOD> "<base_url><api_path>" \
  -H "KALSHI-ACCESS-KEY: <api_key_id>" \
  -H "KALSHI-ACCESS-SIGNATURE: <sig>" \
  -H "KALSHI-ACCESS-TIMESTAMP: <ts>" \
  -H "Content-Type: application/json" \
  -d '<body_json>'
```

**Headers:**
- `KALSHI-ACCESS-KEY`: From `config.kalshi.api_key_id`
- `KALSHI-ACCESS-SIGNATURE`: From signing output
- `KALSHI-ACCESS-TIMESTAMP`: From signing output

## API Calls

### Place Order
```
POST /trade-api/v2/portfolio/orders
```

**Body:**
```json
{
  "ticker": "<market_id>",
  "action": "buy",
  "side": "yes",
  "type": "limit",
  "count": <contracts>,
  "yes_price": <price_in_cents>,
  "expiration_ts": null
}
```

**Field mapping:**
- `ticker`: `market_id`
- `action`: Always "buy" for entries, "sell" for exits
- `side`: "yes" or "no" (lowercase)
- `type`: "limit" (always use limit orders, never market orders)
- `count`: Number of contracts = `floor(size / entry_price)`
- `yes_price`: Price in cents (integer). If side is YES, use `entry_price * 100`. If side is NO, the yes_price = `(1 - entry_price) * 100`.
- `expiration_ts`: null (good-til-cancelled)

**Response:** Contains `order_id`, `status`, fill details.

### Check Order Status (if needed)
```
GET /trade-api/v2/portfolio/orders/<order_id>
```

## Logic

### Step 1: Validate Prerequisites
1. Read `config.json` — verify `setup_complete: true` and `kalshi.verified: true`
2. Check `config.dry_run` — if true, skip to dry run flow
3. Read the decision entry from `pipeline.jsonl` — verify verdict is EXECUTE

### Step 2: Calculate Order Parameters
```
contracts = Math.floor(size / entry_price)
total_cost = contracts * entry_price
yes_price_cents = Math.round(entry_price * 100)  // for YES side
// OR
yes_price_cents = Math.round((1 - entry_price) * 100)  // for NO side
```

### Step 3: Sign and Place Order
1. Build the request body JSON
2. Sign with RSA-PSS using the Node.js one-liner
3. Execute curl with signed headers
4. Parse response

### Step 4: Handle Response
**If filled (status: "filled" or "resting"):**
- Generate trade ID: `trade_<sequential_number>`
- Write to `trades.jsonl`
- Write execution result to `pipeline.jsonl`
- Report success to orchestrator

**If rejected or error:**
- Write failure to `pipeline.jsonl` with error details
- Do NOT write to `trades.jsonl` (no trade happened)
- Report failure to orchestrator

### Step 5: Exit Execution (for monitor-triggered exits)
When the orchestrator passes `action: "sell"`:
1. Use the same signing process
2. Place a sell order for the position's contracts
3. Calculate PnL: `(sell_price - entry_price) * contracts`
4. Write exit record to `trades.jsonl` with trigger, PnL, hold time
5. Notify orchestrator to run portfolio tracker

## Output Schema

### Pipeline Entry
```json
{
  "id": "<uuid>",
  "ts": "<ISO timestamp>",
  "agent": "executor",
  "market_id": "<kalshi_ticker>",
  "market_title": "<market_title>",
  "stage": "execution",
  "data": {
    "action": "buy|sell",
    "side": "yes|no",
    "contracts": 63,
    "price": 0.63,
    "total_cost": 39.69,
    "order_id": "kalshi_abc123",
    "status": "filled|resting|rejected|simulated",
    "dry_run": false,
    "execution_id": "<uuid>"
  },
  "verdict": "filled|failed|simulated",
  "confidence": null,
  "reasoning": "Limit order placed: 63 YES contracts at 63¢ ($39.69 total). Order filled immediately. [OR: DRY RUN — simulated fill.]"
}
```

### Trade Log Entry
```json
{
  "id": "trade_001",
  "ts": "<ISO timestamp>",
  "market_id": "<kalshi_ticker>",
  "market_title": "<market_title>",
  "side": "YES",
  "action": "BUY",
  "price": 0.63,
  "size": 39.69,
  "contracts": 63,
  "status": "filled|simulated",
  "order_id": "kalshi_abc123",
  "confidence_at_entry": 0.71,
  "agents_agreed": 4,
  "stop_loss": 0.44,
  "take_profit": 0.82,
  "reasoning_summary": "One-line summary of why this trade was made",
  "dry_run": false
}
```

## Success/Failure Handling

**Filled**: Trade logged, portfolio tracker notified, user notified (if config.notifications.on_trade_executed).
**Resting (limit order not immediately filled)**: Log as "resting". Monitor agent will track fill status.
**Rejected (insufficient funds, invalid ticker)**: Log failure. Notify user if unexpected.
**Network error**: Retry once after 5 seconds. If still failing, abort and notify user.
**Signature error**: Likely PEM file issue. Report error, suggest re-running setup.

## Security Notes
- **Never log the private key contents**. Only reference the file path.
- **Never embed the PEM in JSON**. Only pass the path to the Node.js signer.
- **Always use limit orders**. Market orders have no price protection.
- **Validate order response** matches expected parameters before logging as success.
