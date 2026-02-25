# Crypto Market Research Context

## Role
Specialized research guidance for cryptocurrency prediction markets on Kalshi. Load this context when the researcher agent is working on a crypto-category market.

## Key Data Sources

### On-Chain Metrics
- **Funding rates**: Extreme positive funding (>0.05%) often precedes pullbacks. Negative funding can signal bottoming.
- **Exchange flows**: Large net inflows to exchanges = selling pressure. Net outflows = accumulation.
- **Whale wallets**: Track top 100 BTC/ETH wallets for large movements via on-chain explorers.
- **Open interest**: Rising OI + rising price = strong trend. Rising OI + falling price = potential liquidation cascade.

### Macro Calendar
- **FOMC meetings**: Rate decisions move crypto significantly. Check Fed calendar.
- **CPI/PPI releases**: Inflation data affects risk-on/risk-off sentiment.
- **Jobs reports**: Strong jobs = hawkish Fed = crypto headwind.
- **Treasury auctions**: Large issuance can drain liquidity from risk assets.

**RULE**: Do not enter crypto positions within 4 hours of FOMC, CPI, or jobs releases unless the market has already priced in the event.

### ETF Data
- **BTC/ETH spot ETF flows**: Daily inflow/outflow data from ETF issuers.
- **Premium/discount**: ETF trading above/below NAV signals retail sentiment.
- **AUM growth**: Sustained inflows = institutional demand.

### Technical Levels
- **Key round numbers**: BTC psychological levels ($90k, $95k, $100k, $105k).
- **Previous ATH/ATL**: Markets often stall at historical extremes.
- **200-day MA**: Widely watched — price below = bearish, above = bullish.

## Research Queries (Perplexity)

When researching a crypto market, generate these queries:

1. **Price action**: `"{asset} price prediction {target} by {expiry_date} analyst forecasts"`
2. **On-chain**: `"{asset} whale activity exchange flows {current_month} {current_year}"`
3. **Macro**: `"crypto market macro outlook {current_month} {current_year} FOMC CPI impact"`
4. **Catalysts**: `"{asset} upcoming catalysts events {expiry_month} {current_year}"`

## Polymaster Integration

Check for recent whale alerts on the same market:
```bash
cd ~/prowl/polymaster/integration
node dist/cli.js alerts --category=crypto --limit=10
```

If whale alerts exist for this market, factor in:
- Number of whale entries (bullish signal if multiple)
- Whale win rates and leaderboard positions
- Direction consensus (all buying YES vs mixed)

## Confidence Adjustments

| Signal | Adjustment |
|--------|------------|
| 3+ whales buying same direction | +10% confidence |
| Funding rate extreme (>0.1%) | -10% confidence (reversal risk) |
| Within 4h of macro event | -15% confidence |
| ETF inflows >$500M/day for 5+ days | +5% confidence |
| Price above 200d MA with rising OI | +5% confidence |
| Price below 200d MA with rising OI | -5% confidence |
| Exchange inflows spike (>2x avg) | -10% confidence |

## Common Pitfalls
- **Weekend liquidity**: Crypto trades 24/7 but weekend liquidity is thinner — larger moves possible.
- **Leverage cascades**: High OI + thin liquidity = flash crashes. Be cautious on leveraged markets.
- **Correlation**: BTC and ETH are highly correlated. Don't count separate ETH and BTC positions as independent — they are correlated exposure.
- **Stablecoin depegs**: Monitor USDT/USDC stability — depeg events crash all crypto markets.
