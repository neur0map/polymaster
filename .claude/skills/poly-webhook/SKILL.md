---
name: poly-webhook
description: Use when setting up, testing, or troubleshooting poly webhook deliveries (n8n, Discord, Slack, custom HTTP receivers), or when building a consumer that needs the poly alert payload JSON schema.
---

# Webhook setup, payload schema, and troubleshooting

poly POSTs one JSON payload per detected whale alert to `webhook_url` from the config. This skill covers wiring a receiver, the exact payload shape, and diagnosing failures. For config file location and general setup see the `poly-setup` skill; the human-facing reference with n8n templates and filter examples lives in `docs/WEBHOOK_REFERENCE.md`.

## 1. Configure

Set `webhook_url` in `~/.config/poly/config.json` (or `poly setup`, option `[9]`; Enter keeps, type a URL to set, `clear` removes):

```json
{
  "webhook_url": "https://your-server.com/webhook/whale-alerts"
}
```

Receiver requirements: accepts HTTPS POST with a JSON body, responds with any 2xx. poly does not follow redirects for you to inspect, does not retry, and ignores auth challenges, so the URL must work unauthenticated or embed a token/path secret. Restart `poly watch` after editing config (config is read at startup).

## 2. Test

```bash
poly test-webhook
```

Sends two fake alerts to the configured URL: a Polymarket BUY ($50,000 @ 0.65) and a Kalshi SELL ($35,000 @ 0.54). Real alerts go through the same code path, so a passing test means live delivery works.

## 3. Payload schema

Top-level fields (field: type, always present unless marked optional):

| Field | Type | Notes |
|-------|------|-------|
| `platform` | string | `"Polymarket"` or `"Kalshi"` |
| `alert_type` | string | `"WHALE_ENTRY"` (buy) or `"WHALE_EXIT"` (sell) |
| `action` | string | `"BUY"` or `"SELL"` |
| `value` | number | Trade value in USD |
| `price` | number | Outcome price 0.0-1.0 |
| `price_percent` | integer | price * 100, rounded |
| `size` | number | Contract count |
| `timestamp` | string | RFC3339 trade timestamp |
| `market_title` | string/null | Sanitized for Markdown receivers (special chars collapsed) |
| `outcome` | string/null | e.g. `"Yes"` |
| `wallet_id` | string, optional | Polymarket only (on-chain wallet); absent for Kalshi |
| `wallet_activity` | object, optional | `{transactions_last_hour, transactions_last_day, total_value_hour, total_value_day, is_repeat_actor, is_heavy_actor}` |
| `market_context` | object, optional | `{yes_price, no_price, spread, volume_24h, open_interest, price_change_24h, liquidity, tags}` plus `url`: a direct link to the market page (Polymarket only; Kalshi's API exposes no page URL) |
| `whale_profile` | object, optional | Polymarket only: `{portfolio_value, leaderboard_rank, leaderboard_profit, win_rate, markets_traded, positions_count}` (fields omitted when unknown) |
| `order_book` | object, optional | `{best_bid, best_ask, bid_depth_10pct, ask_depth_10pct, bid_levels, ask_levels}` |
| `top_holders` | object, optional | `{holders: [{wallet, shares, value}], total_shares}` |

Example body:

```json
{
  "platform": "Polymarket",
  "alert_type": "WHALE_ENTRY",
  "action": "BUY",
  "value": 50000.0,
  "price": 0.65,
  "price_percent": 65,
  "size": 76923.08,
  "timestamp": "2026-09-13T14:42:42.802809Z",
  "market_title": "Will Bitcoin reach 100k by end of 2026?",
  "outcome": "Yes",
  "wallet_id": "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb",
  "wallet_activity": {
    "transactions_last_hour": 2,
    "transactions_last_day": 5,
    "total_value_hour": 125000.0,
    "total_value_day": 380000.0,
    "is_repeat_actor": true,
    "is_heavy_actor": true
  }
}
```

Enrichment objects (`market_context`, `order_book`, `top_holders`, `whale_profile`) are fetched per alert before sending and omitted when a lookup fails or does not apply.

## 4. Troubleshooting

poly logs webhook problems to stderr in these exact shapes:

| Log line | Cause and fix |
|----------|---------------|
| `Webhook failed with status: 4xx \| response: {...}` | Receiver rejected the payload; the logged response body names the reason (n8n: "workflow not active" means the workflow is deactivated or the path/method is wrong) |
| `Webhook failed with status: 404` | Wrong URL path or deactivated n8n workflow |
| `Webhook failed with status: 429` | Receiver rate limiting; raise `interval_secs` or batch on the receiver |
| `Failed to send webhook: ... operation timed out` | Receiver took over 10 seconds to respond; respond 2xx immediately, process async |
| `Failed to send webhook: ... certificate ...` | Receiver TLS problem; poly enforces valid certificates and will not skip verification |

Timeout is 10 seconds. Delivery is fire-and-forget: a failed POST is logged and the alert is still saved to the database (see `poly-database` skill), so nothing is lost.

n8n recipe: create a Workflow with a Webhook trigger node, method POST, path e.g. `/webhook/whale-alerts`, activate the workflow, set `webhook_url` to the production URL shown in the node, then `poly test-webhook`.
