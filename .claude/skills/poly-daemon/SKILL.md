---
name: poly-daemon
description: Use when running poly watch as a background service or daemon via systemd, creating or managing a poly.service unit, viewing service logs, applying config changes to a running watcher, or updating an installed poly.
---

# Run poly watch as a systemd service

`poly watch` is a long-running process: it polls, streams, and sends alerts until stopped. Under systemd it survives logouts and reboots. Configure poly first (see `poly-setup` skill), and prove it works in the foreground before daemonizing:

```bash
timeout 15 poly watch
```

Resolve the absolute binary path; ExecStart requires it:

```bash
readlink -f "$(command -v poly)"   # e.g. /home/USER/Work/polymaster/target/release/poly
```

## Option A: system-wide service

```bash
sudo tee /etc/systemd/system/poly.service > /dev/null << EOF
[Unit]
Description=Polymaster whale watcher
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=$USER
ExecStart=/absolute/path/to/poly watch
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl daemon-reload
sudo systemctl enable --now poly.service
```

Replace the ExecStart path with the resolved one. Threshold and interval come from the config file; command-line overrides (e.g. `poly watch -t 50000`) also work in ExecStart but keeping settings in the config avoids drift between manual and service runs.

## Option B: user service (no sudo)

```bash
mkdir -p ~/.config/systemd/user
# write the same unit content to ~/.config/systemd/user/poly.service
# (drop the User= line, keep the rest)
systemctl --user daemon-reload
systemctl --user enable --now poly.service
loginctl enable-linger    # keeps it running when you log out
```

## Operations

```bash
systemctl status poly.service          # add --user for option B
journalctl -u poly.service -f          # live logs (--user -u for option B)
journalctl -u poly.service -n 100      # last 100 lines
sudo systemctl restart poly.service
sudo systemctl stop poly.service
```

Important behaviors:

- The config is read once at startup. After editing `config.json` (threshold, webhook, keys, anything), restart the service or the change will not apply.
- The process self-heals transient errors: Kalshi WebSocket drops reconnect with backoff (2s up to 60s), and without Kalshi API keys it falls back to HTTP polling. `Restart=always` is only a safety net.
- Kalshi WebSocket 401 messages mean the configured keys are invalid; the watcher keeps running on HTTP polling meanwhile.
- Alerts are logged to stderr and the journal even when the webhook fails; nothing is lost (failed webhook posts are logged and the alert is still saved to the database).

## Updating

```bash
cd /path/to/polymaster
git pull
cargo build --release
systemctl restart poly.service     # --user variant: systemctl --user restart poly
```

A symlink install picks up the rebuilt binary automatically; a `cargo install` setup must re-run `cargo install --path .` first.
