# API Reference

All external API endpoints used by wwatcher, organized by platform and purpose.

---

## Endpoints Summary

wwatcher uses **15 API endpoints** across 3 Polymarket APIs, 1 Kalshi REST API, and 1 Kalshi WebSocket.

| # | Platform | Endpoint | Purpose | Auth |
|---|----------|----------|---------|------|
| 1 | Polymarket | `GET data-api/trades` | Fetch whale trades (server-side filtered) | None |
| 2 | Polymarket | `GET gamma-api/markets` | Market context + tags | None |
| 3 | Polymarket | `GET clob/book` | Order book depth | None |
| 4 | Polymarket | `GET data-api/value` | Whale portfolio value | None |
| 5 | Polymarket | `GET data-api/positions` | Open positions count | None |
| 6 | Polymarket | `GET data-api/closed-positions` | Win rate calculation | None |
| 7 | Polymarket | `GET data-api/leaderboard` | Top 500 leaderboard | None |
| 8 | Polymarket | `GET data-api/top-holders` | Top holders per market | None |
| 9 | Kalshi | `GET /markets/trades` | Fetch recent trades | Optional |
| 10 | Kalshi | `GET /markets/{ticker}` | Market details + category | None |
| 11 | Kalshi | `GET /markets/{ticker}/orderbook` | Order book depth | None |
| 12 | Kalshi | `WSS /ws/v2` (trade channel) | Real-time trade stream | None |

---

## Polymarket APIs

Polymarket has 3 separate API services:

- **Data API** (`data-api.polymarket.com`) — Trade data, user portfolios, leaderboard
- **Gamma API** (`gamma-api.polymarket.com`) — Market metadata, tags, events
- **CLOB API** (`clob.polymarket.com`) — Order book, prices

### 1. Fetch Whale Trades

```
GET https://data-api.polymarket.com/trades
```

**Query Parameters:**

| Param | Value | Description |
|-------|-------|-------------|
| `limit` | `500` | Number of trades to return |
| `filterType` | `CASH` | Filter by USD value |
| `filterAmount` | `25000` | Minimum trade value in USD |
| `takerOnly` | `true` | Only taker trades (not maker fills) |

**Response:** Array of trade objects.

```json
[
  {
    "transactionHash": "0xabc123...",
    "conditionId": "0x5f9c...",
    "asset": "71321...",
    "side": "BUY",
    "size": 76923.08,
    "price": 0.65,
    "timestamp": 1707840000,
    "name": "0x742d...",
    "proxyWallet": "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb",
    "title": "Will Bitcoin reach 100k?",
    "outcome": "Yes"
  }
]
```

**Notes:**
- Server-side `filterType=CASH` eliminates client-side filtering overhead
- `proxyWallet` or `name` field provides the wallet address
- `title` and `outcome` are included directly in newer API responses

### 2. Market Context (Gamma API)

```
GET https://gamma-api.polymarket.com/markets?condition_ids={condition_id}
```

**Response:** Array of market objects.

Key fields extracted:
- `outcomePrices` — JSON string like `["0.65","0.35"]`
- `spread` — Bid-ask spread
- `volume24hr` — 24h volume
- `openInterest` — Total open interest
- `oneDayPriceChange` — 24h price change
- `liquidityClob` — CLOB liquidity
- `tags` — Array of tag objects with `slug`/`label` fields

### 3. Order Book (CLOB API)

```
GET https://clob.polymarket.com/book?token_id={asset_id}
```

**Response:**

```json
{
  "bids": [
    { "price": "0.64", "size": "5000" },
    { "price": "0.63", "size": "8000" }
  ],
  "asks": [
    { "price": "0.66", "size": "4500" },
    { "price": "0.67", "size": "7000" }
  ]
}
```

**Usage:** Calculates bid/ask depth within 10% of best price, number of levels, and spread.

### 4. Portfolio Value

```
GET https://data-api.polymarket.com/value?user={wallet_address}
```

**Response:** May be a plain number or `{ "value": 2340000.0 }`.

**Cache:** 30 minutes per wallet.

