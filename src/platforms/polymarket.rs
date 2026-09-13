use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PolymarketError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    #[error("Failed to parse response: {0}")]
    ParseError(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Trade {
    pub id: String,
    pub market: String,
    pub asset_id: String,
    pub side: String,
    pub size: f64,
    pub price: f64,
    pub timestamp: String,
    #[serde(skip)]
    pub market_title: Option<String>,
    #[serde(skip)]
    pub outcome: Option<String>,
    #[serde(skip)]
    pub wallet_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TradesResponse {
    #[serde(default)]
    data: Vec<ActivityItem>,
}

#[derive(Debug, Deserialize)]
struct ActivityItem {
    #[serde(rename = "transactionHash")]
    id: String,
    #[serde(rename = "conditionId")]
    market: Option<String>,
    #[serde(rename = "asset")]
    asset: Option<String>,
    side: Option<String>,
    size: Option<f64>,
    price: Option<f64>,
    timestamp: Option<i64>,
    #[serde(rename = "name")]
    user: Option<String>,
    maker: Option<String>,
    #[serde(rename = "proxyWallet")]
    proxy_wallet: Option<String>,
    // New API includes these fields directly
    title: Option<String>,
    outcome: Option<String>,
}

pub async fn fetch_market_context(condition_id: &str) -> Option<crate::alerts::MarketContext> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://gamma-api.polymarket.com/markets?condition_ids={}",
        condition_id
    );

    let response = client
        .get(&url)
        .header("Accept", "application/json")
        .send()
        .await
        .ok()?;

    if !response.status().is_success() {
        return None;
    }

    let text = response.text().await.ok()?;
    let markets: Vec<serde_json::Value> = serde_json::from_str(&text).ok()?;
    let market = markets.first()?;

    let yes_price = market.get("outcomePrices")
        .and_then(|v| v.as_str())
        .and_then(|s| {
            // outcomePrices is a JSON string like "[\"0.65\",\"0.35\"]"
            let prices: Vec<String> = serde_json::from_str(s).ok()?;
            prices.first()?.parse::<f64>().ok()
        })
        .or_else(|| {
            market.get("bestBid").and_then(|v| v.as_str()).and_then(|s| s.parse::<f64>().ok())
        })
        .unwrap_or(0.0);

    let no_price = 1.0 - yes_price;

    let spread = market.get("spread")
        .and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
        .unwrap_or(0.0);

    let volume_24h = market.get("volume24hr")
        .and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
        .unwrap_or(0.0);

    let open_interest = market.get("openInterest")
        .and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
        .unwrap_or(0.0);

    let price_change_24h = market.get("oneDayPriceChange")
        .and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
        .unwrap_or(0.0);

    let liquidity = market.get("liquidityClob")
        .or_else(|| market.get("liquidity"))
        .and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
        .unwrap_or(0.0);

    // Extract tags from Gamma API response
    let tags: Vec<String> = market.get("tags")
        .and_then(|v| {
            if let Some(arr) = v.as_array() {
                Some(arr.iter().filter_map(|t| {
                    t.get("slug").or_else(|| t.get("label")).or(Some(t))
                        .and_then(|s| s.as_str())
                        .map(|s| s.to_string())
                }).collect())
            } else if let Some(s) = v.as_str() {
                // Sometimes tags is a JSON string
                serde_json::from_str::<Vec<serde_json::Value>>(s).ok().map(|arr| {
                    arr.iter().filter_map(|t| {
                        t.get("slug").or_else(|| t.get("label")).or(Some(t))
                            .and_then(|s| s.as_str())
                            .map(|s| s.to_string())
                    }).collect()
                })
            } else {
                None
            }
        })
        .unwrap_or_default();

    // Direct market page URL for clickable alerts (from the market slug)
    let url = market
        .get("slug")
        .and_then(|s| s.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| format!("https://polymarket.com/market/{}", s));

    Some(crate::alerts::MarketContext {
        yes_price,
        no_price,
        spread,
        volume_24h,
        open_interest,
        price_change_24h,
        liquidity,
        tags,
        url,
    })
}

/// Fetch order book from CLOB API for a given asset (token) ID
pub async fn fetch_order_book(asset_id: &str) -> Option<crate::alerts::OrderBookSummary> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .ok()?;

    let response = client
        .get("https://clob.polymarket.com/book")
        .query(&[("token_id", asset_id)])
        .header("Accept", "application/json")
        .send()
        .await
        .ok()?;

    if !response.status().is_success() {
        return None;
    }

    let text = response.text().await.ok()?;
    let book: serde_json::Value = serde_json::from_str(&text).ok()?;

    let bids = book.get("bids").and_then(|v| v.as_array())?;
    let asks = book.get("asks").and_then(|v| v.as_array())?;

    let best_bid = bids.first()
        .and_then(|b| b.get("price").and_then(|p| p.as_str().and_then(|s| s.parse::<f64>().ok()).or(p.as_f64())))
        .unwrap_or(0.0);
    let best_ask = asks.first()
        .and_then(|a| a.get("price").and_then(|p| p.as_str().and_then(|s| s.parse::<f64>().ok()).or(p.as_f64())))
        .unwrap_or(0.0);

    // Calculate depth within 10% of best bid/ask
    let bid_threshold = best_bid * 0.9;
    let ask_threshold = best_ask * 1.1;

    let mut bid_depth = 0.0;
    let mut bid_levels = 0u32;
    for bid in bids {
        let price = bid.get("price")
            .and_then(|p| p.as_str().and_then(|s| s.parse::<f64>().ok()).or(p.as_f64()))
            .unwrap_or(0.0);
        let size = bid.get("size")
            .and_then(|s| s.as_str().and_then(|v| v.parse::<f64>().ok()).or(s.as_f64()))
            .unwrap_or(0.0);
        if price >= bid_threshold {
            bid_depth += price * size;
            bid_levels += 1;
        }
    }

    let mut ask_depth = 0.0;
    let mut ask_levels = 0u32;
    for ask in asks {
        let price = ask.get("price")
            .and_then(|p| p.as_str().and_then(|s| s.parse::<f64>().ok()).or(p.as_f64()))
            .unwrap_or(0.0);
        let size = ask.get("size")
            .and_then(|s| s.as_str().and_then(|v| v.parse::<f64>().ok()).or(s.as_f64()))
            .unwrap_or(0.0);
        if price <= ask_threshold {
            ask_depth += price * size;
            ask_levels += 1;
        }
    }

    Some(crate::alerts::OrderBookSummary {
        best_bid,
        best_ask,
        bid_depth_10pct: bid_depth,
        ask_depth_10pct: ask_depth,
        bid_levels,
        ask_levels,
    })
}

