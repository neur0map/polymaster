---
name: poly-trading
description: Use when the user wants to place bets or trade on Kalshi, read live order books, check balance or positions, or act on poly whale alerts with real orders, after configuring Kalshi API keys. Covers authenticated request signing, the V2 order endpoints, and mandatory safety confirmations.
---

# Trading on Kalshi: order books and order placement

poly itself is signal-only (watch, alert, store). Placing orders is done by the agent calling Kalshi's authenticated REST API directly, using the same API keys stored in poly's config. Kalshi only; Polymarket order placement is not supported (requires Ethereum wallet signing).

## 1. Hard safety rules

1. **Every order requires explicit user confirmation** showing ticker, side, count, price, and estimated cost. Never place an order from an alert or a heuristic alone.
2. **Never exceed a stake the user stated.** If no max stake was given, ask before trading.
3. **Start on the demo environment** (separate demo account and demo keys) until the user explicitly says to go live.
4. One order at a time until the user asks for anything else. No loops, no retries on ambiguity. A 5xx or timeout means the order state is unknown: check `GET /portfolio/orders` before ever re-submitting.
5. Whale alerts are research signals, not trade signals. Say so when presenting them.

## 2. Prerequisites

- Kalshi account with API keys configured in poly: `poly setup` option `[8]`, or keys in `~/.config/poly/config.json` (`kalshi_api_key_id`, `kalshi_private_key` as PEM text). Generate keys at https://kalshi.com/profile/api-keys.
- Key scope: order endpoints need the `write::trade` scope (a parent `write` scope also works). `read` covers balance, positions, and all market data.
- Rate-limit budget is tiered (`GET /trade-api/v2/account/limits` shows your tier and grants). Basic can trade small; self-serve upgrade to Advanced via `POST /trade-api/v2/account/upgrade`... use `POST /trade-api/v2/account/api_usage_level/upgrade`.
- Region attestation: keys stop working for Sports/Elections/Entertainment markets when the attestation expires (`GET /trade-api/v2/api_keys` returns `api_key_region_expiration_ts`).
- Check funds: `GET /portfolio/balance` returns `balance_dollars` (fixed-point string).

## 3. Authenticated requests

Base URLs: production `https://api.elections.kalshi.com/trade-api/v2` (also `external-api.kalshi.com`), demo `https://demo-api.kalshi.co/trade-api/v2` (demo keys are a separate account).

Every authenticated request needs three headers:

```
KALSHI-ACCESS-KEY: <key id>
KALSHI-ACCESS-SIGNATURE: <base64 RSA-PSS signature>
KALSHI-ACCESS-TIMESTAMP: <unix milliseconds>
```

Signing: RSA-PSS with SHA-256 and salt length 32 (= digest length) over the string `timestamp + METHOD + path` where path excludes the query string (e.g. `1726233600000POST/trade-api/v2/portfolio/events/orders`). The same scheme poly implements for its WebSocket connection (see `sign_handshake` in `src/ws/kalshi.rs` in the poly repo).

