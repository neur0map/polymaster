# Quick Start Guide

## DISCLAIMER

This tool is for informational and research purposes only. Use this data solely for informed decision-making and market analysis.

---

## Installation

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env

# Clone and build
git clone https://github.com/neur0map/polymaster.git
cd polymaster
cargo build --release
cargo install --path .
```

## Basic Usage

Start monitoring with default settings (monitors transactions over $25k, checks every 5 seconds):

```bash
wwatcher watch
```

Customize threshold and polling interval:

```bash
wwatcher watch --threshold 50000 --interval 30
```

View alert history:

```bash
wwatcher history                              # Last 20 alerts
wwatcher history --limit 50                   # Last 50 alerts
wwatcher history --platform polymarket        # Polymarket only
wwatcher history --json                       # Export as JSON
```

## Running as a System Service (Linux)

To run the watcher continuously as a background service:

### Step 1: Configure webhook (optional)

```bash
mkdir -p ~/.config/wwatcher
cat > ~/.config/wwatcher/config.json << 'EOF'
{
  "webhook_url": "https://your-webhook-url.com/webhook/polymaster"
}
EOF
```

### Step 2: Create systemd service file

```bash
sudo tee /etc/systemd/system/wwatcher.service > /dev/null << EOF
[Unit]
Description=Polymaster Whale Watcher
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=$USER
WorkingDirectory=$HOME
ExecStart=$HOME/.cargo/bin/wwatcher watch --threshold 28000 --interval 5
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
EOF
```

### Step 3: Start and enable service

```bash
sudo systemctl daemon-reload
sudo systemctl enable wwatcher.service
sudo systemctl start wwatcher.service
```

### Service Management Commands

```bash
# Check service status
sudo systemctl status wwatcher.service

# View live logs
sudo journalctl -u wwatcher.service -f

# Restart service
sudo systemctl restart wwatcher.service

# Stop service
sudo systemctl stop wwatcher.service
```

### Quick Update and Restart

```bash
cd ~/polymaster && git pull && cargo build --release && cargo install --path . && sudo systemctl restart wwatcher.service
```

## Command Reference

### wwatcher watch

Start monitoring for large transactions.

```bash
wwatcher watch [OPTIONS]
```

Options:
- `-t, --threshold <AMOUNT>` - Minimum transaction size in USD (default: 25000)
- `-i, --interval <SECONDS>` - Polling interval in seconds (default: 5)

Examples:
```bash
wwatcher watch                        # Default: $25k threshold, 5s interval
wwatcher watch -t 50000               # $50k threshold
wwatcher watch -i 30                  # Check every 30 seconds
wwatcher watch -t 100000 -i 60        # $100k threshold, check every minute
```

### wwatcher history

View saved alert history.

```bash
wwatcher history [OPTIONS]
```

Options:
- `-l, --limit <NUMBER>` - Number of alerts to show (default: 20)
- `-p, --platform <NAME>` - Filter by platform: polymarket, kalshi, or all (default: all)
- `--json` - Output as JSON

Examples:
```bash
wwatcher history                              # Show last 20 alerts
wwatcher history --limit 50                   # Show last 50 alerts
wwatcher history --platform polymarket        # Show only Polymarket alerts
wwatcher history --json                       # Export as JSON
```

Alerts are automatically saved to `~/.config/wwatcher/alert_history.jsonl`.

### wwatcher setup

Interactive setup wizard to configure API credentials and webhook URL.

```bash
wwatcher setup
```

### wwatcher status

Show current configuration status.

```bash
wwatcher status
```

## What It Monitors

- Polymarket and Kalshi transactions over your threshold (default $25k)
- Wallet activity and repeat actors
- Unusual trading patterns and anomalies
- Entry and exit positions

## Configuration

### Config File Location

- macOS/Linux: `~/.config/wwatcher/config.json`
- Windows: `%APPDATA%\wwatcher\config.json`

### Setup Wizard

Run the interactive setup to configure Kalshi API credentials or webhook URL:

```bash
wwatcher setup
```

### Manual Configuration

Create or edit the config file directly:

```json
{
  "kalshi_api_key_id": "your-key-id",
  "kalshi_private_key": "your-private-key",
  "webhook_url": "https://your-server.com/webhook/whale-alerts"
}
```

Note: Kalshi credentials are optional. They provide higher rate limits but are not required for basic monitoring. Currently, there is no functionality to view or place orders.

## Webhook Integration

Configure a webhook URL during setup to receive alerts at your custom server, Discord bot, Telegram bot, or automation platform.

### Setup Webhook

```bash
wwatcher setup
# Enter your webhook URL when prompted
```

Or manually edit config file at `~/.config/wwatcher/config.json`:

```json
{
  "webhook_url": "https://your-server.com/webhook/whale-alerts"
}
```

### Supported Platforms

- Custom HTTP servers (Node.js, Python, Go, etc.)
- Discord webhooks
- Slack webhooks
- Telegram bots
- n8n (self-hosted automation)
- Zapier (cloud automation)
- Make/Integromat

### Webhook Payload Format

Polymaster sends HTTP POST requests with this JSON structure:

```json
{
  "platform": "Polymarket",
  "alert_type": "WHALE_ENTRY",
  "action": "BUY",
  "value": 50000.0,
  "price": 0.75,
  "size": 66666.67,
  "timestamp": "2026-01-09T06:00:00Z",
  "market_title": "Will Trump win the 2024 Presidential Election?",
  "outcome": "Yes",
  "wallet_id": "0x1234567890abcdef1234567890abcdef12345678",
  "wallet_activity": {
    "transactions_last_hour": 3,
    "transactions_last_day": 5,
    "total_value_hour": 150000.0,
    "total_value_day": 250000.0,
    "is_repeat_actor": true,
    "is_heavy_actor": true
  }
}
```

### Field Descriptions

| Field | Type | Description |
|-------|------|-------------|
| `platform` | string | "Polymarket" or "Kalshi" |
| `alert_type` | string | "WHALE_ENTRY" or "WHALE_EXIT" |
| `action` | string | "BUY" or "SELL" |
| `value` | number | Transaction value in USD |
| `price` | number | Price per contract (0.0-1.0 representing probability) |
| `size` | number | Number of contracts traded |
| `timestamp` | string | ISO 8601 timestamp |
| `market_title` | string | Market question or title |
| `outcome` | string | Outcome traded (e.g., "Yes", "No", candidate name) |
| `wallet_id` | string | Wallet address or trader ID |
| `wallet_activity.transactions_last_hour` | number | Transactions in past hour |
| `wallet_activity.transactions_last_day` | number | Transactions in past 24 hours |
| `wallet_activity.total_value_hour` | number | Total USD volume in past hour |
| `wallet_activity.total_value_day` | number | Total USD volume in past 24 hours |
| `wallet_activity.is_repeat_actor` | boolean | true if 2+ transactions in 1 hour |
| `wallet_activity.is_heavy_actor` | boolean | true if 5+ transactions in 24 hours |

### Integration Examples

#### Node.js Express Server

```javascript
const express = require('express');
const app = express();
app.use(express.json());

