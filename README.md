# Whale Watcher

A Rust CLI tool that monitors large transactions on Polymarket and Kalshi prediction markets. Real-time alerts for significant market activity with built-in anomaly detection.

**Repository**: https://github.com/neur0map/polymaster

## ⚠️ DISCLAIMER

**This tool is for informational and research purposes only.**

I do not condone gambling or speculative trading. Use this data solely for informed decision-making and market analysis. Trade responsibly and within your means.

## Features

- **Real-time monitoring** of Polymarket and Kalshi transactions
- **Audio alerts** - instant beep notification (triple beep for repeat actors)
- **Wallet tracking** - detects repeated large transactions from same wallet
  - Elevated alerts for repeat actors (2+ txns in 1 hour)
  - High priority alerts for heavy actors (5+ txns in 24 hours)
  - Tracks volume and transaction frequency per wallet
- **Customizable alerts** for transactions above a threshold (default: $25,000)
- **Anomaly detection** - identifies unusual trading patterns including:
  - Extreme confidence bets (>95% or <5% probability)
  - Contrarian positions on unlikely outcomes
  - Exceptionally large position sizes (>100k contracts)
  - Major capital deployment (>$100k)
  - Possible information asymmetry indicators
- **Persistent configuration** - set up once, no need for exports
- **Professional CLI output** with clear formatting
- **No API keys required** for basic functionality (public data access)
- **Fast and efficient** - built with Rust

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

For authenticated Kalshi access (not required):

```bash
wwatcher setup  # Configure optional API credentials
wwatcher status # View current configuration
```

## API Information

### Polymarket

- **Public API**: `https://data-api.polymarket.com`
- **No authentication required** for public trade data
- **Documentation**: https://docs.polymarket.com

The tool uses the Polymarket Data API to fetch recent trades. This is a public endpoint that provides:
- Recent trade activity
- Market data
- Price information

### Kalshi

- **Public API**: `https://api.elections.kalshi.com/trade-api/v2`
- **Authentication**: Optional (public endpoints available)
- **Documentation**: https://docs.kalshi.com

For public trade data, no API key is needed. If you want access to your personal orders and fills, you can:
1. Create an account at https://kalshi.com
2. Generate API credentials at https://kalshi.com/profile/api-keys
3. Run `wwatcher setup` and enter your credentials

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

### `wwatcher watch`

Start monitoring for large transactions.

**Options:**
- `-t, --threshold <AMOUNT>` - Minimum transaction size in USD (default: 25000)
- `-i, --interval <SECONDS>` - Polling interval in seconds (default: 5)

**Examples:**
```bash
wwatcher watch                        # Default: $25k threshold, 5s interval
wwatcher watch -t 50000               # $50k threshold
wwatcher watch -i 30                  # Check every 30 seconds
wwatcher watch -t 100000 -i 60        # $100k threshold, check every minute
```

### `wwatcher setup`

Interactive setup wizard to configure API credentials.

```bash
wwatcher setup
```

### `wwatcher status`

Show current configuration status.

```bash
wwatcher status
```

## Configuration File

Configuration is stored at:
- **macOS/Linux**: `~/.config/wwatcher/config.json`
- **Windows**: `%APPDATA%\wwatcher\config.json`

Example `config.json`:
```json
{
  "kalshi_api_key_id": "your-key-id",
  "kalshi_private_key": "your-private-key"
}
```

## Development

Built with:
- [Rust](https://www.rust-lang.org/) - Systems programming language
- [Tokio](https://tokio.rs/) - Async runtime
- [Reqwest](https://github.com/seanmonstar/reqwest) - HTTP client
- [Clap](https://github.com/clap-rs/clap) - CLI argument parsing
- [Serde](https://serde.rs/) - JSON serialization

## Troubleshooting

### "No configuration found" warning

This is normal! The tool works without configuration using public APIs. Run `wwatcher setup` only if you want to add Kalshi authentication.

### API errors

- **Polymarket**: Public endpoint, should work without issues
- **Kalshi**: Public endpoint works without auth, but rate limits may apply

### Rate Limiting

If you're getting rate limited:
- Increase the `--interval` to poll less frequently
- For Kalshi: Add API credentials via `wwatcher setup`

## Documentation

- [QUICKSTART.md](QUICKSTART.md) - Quick start guide
- [ANOMALY_DETECTION.md](ANOMALY_DETECTION.md) - Detailed anomaly detection patterns and use cases
- [API_REFERENCE.md](API_REFERENCE.md) - API documentation and technical details

## License

This tool is for educational and monitoring purposes. Please review the terms of service for Polymarket and Kalshi APIs.

