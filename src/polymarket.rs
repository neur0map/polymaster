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
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "market")]
    pub market: String,
    #[serde(rename = "asset_id")]
    pub asset_id: String,
    #[serde(rename = "side")]
    pub side: String,
    #[serde(rename = "size")]
    pub size: f64,
    #[serde(rename = "price")]
    pub price: f64,
    #[serde(rename = "timestamp")]
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
    #[serde(rename = "id")]
    id: String,
    #[serde(rename = "market")]
    market: Option<String>,
    #[serde(rename = "asset")]
    asset: Option<String>,
    #[serde(rename = "side")]
    side: Option<String>,
    #[serde(rename = "size")]
    size: Option<f64>,
    #[serde(rename = "price")]
    price: Option<f64>,
    #[serde(rename = "timestamp")]
    timestamp: Option<i64>,
    #[serde(rename = "type")]
    activity_type: Option<String>,
    #[serde(rename = "user")]
    user: Option<String>,
    #[serde(rename = "maker")]
    maker: Option<String>,
    #[serde(rename = "proxyWallet")]
    proxy_wallet: Option<String>,
}

pub async fn fetch_recent_trades() -> Result<Vec<Trade>, PolymarketError> {
    let client = reqwest::Client::new();
    
    // Use the Polymarket Data API to fetch recent activity
    // This is a public endpoint that doesn't require authentication
    let url = "https://data-api.polymarket.com/trades";
    
    let response = client
        .get(url)
        .query(&[("limit", "100")])
        .header("Accept", "application/json")
        .send()
        .await?;

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
                // Only process TRADE type activities
                if item.activity_type.as_deref() != Some("TRADE") {
                    return None;
                }
                
                Some(Trade {
                    id: item.id.clone(),
                    market: item.market.unwrap_or_default(),
                    asset_id: item.asset.unwrap_or_default(),
                    side: item.side.unwrap_or_default(),
                    size: item.size.unwrap_or(0.0),
                    price: item.price.unwrap_or(0.0),
                    timestamp: item
                        .timestamp
                        .map(|ts| chrono::DateTime::from_timestamp(ts, 0)
                            .map(|dt| dt.to_rfc3339())
                            .unwrap_or_else(|| "Unknown".to_string()))
                        .unwrap_or_else(|| "Unknown".to_string()),
                    market_title: None,
                    outcome: None,
                    wallet_id: item.user.or(item.maker).or(item.proxy_wallet),
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
                if item.activity_type.as_deref() != Some("TRADE") {
                    return None;
                }
                
                Some(Trade {
                    id: item.id.clone(),
                    market: item.market.unwrap_or_default(),
                    asset_id: item.asset.unwrap_or_default(),
                    side: item.side.unwrap_or_default(),
                    size: item.size.unwrap_or(0.0),
                    price: item.price.unwrap_or(0.0),
                    timestamp: item
                        .timestamp
                        .map(|ts| chrono::DateTime::from_timestamp(ts, 0)
                            .map(|dt| dt.to_rfc3339())
                            .unwrap_or_else(|| "Unknown".to_string()))
                        .unwrap_or_else(|| "Unknown".to_string()),
                    market_title: None,
                    outcome: None,
                    wallet_id: item.user.or(item.maker).or(item.proxy_wallet),
                })
            })
            .collect();
        return Ok(trades);
    }

    // If parsing fails, return empty list rather than error
    // This allows the tool to continue working even if Polymarket API format changes
    Ok(Vec::new())
}

#[derive(Debug, Deserialize)]
struct MarketResponse {
    #[serde(rename = "title")]
    title: Option<String>,
    #[serde(rename = "question")]
    question: Option<String>,
    #[serde(rename = "outcomes")]
    outcomes: Option<Vec<String>>,
    #[serde(rename = "description")]
    description: Option<String>,
    #[serde(rename = "category")]
    category: Option<String>,
}

pub async fn fetch_market_info(market_id: &str) -> Option<(String, String)> {
    let client = reqwest::Client::new();
    
    // Try Gamma Markets API
    let url = format!("https://gamma-api.polymarket.com/markets/{}", market_id);
    
    match client.get(&url).send().await {
        Ok(response) if response.status().is_success() => {
            if let Ok(text) = response.text().await {
                if let Ok(market) = serde_json::from_str::<MarketResponse>(&text) {
                    let title = market.question
                        .or(market.title)
                        .unwrap_or_else(|| "Unknown Market".to_string());
                    let outcome = market.outcomes
                        .and_then(|o| o.first().cloned())
                        .unwrap_or_else(|| "Yes".to_string());
                    return Some((title, outcome));
                }
            }
        }
        _ => {}
    }
    
    None
}