Shell recipe (keys in poly's config, private key extracted to a PEM file):

```bash
KEY_ID="<kalshi_api_key_id from poly config>"
TS=$(date +%s%3N)
MSG="${TS}GET/trade-api/v2/portfolio/balance"
SIG=$(printf '%s' "$MSG" | openssl dgst -sha256 \
      -sigopt rsa_padding_mode:pss -sigopt rsa_pss_saltlen:32 \
      -sign key.pem | base64 -w0)
curl -sS "https://api.elections.kalshi.com/trade-api/v2/portfolio/balance" \
  -H "KALSHI-ACCESS-KEY: $KEY_ID" \
  -H "KALSHI-ACCESS-SIGNATURE: $SIG" \
  -H "KALSHI-ACCESS-TIMESTAMP: $TS"
```

Generate a fresh timestamp and signature for every request; reuse within the same second is fine, minutes are not.

## 4. Reading markets and order books

Market object (`GET /markets/{ticker}`), all prices are fixed-point dollar strings:

- `yes_bid_dollars`, `yes_ask_dollars`, `no_bid_dollars`, `no_ask_dollars`, `last_price_dollars`
- `volume_24h_fp`, `open_interest_fp`, `liquidity_dollars`
- `status` (`active`, `finalized`, ...), `title`, `strike_type`, `close_time`
- `price_ranges`: array of `{start, end, step}` bands. **The step of the band containing your price is the only valid tick.** Many markets use $0.01 center ticks with $0.001 (or even $0.0001 on combo markets) edge ticks below $0.01 and above $0.99. Snap order prices to a band step; off-tick prices are rejected.

Order book (`GET /markets/{ticker}/orderbook`): returns `orderbook_fp.yes_dollars` and `orderbook_fp.no_dollars`, each an array of `["<price in dollars>", "<quantity in contracts>"]` string pairs sorted by price. These are bid books: `yes_dollars` holds bids for YES, `no_dollars` holds bids for NO. Best YES ask = 1 minus the best NO bid.

While `poly watch` runs, every alert already prints top-of-book, depths, and market context; use the REST calls above when evaluating a specific market manually. Polymarket alerts also carry a direct market page link (`market_context.url` in payloads, printed as `Link:` in the terminal); Kalshi's API exposes no page URL, so find its markets by searching the event ticker on kalshi.com.

## 5. Placing an order

`POST /portfolio/events/orders` (V2; the legacy `/portfolio/orders` endpoints are deprecated and burn 10x rate-limit tokens):

```json
{
  "ticker": "KXNFLGAME-26SEP13CLEJAC-JAC",
  "side": "bid",
  "count": "10.00",
  "price": "0.5600",
  "time_in_force": "good_till_canceled",
  "self_trade_prevention_type": "taker_at_cross"
}
```

- `side`: `bid` = buy YES, `ask` = sell YES (the book is YES-leg only; selling YES is equivalent to buying NO at 1 - price).
- `count`: fixed-point contract string, 0.01 granularity ("10", "10.00").
- `price`: fixed-point dollars snapped to the market's `price_ranges` step.
- `time_in_force`: `good_till_canceled` (rests; optional `expiration_time` unix seconds for GTT), `fill_or_kill` (all or nothing now), `immediate_or_cancel` (partial fills allowed, rest cancelled).
- `self_trade_prevention_type`: `taker_at_cross` (default choice) or `maker`.
- Useful optional flags: `post_only` (reject/cancel instead of crossing), `reduce_only` (cap at current position), `cancel_order_on_pause` (auto-cancel if trading halts), `client_order_id` (idempotency; reuse it when retrying an unconfirmed request), `exchange_index` (omit for auto-routing by ticker).

Responses: `201` with `order_id`, `fill_count`, `remaining_count`, and, when filled immediately, `average_fill_price` and `average_fee_paid`. Error bodies are `{code, message, details}`; 429 means rate limit (default 10 tokens per order).

Typical orders:

- **Take (buy at market)**: side `bid`, price = best YES ask, `fill_or_kill`.
- **Make (provide liquidity)**: side `bid`, price inside the spread, `good_till_canceled`, `post_only` true.

## 6. Managing orders and account state

```
GET    /portfolio/orders?status=resting           # open orders
DELETE /portfolio/events/orders/{order_id}        # cancel one
POST   /portfolio/events/orders/{order_id}/amend  # change price/count
POST   /portfolio/events/orders/{order_id}/decrease
GET    /portfolio/positions                       # open positions
GET    /portfolio/fills                           # execution history
DELETE /portfolio/orders/cancel-all               # cancel everything resting
```

## 7. 2026 API changes to keep in mind

- Fixed-point dollars everywhere (`*_dollars`, `*_fp` strings); integer-cent fields were removed from markets and trades in 2026.
- V2 event-order endpoints replaced legacy order mutations; legacy paths now cost 10x rate-limit tokens.
- Exchange sharding: markets live on shards (crypto, tennis, baseball, commodities on 2, basketball on 3). Omit `exchange_index` (or send -1) to auto-route by ticker.
- Kalshi requires authentication on the WebSocket handshake too (poly's watch already signs it when keys are configured).
- Public trades carry `is_block_trade`; block trades are matched off-book and can distort "whale" reads, so treat them as informed-but-off-book flow.

## 8. Poly + agent loop

1. `poly watch` (or the webhook) surfaces a whale alert with market context and order book.
2. Pull the full book and `price_ranges` for the market, check `status=active`, check liquidity and spread.
3. Form a thesis, size it within the user's stated stake, present the exact order JSON to the user.
4. On explicit approval, sign and POST, then verify with `GET /portfolio/orders` and report `order_id`, fills, and fees.
