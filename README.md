# Whale Watcher

A Rust CLI tool that monitors large transactions on Polymarket and Kalshi prediction markets. Real-time alerts for significant market activity with built-in anomaly detection.

Repository: https://github.com/neur0map/polymaster

## ⚠️ DISCLAIMER
This tool is for informational and research purposes only. Use this data solely for informed decision-making and market analysis.

## Features

- Real-time monitoring of Polymarket and Kalshi transactions
- Audio alerts with triple beep for repeat actors
- Wallet tracking detects repeated large transactions from same wallet
  - Elevated alerts for repeat actors (2+ txns in 1 hour)
  - High priority alerts for heavy actors (5+ txns in 24 hours)
  - Tracks volume and transaction frequency per wallet
- Customizable alerts for transactions above a threshold (default $25,000)
- Anomaly detection identifies unusual trading patterns:
  - Extreme confidence bets (over 95% or under 5% probability)
  - Contrarian positions on unlikely outcomes
  - Exceptionally large position sizes (over 100k contracts)
  - Major capital deployment (over $100k)
  - Possible information asymmetry indicators
- Webhook notifications send alerts to n8n, Zapier, Make, or any webhook endpoint
- Exit detection with special alerts when whales are selling or exiting positions
- Persistent configuration saves settings between runs
- Clean CLI output with clear formatting
- No API keys required for basic functionality (public data access)
- Fast and efficient, built with Rust

## Installation

### From Source

```bash
# Clone or navigate to the project
cd polymaster

# Build the project
cargo build --release

# Install to system (optional)
cargo install --path .
```

The binary will be available at `target/release/wwatcher` or in your cargo bin directory.

## Quick Start

```bash
# 1. Install
cargo install --path .

# 2. Start monitoring (no setup required)
wwatcher watch

# 3. Optional: Customize threshold or interval
wwatcher watch --threshold 50000 --interval 10
```

That's it! The tool monitors Polymarket and Kalshi for transactions over $25k.

### Optional Setup

For authenticated Kalshi access or webhook notifications:

```bash
wwatcher setup  # Configure optional API credentials and webhook
wwatcher status # View current configuration
```

### Webhook Integration

During setup, you can configure a webhook URL to receive alerts. This works with:

- n8n (self-hosted automation platform)
- Zapier (cloud automation service)
- Make, formerly Integromat (automation platform)
- Any service that accepts HTTP POST requests with JSON payloads

Webhook payload example:
```json
{
  "platform": "Polymarket",
  "alert_type": "WHALE_ENTRY" or "WHALE_EXIT",
  "action": "BUY" or "SELL",
  "value": 50000.0,
  "price": 0.75,
  "size": 66666.67,
  "timestamp": "2026-01-09T06:00:00Z",
  "market_title": "Will X happen?",
  "outcome": "Yes",
  "wallet_id": "0x1234...",
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

## API Information

### Polymarket

Public API: https://data-api.polymarket.com

No authentication required for public trade data. The tool uses the Polymarket Data API to fetch:

- Recent trade activity
- Market data
- Price information

Documentation: https://docs.polymarket.com

### Kalshi

Public API: https://api.elections.kalshi.com/trade-api/v2

Authentication is optional. Public endpoints are available without an API key. For access to personal orders and fills:

1. Create an account at https://kalshi.com
2. Generate API credentials at https://kalshi.com/profile/api-keys
3. Run `wwatcher setup` and enter your credentials

Documentation: https://docs.kalshi.com

## ⚠️ DISCLAIMER
 Currently, there is no code in place to view your order or place orders.
 current setup only allows for tracking transactions but I do plan to improve the application down the road with n8n.

## Alert Example

When a whale transaction is detected, you'll see:

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

## Command Reference

### wwatcher watch

Start monitoring for large transactions.

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

### wwatcher setup

Interactive setup wizard to configure API credentials.

```bash
wwatcher setup
```

### wwatcher status

Show current configuration status.

```bash
wwatcher status
```

## Configuration File

Configuration is stored at:

- macOS/Linux: `~/.config/wwatcher/config.json`
- Windows: `%APPDATA%\wwatcher\config.json`

Example config.json:
 - Kalshi credentials are not needed; they are only used to view your orders or place orders, but there is no functionality for this at the moment.
```json
{
  "kalshi_api_key_id": "your-key-id",
  "kalshi_private_key": "your-private-key",
  "webhook_url": "https://your-n8n-instance.com/webhook/whale-alerts"
}
```

## Development

Built with:

- Rust - Systems programming language
- Tokio - Async runtime
- Reqwest - HTTP client
- Clap - CLI argument parsing
- Serde - JSON serialization

## Troubleshooting

### No configuration found warning

This is normal. The tool works without configuration using public APIs. Run `wwatcher setup` only if you want to add Kalshi authentication.

### API errors

- Polymarket: Public endpoint should work without issues
- Kalshi: Public endpoint works without auth, but rate limits may apply

### Rate Limiting

If you're getting rate limited:

- Increase the `--interval` to poll less frequently
- For Kalshi: Add API credentials via `wwatcher setup`

## License

This tool is for educational and monitoring purposes. Review the terms of service for Polymarket and Kalshi APIs.