### 5. Open Positions

```
GET https://data-api.polymarket.com/positions?user={wallet_address}&limit=100
```

**Response:** Array of position objects. Count is used as `positions_count`.

**Cache:** 30 minutes per wallet (part of whale profile).

### 6. Closed Positions (Win Rate)

```
GET https://data-api.polymarket.com/closed-positions?user={wallet_address}&limit=100
```

**Response:** Array of closed position objects with `payout` and `cashPaid` fields.

**Win rate calculation:** `wins / total` where `payout > cashPaid` is a win.

**Cache:** 30 minutes per wallet (part of whale profile).

### 7. Leaderboard

```
GET https://data-api.polymarket.com/leaderboard?limit=500
```

**Response:** Array of leaderboard entries.

```json
[
  {
    "proxyWallet": "0x742d...",
    "rank": 45,
    "profit": 890000.0,
    "volume": 12500000.0,
    "marketsTraded": 195
  }
]
```

**Cache:** 1 hour (same data for all alerts).

**Usage:** When a whale trade occurs, check if their wallet is in the top 500 leaderboard. If so, include their rank, profit, and markets traded in the whale profile.

### 8. Top Holders

```
GET https://data-api.polymarket.com/top-holders?market={condition_id}
```

**Response:** Array of holder objects with `proxyWallet`/`wallet`, `shares`/`size`, and `value` fields.

**Usage:** Shows top 5 holders and their share concentration for each whale alert market.

---

## Kalshi APIs

### 9. Fetch Recent Trades

```
GET https://api.elections.kalshi.com/trade-api/v2/markets/trades?limit=100
```

**Optional auth header:** `KALSHI-ACCESS-KEY: {api_key_id}`

**Response:**

```json
{
  "trades": [
    {
      "trade_id": "abc123",
      "ticker": "KXBTCD-26FEB13-T100000",
      "price": 65,
      "count": 500,
      "yes_price": 65.0,
      "no_price": 35.0,
      "taker_side": "yes",
      "created_time": "2026-02-13T18:00:00Z"
    }
  ]
}
```

**Notes:**
- Kalshi trades are **anonymous** — no user/wallet IDs
- `taker_side` is `"yes"` or `"no"`, never `"sell"`/`"buy"`
- Prices are in cents (0-100), converted to 0.0-1.0 internally

### 10. Market Details

```
GET https://api.elections.kalshi.com/trade-api/v2/markets/{ticker}
```

**Response:**

```json
{
  "market": {
    "title": "Bitcoin above $100,000 on Feb 13?",
    "subtitle": "...",
    "category": "Crypto",
    "tags": ["crypto", "bitcoin"],
    "yes_bid": 65,
    "yes_ask": 67,
    "no_bid": 33,
    "last_price": 66,
    "previous_price": 62,
    "volume_24h": 15000,
    "open_interest": 45000,
    "liquidity": 25000
  }
}
```

**Usage:**
- Market title for display
- Native `category` field for filtering (more accurate than keyword matching)
- Price/volume/OI for market context

### 11. Order Book

```
GET https://api.elections.kalshi.com/trade-api/v2/markets/{ticker}/orderbook
```

**Response:**

```json
{
  "orderbook": {
    "yes": [[65, 500], [64, 300], [63, 200]],
    "no": [[35, 400], [34, 250]]
  }
}
```

Format: `[[price_in_cents, quantity], ...]`

### 12. WebSocket (Real-Time Trades)

```
WSS wss://api.elections.kalshi.com/trade-api/ws/v2
```

**Subscribe command:**

```json
{
  "id": 1,
  "cmd": "subscribe",
  "params": {
    "channels": ["trade"]
  }
}
```

**Trade message format:**

```json
{
  "type": "trade",
  "msg": {
    "trades": [
      {
        "trade_id": "abc123",
        "ticker": "KXBTCD-26FEB13-T100000",
        "count": 500,
        "yes_price": 65.0,
        "no_price": 35.0,
        "taker_side": "yes",
        "created_time": "2026-02-13T18:00:00Z"
      }
    ]
  }
}
```

