---
name: poly-database
description: Use when working with poly's stored alert data, locating or querying the SQLite alert history, exporting results, configuring retention, backing up or resetting the database, or answering where alerts are saved.
---

# Alert history database

Every alert poly fires is also stored locally in SQLite. This skill covers the location, schema, retention, and common queries. No setup is needed; the database is created on the first run of any `poly` command.

## 1. Location

- Linux: `~/.config/poly/poly.db`
- macOS: `~/Library/Application Support/poly/poly.db`
- Windows: `%APPDATA%\poly\poly.db`

`poly status` prints the exact location and the current alert count. The engine runs in WAL mode with a 5s busy timeout, so concurrent reads (your queries) never block the watcher.

## 2. Schema

Table `alerts`, one row per fired alert:

| Column | Type | Notes |
|--------|------|-------|
| `id` | integer PK | Insertion order |
| `platform` | text | `"Polymarket"` or `"Kalshi"` |
| `alert_type` | text | `"WHALE_ENTRY"` (buy) or `"WHALE_EXIT"` (sell) |
| `action` | text | `"BUY"` or `"SELL"` |
| `category`, `subcategory` | text | Native Kalshi category or Polymarket tag when known |
| `value` | real | Trade value in USD |
| `price` | real | Outcome price 0.0-1.0 |
| `size` | real | Contract count |
| `market_title`, `market_id`, `outcome` | text | May be null on lookup failure |
| `wallet_hash` | text | SHA-256 of wallet_id; one-way, cannot be reversed |
| `wallet_id` | text | Plaintext wallet, Polymarket only; Kalshi exposes none |
| `timestamp` | text | RFC3339 trade time |
| `market_context`, `wallet_activity` | text | JSON blobs, same shapes as the webhook payload |
| `created_at` | integer | Unix seconds; used by retention pruning |

Table `wallet_memory`: 12-hour rolling memory of recent wallet trades (columns `wallet_hash`, `wallet_id`, `market_title`, `market_id`, `outcome`, `action`, `value`, `price`, `platform`, `category`, `seen_at`). It powers "returning whale" detection and is pruned automatically; do not treat it as permanent history.

Table `metadata`: key/value, holds `schema_version` and `created_at`.

## 3. Query recipes

```bash
DB=~/.config/poly/poly.db

# 10 most recent alerts
sqlite3 "$DB" "SELECT id, platform, alert_type, action, value, market_title FROM alerts ORDER BY id DESC LIMIT 10;"

# Alert counts by platform
sqlite3 "$DB" "SELECT platform, COUNT(*) FROM alerts GROUP BY platform;"

# Alerts per day over the last 7 days
sqlite3 "$DB" "SELECT date(created_at, 'unixepoch') day, COUNT(*) FROM alerts WHERE created_at > strftime('%s','now') - 604800 GROUP BY day ORDER BY day;"

# Largest alerts on record
sqlite3 "$DB" "SELECT value, platform, action, market_title, timestamp FROM alerts ORDER BY value DESC LIMIT 10;"

# Full activity trail for one wallet (query wallet_id, not wallet_hash)
sqlite3 "$DB" "SELECT timestamp, action, value, market_title FROM alerts WHERE wallet_id = '0x...' ORDER BY id DESC;"
```

For agent-friendly output use the built-in export instead of hand-writing SQL for the common shapes:

```bash
poly history --json                 # last 20 as a JSON array
poly history --json -l 100 -p kalshi
```

Each JSON object has: `platform`, `alert_type`, `action`, `value`, `price`, `size`, `timestamp`, `market_title`, `outcome`, and when present `wallet_id`, `wallet_activity` (object), `market_context` (object).

## 4. Retention

`history_retention_days` in the config (default 30) controls pruning: while `poly watch` runs, a cleanup pass every ~5 minutes deletes alerts older than the window. `0` keeps alerts forever. Pruning never touches `wallet_memory` beyond its fixed 12-hour window.

## 5. Backup, reset, migration

```bash
# Safe while the watcher is running (WAL):
sqlite3 ~/.config/poly/poly.db "VACUUM INTO '/path/to/poly-backup.db';"
```

Alternatively stop the service and copy the file (include `poly.db-wal` and `poly.db-shm` if present). Deleting the database file is also safe: the schema is recreated empty on the next start.

Startup migrations are automatic and idempotent: an old `~/.config/wwatcher` directory is renamed to `poly` (with `wwatcher.db` becoming `poly.db`), and a legacy `alert_history.jsonl` is imported into SQLite and backed up as `alert_history.jsonl.bak`. If an old install seems to have lost its history, check for those backup files.
