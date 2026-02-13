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

Polymaster sends HTTP POST requests with enriched JSON payloads. See [`docs/WEBHOOK_REFERENCE.md`](docs/WEBHOOK_REFERENCE.md) for the complete schema.

```json
{
  "platform": "Polymarket",
  "alert_type": "WHALE_ENTRY",
  "action": "BUY",
  "value": 50000.0,
  "price": 0.75,
  "size": 66666.67,
  "timestamp": "2026-01-09T06:00:00Z",
  "market_title": "Will Bitcoin reach 100k by end of 2026?",
  "outcome": "Yes",
  "wallet_id": "0x1234...5678",
  "wallet_activity": {
    "transactions_last_hour": 3,
    "transactions_last_day": 5,
    "total_value_hour": 150000.0,
    "total_value_day": 250000.0,
    "is_repeat_actor": true,
    "is_heavy_actor": true
  },
  "whale_profile": {
    "portfolio_value": 2340000.0,
    "leaderboard_rank": 45,
    "leaderboard_profit": 890000.0,
    "win_rate": 0.73,
    "positions_count": 12,
    "markets_traded": 195
  },
  "market_context": {
    "yes_price": 0.65,
    "no_price": 0.35,
    "spread": 0.02,
    "volume_24h": 450000.0,
    "open_interest": 2100000.0,
    "price_change_24h": 3.2,
    "liquidity": 180000.0,
    "tags": ["crypto", "bitcoin"]
  },
  "order_book": {
    "best_bid": 0.64,
    "best_ask": 0.66,
    "bid_depth_10pct": 45000.0,
    "ask_depth_10pct": 38000.0,
    "bid_levels": 12,
    "ask_levels": 9
  },
  "top_holders": {
    "holders": [
      { "wallet": "0x742d...bEb", "shares": 150000, "value": 97500 }
    ],
    "total_shares": 1250000
  }
}
```

### Key Fields

| Field | Description |
|-------|-------------|
| `platform` | "Polymarket" or "Kalshi" |
| `alert_type` | "WHALE_ENTRY" or "WHALE_EXIT" |
| `action` / `value` / `price` | Trade direction, USD value, contract price |
| `wallet_activity` | Transaction counts, volume, repeat/heavy actor flags |
| `whale_profile` | Portfolio value, leaderboard rank, win rate, positions (Polymarket only) |
| `market_context` | YES/NO odds, spread, volume, open interest, tags |
| `order_book` | Best bid/ask, depth, level counts |
| `top_holders` | Top holders with share counts (Polymarket only) |

See [`docs/WEBHOOK_REFERENCE.md`](docs/WEBHOOK_REFERENCE.md) for full field descriptions, n8n templates, and filter examples.

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

#### n8n Workflow

n8n is a self-hosted workflow automation platform. Use the Webhook node to receive alerts and process them.

**Step 1**: Create a new workflow in n8n

**Step 2**: Add a Webhook node
- Set "HTTP Method" to POST
- Set "Path" to something like `whale-alerts`
- Copy the webhook URL (e.g., `https://your-n8n-instance.com/webhook/whale-alerts`)

**Step 3**: Configure polymaster with the webhook URL
```bash
wwatcher setup
# Enter: https://your-n8n-instance.com/webhook/whale-alerts
```

**Step 4**: Add processing nodes to your workflow

Example n8n workflow structure:

```
Webhook (Receive alert)
  |
  v
Function (Process data)
  |
  +---> [IF: is_heavy_actor = true]
  |       |
  |       v
  |     Send urgent notification (Telegram/Discord/Email)
  |
  +---> [IF: value > 100000]
  |       |
  |       v
  |     Log to database (PostgreSQL/MongoDB)
  |
  +---> Always: Send to Slack/Discord
```

**Function Node Example** (to format the message):

```javascript
const alert = $input.item.json;

const message = `
WHALE ALERT: ${alert.alert_type}
Platform: ${alert.platform}
Market: ${alert.market_title}
Action: ${alert.action} ${alert.outcome}
Value: $${alert.value.toLocaleString()}
Price: ${(alert.price * 100).toFixed(1)}%
${alert.wallet_activity.is_heavy_actor ? '⚠️ HEAVY ACTOR DETECTED' : ''}
`;

return {
  json: {
    message: message,
    alert: alert,
    priority: alert.wallet_activity.is_heavy_actor ? 'urgent' : 'normal',
    value_usd: alert.value
  }
};
```

**Conditional Routing** (IF node):
- Condition 1: `{{$json.alert.wallet_activity.is_heavy_actor}} === true`
- Condition 2: `{{$json.alert.value}} > 100000`

**Common n8n integrations**:
- Telegram node: Send alerts to Telegram channel
- Discord node: Post to Discord server
- Email node: Send email notifications
- PostgreSQL/MongoDB node: Store alerts in database
- HTTP Request node: Forward to other services
- Code node: Custom processing logic

#### ntfy Integration

ntfy is a simple pub/sub notification service. Forward alerts to ntfy topics for mobile/desktop notifications.

**Direct Integration** (requires middleware):

Create a simple Node.js bridge to convert webhook to ntfy format:

```javascript
const express = require('express');
const axios = require('axios');
const app = express();
app.use(express.json());

const NTFY_TOPIC = 'your-whale-alerts-topic';
const NTFY_SERVER = 'https://ntfy.sh'; // or your self-hosted server

app.post('/webhook/whale-alerts', async (req, res) => {
  const alert = req.body;

  // Format priority based on conditions
  let priority = 'default';
  if (alert.wallet_activity.is_heavy_actor) {
    priority = 'urgent';
  } else if (alert.value > 100000) {
    priority = 'high';
  }

  // Determine tags
  const tags = [alert.platform.toLowerCase()];
  if (alert.action === 'BUY') tags.push('chart_with_upwards_trend');
  if (alert.action === 'SELL') tags.push('chart_with_downwards_trend');
  if (alert.wallet_activity.is_heavy_actor) tags.push('warning');

  // Create title and message
  const title = `${alert.alert_type} - ${alert.platform}`;
  const message = `${alert.action} ${alert.outcome}
Market: ${alert.market_title}
Value: $${alert.value.toLocaleString()}
Price: ${(alert.price * 100).toFixed(1)}%
${alert.wallet_activity.is_heavy_actor ? '\n⚠️ Heavy Actor: 5+ txns in 24h' : ''}`;

  // Send to ntfy
  await axios.post(`${NTFY_SERVER}/${NTFY_TOPIC}`, message, {
    headers: {
      'Title': title,
      'Priority': priority,
      'Tags': tags.join(','),
      'Actions': `view, Open Market, https://polymarket.com, clear=true`
    }
  });

  res.sendStatus(200);
});

app.listen(3000, () => {
  console.log('ntfy bridge running on port 3000');
  console.log(`Subscribe: ${NTFY_SERVER}/${NTFY_TOPIC}`);
});
```

**Usage**:

1. Run the bridge server:
```bash
node ntfy-bridge.js
```

2. Configure polymaster to send to the bridge:
```bash
wwatcher setup
# Enter: http://localhost:3000/webhook/whale-alerts
```

3. Subscribe to notifications:
```bash
# Mobile app: Subscribe to "your-whale-alerts-topic"
# Desktop: ntfy subscribe your-whale-alerts-topic
# Web: https://ntfy.sh/your-whale-alerts-topic
```

**n8n + ntfy Workflow**:

You can also use n8n to forward alerts to ntfy:

1. Webhook node receives alert from polymaster
2. Function node formats the message
3. HTTP Request node posts to ntfy

HTTP Request node configuration:
- Method: POST
- URL: `https://ntfy.sh/your-whale-alerts-topic`
- Headers:
  - `Title`: `{{$json.alert.alert_type}} - {{$json.alert.platform}}`
  - `Priority`: `{{$json.priority}}`
  - `Tags`: `whale,{{$json.alert.platform}}`
- Body: `{{$json.message}}`

## Example Alert Output

```
[ALERT] LARGE TRANSACTION DETECTED - Polymarket
======================================================================
Question:   Will Bitcoin reach 100k by end of 2026?
Position:   BUYING 'Yes' shares
Prediction: Market believes 'Yes' has 65.0% chance

TRANSACTION DETAILS
Amount:     $50,000.00
Contracts:  76923.08 @ $0.6500 each
Action:     BUY shares
Timestamp:  2026-02-13T18:00:00Z

[WALLET ACTIVITY]
Wallet:   0x742d35...f0bEb
Txns (1h):  2
Txns (24h): 5
Volume (24h): $380,000.00
Status: HEAVY ACTOR (5+ transactions in 24h)

[MARKET CONTEXT]
Odds:          YES 65.0% | NO 35.0%
Spread:        $0.02 (tight)
24h Volume:    $450,000
Open Interest: $2,100,000
Tags:          crypto, bitcoin

[WHALE PROFILE]
Leaderboard:  #45 (TOP 50)
Profit:       +$890,000
Portfolio:    $2,340,000
Win Rate:     73.0%

[ORDER BOOK]
Best Bid:   $0.6400  |  Best Ask: $0.6600
Bid Depth:  $45,000 (12 levels)  |  Ask Depth: $38,000 (9 levels)
Imbalance:  54% bid / 46% ask
======================================================================
```

## AI Agent Integration

wwatcher includes an AI integration layer that scores whale alerts, runs context-aware research, and delivers structured signals (bullish/bearish, confidence, key factors).

### Quick Setup

```bash
cd integration
npm install
npm run build

# Configure API keys
cat > .env << EOF
PERPLEXITY_API_KEY=your-key
RAPIDAPI_KEY=your-key
EOF

# Test
node dist/cli.js status
```

### CLI Commands

```bash
cd integration

# Health check
node dist/cli.js status

# Query alerts (enriched with whale profile, order book, tags)
node dist/cli.js alerts --limit=10 --min=50000

# Score an alert — returns tier (high/medium/low) + factors
node dist/cli.js score '<alert_json>'

# Context-aware research — scores alert, targeted queries, structured signal
node dist/cli.js research "Bitcoin above 100k" --context='<alert_json>'

# Generic research (no alert context)
node dist/cli.js research "Bitcoin above 100k" --category=crypto

# RapidAPI market data only
node dist/cli.js fetch "Bitcoin price above 100k"

# Show user preference schema
node dist/cli.js preferences show
```

### OpenClaw Skill

```bash
mkdir -p ~/.openclaw/skills/wwatcher-ai
cp integration/skill/SKILL.md ~/.openclaw/skills/wwatcher-ai/SKILL.md
```

The skill auto-triggers on whale alerts and supports user preferences like "only 60%+ win rate" or "skip under $100k".

### MCP Server

```bash
npm run start:mcp
```

See [`integration/README.md`](./integration/README.md) for full details including scoring, structured signals, and prediction market data.

---

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