/// Fetch top holders for a Polymarket market (by condition ID)
pub async fn fetch_top_holders(condition_id: &str) -> Option<crate::alerts::TopHoldersSummary> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .ok()?;

    let response = client
        .get("https://data-api.polymarket.com/top-holders")
        .query(&[("market", condition_id)])
        .header("Accept", "application/json")
        .send()
        .await
        .ok()?;

    if !response.status().is_success() {
        return None;
    }

    let text = response.text().await.ok()?;
    let items: Vec<serde_json::Value> = serde_json::from_str(&text).ok()?;

    if items.is_empty() {
        return None;
    }

    let mut holders = Vec::new();

    for item in items.iter().take(5) {
        let wallet = item.get("proxyWallet")
            .or_else(|| item.get("wallet"))
            .or_else(|| item.get("address"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        let shares = item.get("shares")
            .or_else(|| item.get("size"))
            .or_else(|| item.get("amount"))
            .and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
            .unwrap_or(0.0);
        let value = item.get("value")
            .or_else(|| item.get("currentValue"))
            .and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
            .unwrap_or(0.0);

        holders.push(crate::alerts::TopHolder { wallet, shares, value });
    }

    // Sum all holders for total
    let all_total: f64 = items.iter().map(|item| {
        item.get("shares")
            .or_else(|| item.get("size"))
            .or_else(|| item.get("amount"))
            .and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
            .unwrap_or(0.0)
    }).sum();

    Some(crate::alerts::TopHoldersSummary {
        top_holders: holders,
        total_shares: all_total,
    })
}

pub async fn fetch_recent_trades(min_value: Option<u64>) -> Result<Vec<Trade>, PolymarketError> {
    let client = reqwest::Client::new();

    // Use the Polymarket Data API to fetch recent activity
    // This is a public endpoint that doesn't require authentication
    let url = "https://data-api.polymarket.com/trades";

    let mut request = client
        .get(url)
        .header("Accept", "application/json");

    // Use server-side CASH filtering when threshold is set
    // This lets the API pre-filter to only whale-sized trades
    if let Some(threshold) = min_value {
        request = request
            .query(&[("limit", "500"), ("filterType", "CASH"), ("filterAmount", &threshold.to_string()), ("takerOnly", "true")]);
    } else {
        request = request.query(&[("limit", "100"), ("takerOnly", "true")]);
    }

    let response = request.send().await?;

    if !response.status().is_success() {
        return Err(PolymarketError::ParseError(format!(
            "API returned status: {}",
            response.status()
        )));
    }

    let text = response.text().await?;

    // Try to parse as array first (some endpoints return arrays directly)
    if let Ok(items) = serde_json::from_str::<Vec<ActivityItem>>(&text) {
        let trades = items
            .into_iter()
            .filter_map(|item| {
                // Skip trades missing critical data
                let market = item.market?;
                let asset_id = item.asset?;
                let side = item.side?;
                let size = item.size?;
                let price = item.price?;

                Some(Trade {
                    id: item.id.clone(),
                    market,
                    asset_id,
                    side,
                    size,
                    price,
                    timestamp: item
                        .timestamp
                        .and_then(|ts| {
                            chrono::DateTime::from_timestamp(ts, 0)
                                .map(|dt| dt.to_rfc3339())
                        })
                        .unwrap_or_else(|| format!("timestamp_error_{}", item.id)),
                    // New API includes title and outcome directly
                    market_title: item.title,
                    outcome: item.outcome,
                    wallet_id: item.proxy_wallet.or(item.user).or(item.maker),
                })
            })
            .collect();
        return Ok(trades);
    }

    // Try wrapped response format
    if let Ok(wrapped) = serde_json::from_str::<TradesResponse>(&text) {
        let trades = wrapped
            .data
            .into_iter()
            .filter_map(|item| {
                // Skip trades missing critical data
                let market = item.market?;
                let asset_id = item.asset?;
                let side = item.side?;
                let size = item.size?;
                let price = item.price?;

                Some(Trade {
                    id: item.id.clone(),
                    market,
                    asset_id,
                    side,
                    size,
                    price,
                    timestamp: item
                        .timestamp
                        .and_then(|ts| {
                            chrono::DateTime::from_timestamp(ts, 0)
                                .map(|dt| dt.to_rfc3339())
                        })
                        .unwrap_or_else(|| format!("timestamp_error_{}", item.id)),
                    // New API includes title and outcome directly
                    market_title: item.title,
                    outcome: item.outcome,
                    wallet_id: item.proxy_wallet.or(item.user).or(item.maker),
                })
            })
            .collect();
        return Ok(trades);
    }

    // If parsing fails, return empty list rather than error
    // This allows the tool to continue working even if Polymarket API format changes
    Ok(Vec::new())
}