**Connection details:**
- Ping every 10 seconds to keep alive
- Auto-reconnect with exponential backoff (2s to 60s max)
- Falls back to HTTP polling if WebSocket goes silent for ~1 minute

---

## Data Flow Architecture

```
                        ┌─────────────────┐
                        │   Kalshi WS      │
                        │  (trade channel)  │
                        └────────┬─────────┘
                                 │ real-time trades
                                 ▼
┌──────────────────┐    ┌──────────────────┐    ┌──────────────────┐
│ Polymarket       │    │                  │    │ Kalshi HTTP      │
│ Data API /trades │───▶│  Watch Loop      │◀───│ /markets/trades  │
│ (5s polling)     │    │  (threshold      │    │ (fallback only)  │
└──────────────────┘    │   filter)        │    └──────────────────┘
                        └────────┬─────────┘
                                 │ whale detected
                                 ▼
                   ┌─────────────────────────────┐
                   │    Per-Alert Enrichment       │
                   │                               │
                   │  1. Market Context (1 call)   │
                   │  2. Whale Profile  (3 calls)  │
                   │  3. Order Book     (1 call)   │
                   │  4. Top Holders    (1 call)   │
                   └──────────┬────────────────────┘
                              │
              ┌───────────────┼───────────────┐
              ▼               ▼               ▼
        ┌──────────┐   ┌──────────┐   ┌──────────┐
        │ Terminal  │   │ SQLite   │   │ Webhook  │
        │ Display   │   │ History  │   │ (n8n)    │
        └──────────┘   └──────────┘   └──────────┘
```

**Per Polymarket whale alert:** Up to 6 API calls (market context + portfolio value + positions + closed positions + order book + top holders). Whale profile is cached for 30 min.

**Per Kalshi whale alert:** Up to 3 API calls (market details + market context + order book). No whale profile (anonymous trades).

---

## Caching Strategy

| Data | TTL | Storage | Reason |
|------|-----|---------|--------|
| Whale profile (per wallet) | 30 minutes | In-memory HashMap | Avoid hitting Data API on every alert from same whale |
| Leaderboard (top 500) | 1 hour | In-memory Vec | Same for all alerts, rarely changes |
| Wallet memory | 12 hours | SQLite | Persistent across restarts for returning whale detection |
| Alert history | Configurable (default 30 days) | SQLite | Long-term storage and querying |

---

## Rate Limits

| API | Documented Limit | Our Usage |
|-----|-----------------|-----------|
| Polymarket Data API | No published limit | ~1 req/5s (trades) + 1-6 req/whale |
| Polymarket Gamma API | No published limit | 1 req/whale (market context) |
| Polymarket CLOB API | No published limit | 1 req/whale (order book) |
| Kalshi REST API | ~10 req/s (public) | ~1 req/5s (fallback) + 1-3 req/whale |
| Kalshi WebSocket | No published limit | 1 persistent connection |

If you experience rate limiting, increase the `--interval` flag:

```bash
wwatcher watch --interval 10  # poll every 10 seconds instead of 5
```

---

## Configuration

All API behavior is controlled via `~/.config/wwatcher/config.json` (or `wwatcher setup`).

```json
{
  "categories": ["all"],
  "threshold": 25000,
  "platforms": ["polymarket", "kalshi"],
  "history_retention_days": 30,
  "kalshi_api_key_id": null,
  "kalshi_private_key": null,
  "webhook_url": "https://your-n8n-instance/webhook/xxx"
}
```

| Field | Default | Description |
|-------|---------|-------------|
| `categories` | `["all"]` | Market categories to watch. See categories.rs for full list. |
| `threshold` | `25000` | Minimum trade value in USD to trigger alert |
| `platforms` | `["polymarket", "kalshi"]` | Which platforms to monitor |
| `history_retention_days` | `30` | Days to keep alert history in SQLite |
| `kalshi_api_key_id` | `null` | Optional Kalshi API key for enhanced access |
| `kalshi_private_key` | `null` | Optional Kalshi private key |
| `webhook_url` | `null` | Webhook URL for external notifications |