app.post('/webhook/whale-alerts', (req, res) => {
  const alert = req.body;

  console.log(`[${alert.platform}] ${alert.action} ${alert.outcome}`);
  console.log(`Value: $${alert.value.toLocaleString()}`);
  console.log(`Market: ${alert.market_title}`);

  if (alert.wallet_activity.is_heavy_actor) {
    console.log('WARNING: Heavy actor detected!');
  }

  res.sendStatus(200);
});

app.listen(3000);
```

#### Python Flask Server

```python
from flask import Flask, request
app = Flask(__name__)

@app.route('/webhook/whale-alerts', methods=['POST'])
def whale_alert():
    alert = request.json

    print(f"[{alert['platform']}] {alert['action']} {alert['outcome']}")
    print(f"Value: ${alert['value']:,.2f}")
    print(f"Market: {alert['market_title']}")

    if alert['wallet_activity']['is_heavy_actor']:
        print("WARNING: Heavy actor detected!")

    return '', 200

if __name__ == '__main__':
    app.run(port=3000)
```

#### Discord Webhook

```javascript
const express = require('express');
const axios = require('axios');
const app = express();
app.use(express.json());

const DISCORD_WEBHOOK_URL = 'https://discord.com/api/webhooks/YOUR_WEBHOOK_ID/YOUR_TOKEN';

app.post('/webhook/whale-alerts', async (req, res) => {
  const data = req.body;

  const embed = {
    title: `${data.alert_type} - ${data.platform}`,
    description: data.market_title,
    fields: [
      { name: "Action", value: data.action, inline: true },
      { name: "Value", value: `$${data.value.toLocaleString()}`, inline: true },
      { name: "Outcome", value: data.outcome, inline: true },
      { name: "Price", value: `${(data.price * 100).toFixed(1)}%`, inline: true },
      { name: "Size", value: `${data.size.toLocaleString()} contracts`, inline: true },
      { name: "Wallet", value: `\`${data.wallet_id.substring(0, 10)}...\``, inline: true }
    ],
    color: data.action === "BUY" ? 0x00ff00 : 0xff0000,
    timestamp: data.timestamp
  };

  if (data.wallet_activity.is_heavy_actor) {
    embed.footer = { text: "Heavy Actor Alert: 5+ transactions in 24h" };
  } else if (data.wallet_activity.is_repeat_actor) {
    embed.footer = { text: "Repeat Actor: 2+ transactions in 1h" };
  }

  await axios.post(DISCORD_WEBHOOK_URL, { embeds: [embed] });
  res.sendStatus(200);
});

app.listen(3000);
```

## Example Alert Output

```
[ALERT] LARGE TRANSACTION DETECTED - Polymarket
======================================================================
Market:   Will Trump win the 2024 Presidential Election?
Outcome:  Yes
Value:    $45,250.00
Price:    $0.7500 (75.0%)
Size:     60333.33 contracts
Side:     BUY
Time:     2026-01-08T21:30:00Z

[ANOMALY INDICATORS]
  - High conviction in likely outcome

Asset ID: 65396714035221124737...
======================================================================
```

## Troubleshooting

### Rate limit errors
Increase polling interval:
```bash
wwatcher watch --interval 60
```

### No transactions detected
Lower the threshold:
```bash
wwatcher watch --threshold 10000
```

### Service not starting
Check logs:
```bash
sudo journalctl -u wwatcher.service -n 50
```

### Update service configuration
Edit threshold or interval in service file, then reload:
```bash
sudo systemctl daemon-reload
sudo systemctl restart wwatcher.service
```
