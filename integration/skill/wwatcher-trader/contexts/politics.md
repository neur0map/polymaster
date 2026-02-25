# Politics Market Research Context

## Role
Specialized research guidance for political prediction markets on Kalshi. Load this context when the researcher agent is working on a politics-category market.

## Key Data Sources

### Polling Data
- **Aggregators**: FiveThirtyEight, RealClearPolitics, The Economist, Silver Bulletin — use aggregates, never single polls.
- **Poll quality**: Check sample size (n>1000 preferred), methodology (live caller vs online), sponsor bias, and recency.
- **Likely voter vs registered voter**: Likely voter screens are more accurate closer to elections. Registered voter polls overcount non-voters.
- **Margin of error**: A 48-45 lead with ±3% MOE is NOT a safe bet. The true range overlaps.

**RULE**: Weight polling averages, not individual polls. A single outlier poll ≠ a trend.

### Prediction Market Cross-Reference
- **Polymarket**: Largest crypto prediction market. Check for whale activity via polymaster.
- **PredictIt**: Smaller but has unique markets. 85¢ ceiling creates distortions.
- **Metaculus**: Community forecasts, useful for calibration comparison.

Cross-platform price divergence can signal opportunities or information asymmetry.

### Legislative/Policy Markets
- **Vote counting**: Know the chamber (House needs 218, Senate needs 60 for cloture or 51 for reconciliation).
- **Whip counts**: Track announced positions. Undecided members are where uncertainty lives.
- **Committee passage**: Bills must pass committee before floor vote. Committee composition matters.
- **Reconciliation vs regular order**: Different thresholds, different procedures.

### Demographic & Structural Factors
- **Redistricting**: Check if district lines have changed since last election.
- **Early voting data**: Party registration of early voters can signal turnout patterns.
- **Incumbency advantage**: Sitting officials win ~90% of House races, ~80% of Senate races.
- **Historical patterns**: Midterm elections typically favor the opposition party.

## Research Queries (Perplexity)

1. **Polls**: `"{candidate/issue} polling average {month} {year} latest polls"`
2. **Analysis**: `"{race/legislation} prediction analysis {year} expert forecasts"`
3. **Developments**: `"{candidate/issue} latest news developments {current_week} {year}"`
4. **Historical**: `"{similar past event} historical precedent outcome"`

## Polymaster Integration

Political whale alerts show money movement:
```bash
cd ~/prowl/polymaster/integration
node dist/cli.js alerts --category=politics --limit=10
```

Political markets on Kalshi and Polymarket attract large institutional bets. Whale activity often precedes polling shifts or insider knowledge of developments.

## Confidence Adjustments

| Signal | Adjustment |
|--------|------------|
| Polling average lead >10pts with 5+ polls | +15% confidence |
| Polling average lead 3-5pts | +5% confidence |
| Polling average within margin of error | -10% confidence |
| Major news event in last 24h | -10% confidence (wait for polls to absorb) |
| Cross-platform price agreement (within 3¢) | +5% confidence |
| Cross-platform price divergence (>10¢) | Flag for deeper research |
| Event is >30 days away | -10% confidence |
| Historical pattern strongly favors one side | +5% confidence |

## Common Pitfalls
- **Herding**: Pollsters often converge toward each other before elections. Late-breaking shifts can be missed.
- **Shy voter effect**: Some demographics underreport their true preferences. Adjust for historical polling misses in that region.
- **Primary vs general**: Primary polls are notoriously unreliable — smaller electorate, volatile preferences.
- **Legislation timing**: Bills get delayed, amended, or combined with other legislation. "By date X" markets are risky if the legislative calendar is uncertain.
- **Appointment/confirmation markets**: Executive nominations require Senate confirmation — count votes, don't assume.
- **October surprises**: Major political events can shift races dramatically in final weeks. Maintain larger position flexibility near election dates.
