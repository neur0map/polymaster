# Quick Start Guide

## ⚠️ DISCLAIMER

**This tool is for informational and research purposes only.** I do not condone gambling or speculative trading. Use this data solely for informed decision-making and market analysis. Trade responsibly and within your means.

---

Get started with Whale Watcher in under 2 minutes.

## Installation & Usage

```bash
# Build and install
cargo install --path .

# Start monitoring (no setup required)
wwatcher watch

# Customize threshold or interval
wwatcher watch -t 50000 -i 10
```

### What it does:
- Monitors Polymarket & Kalshi for transactions over $25k (default)
- Alerts with audio notification when whales are detected
- Tracks wallet activity and repeat actors
- Identifies unusual trading patterns automatically
- No API keys required (uses public endpoints)

### Optional: Authenticated Access

```bash
wwatcher setup  # Configure Kalshi credentials for higher rate limits
```

## API Key Information

### Polymarket
- **No API key needed**
- Uses public data endpoint: `https://data-api.polymarket.com`
- Works out of the box

### Kalshi
- **Public endpoint available** (no auth needed)
- **Optional auth**: For higher rate limits
  - Create account: https://kalshi.com
  - Generate keys: https://kalshi.com/profile/api-keys
  - Add via: `./target/release/wwatcher setup`

## Example Output

When a whale is detected:

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

## Pro Tips

1. **Lower thresholds** for more alerts: `-t 10000`
2. **Slower polling** to reduce API calls: `-i 30`
3. **Install system-wide**: `cargo install --path .`
4. **Run in background**: `nohup wwatcher watch > whales.log 2>&1 &`
5. **Anomaly detection**: Automatically identifies unusual trading patterns

## Troubleshooting

**Q: I get rate limit errors**  
A: Increase the interval: `wwatcher watch -i 60`

**Q: No whales detected**  
A: Markets might be quiet. Try lowering threshold: `-t 10000`

**Q: API errors**  
A: Both APIs are public and should work. Check your internet connection.

## Next Steps

- Read the full [README.md](README.md) for detailed documentation
- Explore command options: `wwatcher watch --help`
- Check configuration: `wwatcher status`

Happy whale watching!
