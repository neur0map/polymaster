---
name: poly-setup
description: Use when installing, building, configuring, or tuning poly (polymaster), running first-time setup, or changing any setting for a user, including platforms, categories, threshold, interval, odds filters, retention, Kalshi API keys, or webhook.
---

# Install, configure, and control poly

poly is a CLI that monitors Polymarket and Kalshi for large ("whale") transactions and fires alerts to the terminal, a webhook, local sound, and a SQLite history. Everything an agent needs to install, configure, verify, and adjust it is in this file. Companion skills: `poly-webhook`, `poly-daemon`, `poly-database`.

## 1. Build and install

```bash
git clone https://github.com/neur0map/polymaster.git   # skip if repo already present
cd polymaster
cargo build --release
```

Install one of two ways:

```bash
# Option A: symlink (recommended for a repo checkout, easy updates via rebuild)
ln -sf "$(pwd)/target/release/poly" ~/.local/bin/poly

# Option B: cargo install (puts it at ~/.cargo/bin/poly)
cargo install --path .
```

Verify: `command -v poly && poly --help` shows subcommands `watch`, `history`, `setup`, `status`, `test-sound`, `test-webhook`.

Rust toolchain is the only build prerequisite; TLS is handled by the crate's bundled native-tls.

## 2. Configuration file

Location (created automatically on first run of any `poly` command):

- Linux: `~/.config/poly/config.json`
- macOS: `~/Library/Application Support/poly/config.json`
- Windows: `%APPDATA%\poly\config.json`

Full reference, exact JSON keys:

| Key | Type | Default | Meaning |
|-----|------|---------|---------|
| `threshold` | number (USD) | `25000` | Minimum trade value that triggers an alert |
| `interval_secs` | number, >= 1 | `5` | Polling interval in seconds |
| `platforms` | array | `["all"]` | `["all"]`, `["polymarket"]`, or `["kalshi"]` |
| `categories` | array | `["all"]` | `["all"]` or specific keys like `"sports:nba"`, `"crypto:all"` |
| `max_odds` | number 0.0-1.0 | `0.95` | Skip alerts when YES or NO price exceeds this (near-certainties) |
| `min_spread` | number 0.0-1.0 | `0.0` | Skip markets with spread below this (dead markets); 0 disables |
| `history_retention_days` | number | `30` | Days to keep alerts in the DB; `0` keeps forever |
| `webhook_url` | string or null | `null` | POST target for every alert; see `poly-webhook` skill |
| `kalshi_api_key_id` | string or null | `null` | Optional Kalshi credential |
| `kalshi_private_key` | string or null | `null` | Optional Kalshi credential (PEM) |

Notes:
- Partial JSON is safe. Missing keys fall back to defaults; you do not need to write all keys.
- Polymarket needs no API key. Kalshi REST polling also works without keys, but the Kalshi WebSocket (real-time trades) requires them. Generate keys at https://kalshi.com/profile/api-keys and store the key ID and the PEM private key text.
- Kalshi keys also unlock authenticated trading: order placement, balance, positions (see the `poly-trading` skill). Use a key with the `write::trade` scope if the user wants trading.
- The config is read once at startup. A running `poly watch` does not hot-reload; restart it after edits (see `poly-daemon` skill if running under systemd).

## 3. Configure for the user's needs

Preferred agent method: edit `config.json` directly, then verify.

```bash
poly status        # confirm the new values are picked up
```

Interactive method (for humans, or agents piping stdin): `poly setup` is a menu editor. Options are `[1]`-`[9]` for settings, `[t]` test notifications, `[d]` reset to defaults (asks y/N), `[q]` done. Every change saves immediately; Enter at any prompt keeps the current value. Piping works: `printf '3\n50000\nq\n' | poly setup` sets the threshold to $50,000.

Tuning guide, map user complaints to settings:

| User says | Change |
|-----------|--------|
| "Too many alerts" | Raise `threshold`; optionally lower `max_odds` (e.g. 0.90) to cut near-settled markets; raise `min_spread` (e.g. 0.02) to skip dead ones |
| "I'm missing alerts" | Lower `threshold`; set `max_odds` to `1` and `min_spread` to `0` to disable filtering |
| "Only care about X" | Set `platforms` and `categories` accordingly |
| "Want real-time Kalshi trades" | Configure Kalshi API keys (without them, Kalshi uses ~interval HTTP polling) |
| "Want alerts in Discord/n8n/Telegram" | Set `webhook_url`, then `poly test-webhook`; see `poly-webhook` skill |

One-shot overrides without touching config: `poly watch --threshold 50000 --interval 10`.

## 4. Verify

```bash
poly status                     # shows effective config, alert count, DB location
poly test-sound                 # audible alert check
poly test-webhook               # sends two fake alerts (BUY + SELL) to the webhook
timeout 15 poly watch           # smoke test; Ctrl-C or timeout kills it safely
```

A healthy `poly watch` prints a disclaimer, the effective threshold/interval/platforms, and either `Kalshi WS: Connecting (authenticated)...` or `Kalshi WS: skipped — no API keys configured; using HTTP polling instead.` Skipping is normal without keys and is not an error.

## 5. Common pitfalls

- `Kalshi WS ... 401 Unauthorized`: keys are configured but invalid, or were entered with stray whitespace. Re-enter via option `[8]` in `poly setup` or edit the config.
- No alerts after hours of watching: threshold is likely far above current trade sizes. Check what markets actually trade (`poly history` or lower `threshold` temporarily to 100 and watch for a minute).
- Edited config but behavior unchanged: the running process is still using the old config. Restart it.
- Do not put shell escapes or quotes around the PEM key inside config.json beyond standard JSON string escaping; the key text goes in as one JSON string with `\n` line breaks.
