# Sports Market Research Context

## Role
Specialized research guidance for sports prediction markets on Kalshi. Load this context when the researcher agent is working on a sports-category market.

## Key Data Sources

### Injury Reports
- **Official injury reports**: NBA/NFL/NHL/MLB all publish injury reports before games. Check timing — reports can be updated up to game time.
- **Game-time decisions (GTD)**: Players listed as "questionable" or "doubtful" can swing lines dramatically if they play or sit.
- **Load management**: Star players sometimes rest on back-to-back games, especially late in regular season. Monitor for rest announcements.
- **Key player impact**: Quantify the impact — a star QB being out is worth more than a backup OL. Check win probability models with/without the player.

**RULE**: Always check the latest injury report before entering any sports position. Stale injury data is the #1 source of bad sports bets.

### Line Movement & Sharp Money
- **Opening vs current line**: Large line movement since open indicates sharp money or significant new information.
- **Reverse line movement (RLM)**: When the line moves opposite to the public betting percentage, sharps are likely on the other side.
- **Steam moves**: Rapid line movement across multiple books simultaneously indicates coordinated sharp action.
- **Closing line value (CLV)**: The closing line is the most accurate predictor. If you can beat it consistently, you have an edge.

### Rest & Schedule
- **Back-to-back games**: NBA teams on B2B play significantly worse, especially on the road.
- **Travel distance**: West-to-East coast travel + early tip-off = fatigue disadvantage.
- **Scheduling spots**: NFL teams after a bye week perform better. Teams in "look-ahead" spots (easy game before a big rivalry) may underperform.
- **Altitude**: Denver (5,280 ft) is a significant factor for visiting teams in NFL, NBA, and MLB.

### Advanced Metrics
- **NBA**: Net rating, pace, offensive/defensive efficiency, clutch stats.
- **NFL**: DVOA, EPA/play, success rate, pressure rate.
- **MLB**: FIP, wRC+, park factors, bullpen usage.
- **NHL**: Expected goals (xG), Corsi, high-danger chances.

## Research Queries (Perplexity)

1. **Matchup**: `"{team1} vs {team2} {date} preview prediction odds"`
2. **Injuries**: `"{team} injury report {date} {player_name} status"`
3. **Trends**: `"{team} recent form last 10 games stats {year}"`
4. **Sharps**: `"{game} line movement sharp money {date} {year}"`

## Polymaster Integration

Sports whale alerts track big money on Polymarket:
```bash
cd ~/prowl/polymaster/integration
node dist/cli.js alerts --category=sports --limit=10
```

Sports markets attract sophisticated bettors. Whale entries from high win-rate accounts are particularly meaningful in sports where edge detection is well-established.

## Confidence Adjustments

| Signal | Adjustment |
|--------|------------|
| Star player confirmed OUT (late scratch) | +10% confidence on the other side |
| Reverse line movement detected | +10% confidence (follow the sharps) |
| Team on 2nd of back-to-back, road | +10% confidence for the opponent |
| 3+ whale alerts same direction | +10% confidence |
| Key player "game-time decision" | -15% confidence (wait for clarity) |
| Playoff game (higher variance) | -5% confidence |
| Regular season, meaningless game | -10% confidence (motivation unclear) |
| Rain/snow for outdoor game | -5% confidence (increases variance) |

## Common Pitfalls
- **Public bias**: Public money heavily favors favorites, home teams, and big-name teams. Contrarian underdogs can offer value.
- **Small sample sizes**: Season-long stats are more reliable than "last 3 games" trends. Don't overreact to small samples.
- **Prop market correlation**: Player props (e.g., "LeBron over 25 points") correlate with game outcome. Don't treat them as independent.
- **Weather for outdoor sports**: Check game-time weather for NFL, MLB, and outdoor soccer. Wind and precipitation affect scoring.
- **Time zone disadvantage**: West Coast teams playing early East Coast games (1pm ET) are at a circadian disadvantage.
- **Referee/umpire assignments**: Some officials call more fouls/penalties. Check assignment for systematic biases in foul-heavy markets.
