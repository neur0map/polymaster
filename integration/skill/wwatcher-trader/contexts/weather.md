# Weather Market Research Context

## Role
Specialized research guidance for weather prediction markets on Kalshi. Load this context when the researcher agent is working on a weather-category market.

## Key Data Sources

### Forecast Models
- **GFS (Global Forecast System)**: US model, updates every 6 hours. Good for 1-7 day forecasts.
- **ECMWF (European model)**: Generally more accurate than GFS, especially 3-10 day range.
- **NAM (North American Mesoscale)**: Higher resolution, best for 1-3 day detailed forecasts.
- **HRRR (High-Resolution Rapid Refresh)**: Best for 0-18 hour forecasts, hourly updates.

**CRITICAL RULE**: Require **2+ model agreement** before entering a weather position. A single model showing a result is insufficient — models diverge frequently.

### Temperature Markets
- **Margin requirement**: Need 3°F+ buffer above/below the threshold. A forecast of 101°F for a "100°F+" market is too close.
- **Urban heat island**: City weather stations read 2-5°F warmer than surrounding areas. Check which station Kalshi uses for resolution.
- **Record vs threshold**: "Record high" markets are harder than "above X°F" markets — records have survivorship bias.

### Precipitation Markets
- **Type matters**: Rain, snow, sleet, and freezing rain are different. A 70% precipitation probability doesn't mean 70% snow probability.
- **Snow ratios**: Standard 10:1 snow-to-liquid ratio varies enormously (5:1 to 30:1). Temperature at cloud level matters more than surface temp.
- **Trace amounts**: Kalshi typically requires measurable precipitation (≥0.01"). Flurries or mist may not count.
- **Rain/snow line**: Check the freezing level altitude — a few hundred feet of elevation can change rain to snow.

### Severe Weather
- **SPC outlooks**: Storm Prediction Center issues Day 1-8 convective outlooks. Enhanced/Moderate/High risk areas are where the action is.
- **Hurricane tracks**: Ensemble model spread narrows as landfall approaches. Wide spread = high uncertainty = avoid.
- **Tornado probability**: Extremely hard to predict specific locations. Only trade broad area markets.

## Research Queries (Perplexity)

1. **Current forecast**: `"{city} weather forecast {target_date} temperature precipitation"`
2. **Model comparison**: `"{city} GFS vs ECMWF forecast {target_date} {current_year}"`
3. **Climatology**: `"{city} average {metric} {month} historical records"`
4. **Pattern analysis**: `"weather pattern {region} {current_week} jet stream outlook"`

## Polymaster Integration

Weather whale alerts are rare but valuable:
```bash
cd ~/prowl/polymaster/integration
node dist/cli.js alerts --category=weather --limit=5
```

Weather markets on Kalshi are less liquid than crypto/politics — whale activity here often indicates strong conviction from specialized forecasters.

## Confidence Adjustments

| Signal | Adjustment |
|--------|------------|
| 3+ models agree on outcome | +15% confidence |
| Models disagree (split verdict) | -20% confidence, consider skipping |
| Temp forecast within 2°F of threshold | -15% confidence |
| Temp forecast 5°F+ from threshold | +10% confidence |
| Event is <24h away with consistent forecasts | +10% confidence |
| Event is 5+ days away | -10% confidence (models less reliable) |
| Tropical system involved | -10% confidence (chaotic) |
| Snow market with temps near 32°F | -15% confidence (rain/snow line risk) |

## Common Pitfalls
- **Single model reliance**: The #1 mistake. Always cross-reference GFS, ECMWF, and at least one regional model.
- **Temperature inversions**: Surface temperature can differ significantly from air temperature at measurement height.
- **Verification station**: Confirm which weather station Kalshi uses for market resolution. Airport stations vs downtown stations can differ by 3-5°F.
- **Timing**: A market for "high above 90°F" might verify at 2pm when the peak is at 3pm. Check resolution timing.
- **Precipitation accumulation periods**: Is it 24-hour total or calendar day? Overnight rain might count for a different day depending on the measurement window.
