use crate::config::Config;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum KalshiError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    #[error("Failed to parse response: {0}")]
    ParseError(String),
    #[error("Authentication failed: {0}")]
    AuthError(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Trade {
    #[serde(rename = "trade_id")]
    pub trade_id: String,
    #[serde(rename = "ticker")]
    pub ticker: String,
    #[serde(rename = "price")]
    pub price: f64,
    #[serde(rename = "count")]
    pub count: i32,
    #[serde(rename = "yes_price")]
    pub yes_price: f64,
    #[serde(rename = "no_price")]
    pub no_price: f64,
    #[serde(rename = "taker_side")]
    pub taker_side: String,
    #[serde(rename = "created_time")]
    pub created_time: String,
    #[serde(skip)]
    pub market_title: Option<String>,
    // Note: Kalshi public API doesn't expose account IDs for privacy
    // Use trade_id as proxy for tracking patterns
}

#[derive(Debug, Deserialize)]
struct TradesResponse {
    #[serde(default)]
    trades: Vec<Trade>,
    #[serde(default)]
    cursor: Option<String>,
}

pub async fn fetch_recent_trades(config: Option<&Config>) -> Result<Vec<Trade>, KalshiError> {
    let client = reqwest::Client::new();
    
    // Kalshi's public trades endpoint
    let url = "https://api.elections.kalshi.com/trade-api/v2/markets/trades";
    
    let mut request = client
        .get(url)
        .query(&[("limit", "100")])
        .header("Accept", "application/json");

    // Add authentication if credentials are provided
    if let Some(cfg) = config {
        if let (Some(key_id), Some(_private_key)) = (&cfg.kalshi_api_key_id, &cfg.kalshi_private_key) {
            // For simplicity, we'll use basic auth
            // In production, you'd implement proper HMAC signature
            request = request.header("KALSHI-ACCESS-KEY", key_id);
        }
    }

    let response = request.send().await?;

    if !response.status().is_success() {
        return Err(KalshiError::ParseError(format!(
            "API returned status: {}",
            response.status()
        )));
    }

    let text = response.text().await?;
    
    match serde_json::from_str::<TradesResponse>(&text) {
        Ok(response) => Ok(response.trades),
        Err(e) => {
            // If parsing fails, return empty list to allow tool to continue
            eprintln!("Warning: Failed to parse Kalshi response: {}", e);
            Ok(Vec::new())
        }
    }
}

#[derive(Debug, Deserialize)]
struct MarketResponse {
    market: MarketData,
}

#[derive(Debug, Deserialize)]
struct MarketData {
    title: Option<String>,
    subtitle: Option<String>,
}

pub async fn fetch_market_info(ticker: &str) -> Option<String> {
    let client = reqwest::Client::new();
    let url = format!("https://api.elections.kalshi.com/trade-api/v2/markets/{}", ticker);
    
    match client.get(&url).send().await {
        Ok(response) if response.status().is_success() => {
            if let Ok(text) = response.text().await {
                if let Ok(market_response) = serde_json::from_str::<MarketResponse>(&text) {
                    return market_response.market.title
                        .or(market_response.market.subtitle);
                }
            }
        }
        _ => {}
    }
    
    None
}

pub fn parse_ticker_details(ticker: &str) -> String {
    // Parse Kalshi ticker to extract bet details
    // Format examples:
    // KXNHLGAME-26JAN08ANACAR-CAR = NHL game, Carolina wins
    // KXNCAAFTOTAL-26JAN08MIAMISS-51 = NCAA football total points over 51
    // KXHIGHNY-24DEC-T63 = NYC high temp threshold
    // KXETHD-26JAN0818-T3109.99 = ETH price threshold
    
    // Cryptocurrency/Stock price thresholds
    if ticker.contains("ETH") || ticker.contains("BTC") || ticker.contains("SOL") || 
       ticker.contains("SPX") || ticker.contains("TSLA") {
        let parts: Vec<&str> = ticker.split('-').collect();
        if let Some(threshold_part) = parts.last() {
            if threshold_part.starts_with('T') || threshold_part.starts_with('t') {
                let price = &threshold_part[1..];
                let asset = if ticker.contains("ETH") { "Ethereum (ETH)" }
                          else if ticker.contains("BTC") { "Bitcoin (BTC)" }
                          else if ticker.contains("SOL") { "Solana (SOL)" }
                          else if ticker.contains("SPX") { "S&P 500" }
                          else if ticker.contains("TSLA") { "Tesla" }
                          else { "Asset" };
                
                return format!("Betting YES = {} price ≥ ${} at expiry", asset, price);
            }
        }
    }
    
    // Check for sports totals (over/under)
    if ticker.contains("TOTAL") {
        let parts: Vec<&str> = ticker.split('-').collect();
        if let Some(threshold) = parts.last() {
            if threshold.chars().all(|c| c.is_numeric()) {
                let sport = if ticker.contains("NFL") { "NFL" }
                          else if ticker.contains("NBA") { "NBA" }
                          else if ticker.contains("NHL") { "NHL" }
                          else if ticker.contains("MLB") { "MLB" }
                          else if ticker.contains("NCAAF") || ticker.contains("CFB") { "College Football" }
                          else if ticker.contains("NCAAB") || ticker.contains("CBB") { "College Basketball" }
                          else { "Game" };
                
                // Extract teams if possible
                if parts.len() >= 3 {
                    if let Some(teams_part) = parts.get(parts.len() - 2) {
                        if teams_part.len() >= 6 {
                            let team_codes = &teams_part[teams_part.len()-6..];
                            let away = &team_codes[..3];
                            let home = &team_codes[3..];
                            return format!("Betting YES = Total points OVER {} | {} @ {} ({})", 
                                threshold, away.to_uppercase(), home.to_uppercase(), sport);
                        }
                    }
                }
                
                return format!("Betting YES = Total points OVER {} ({})", threshold, sport);
            }
        }
    }
    
    if ticker.contains("NHLGAME") || ticker.contains("NFLGAME") || 
       ticker.contains("NBAGAME") || ticker.contains("MLBGAME") {
        // Sports game format
        let parts: Vec<&str> = ticker.split('-').collect();
        if parts.len() >= 3 {
            let outcome = parts.last().unwrap_or(&"");
            
            // Extract team codes from middle part
            if let Some(teams_part) = parts.get(parts.len() - 2) {
                // Format like "26JAN08ANACAR" - extract last 6 chars for teams
                if teams_part.len() >= 6 {
                    let team_codes = &teams_part[teams_part.len()-6..];
                    let away = &team_codes[..3];
                    let home = &team_codes[3..];
                    
                    let sport = if ticker.contains("NHL") { "NHL" }
                              else if ticker.contains("NFL") { "NFL" }
                              else if ticker.contains("NBA") { "NBA" }
                              else { "MLB" };
                    
                    return format!("Betting YES = {} wins | {} @ {} ({})", 
                        outcome.to_uppercase(), away.to_uppercase(), home.to_uppercase(), sport);
                }
            }
        }
    // Check for point spreads
    } else if ticker.contains("SPREAD") {
        let parts: Vec<&str> = ticker.split('-').collect();
        if let Some(last_part) = parts.last() {
            // Could be like "CAR3" meaning Carolina -3
            let team = last_part.chars().take_while(|c| c.is_alphabetic()).collect::<String>();
            let spread = last_part.chars().skip_while(|c| c.is_alphabetic()).collect::<String>();
            
            if !team.is_empty() && !spread.is_empty() {
                return format!("Betting YES = {} covers -{} point spread", 
                    team.to_uppercase(), spread);
            }
        }
    // Check for player props (touchdowns, points, etc)
    } else if ticker.contains("TD") || ticker.contains("SCORE") {
        let parts: Vec<&str> = ticker.split('-').collect();
        if let Some(threshold) = parts.last() {
            if threshold.chars().all(|c| c.is_numeric()) {
                let prop_type = if ticker.contains("TD") { "touchdowns" } else { "points" };
                return format!("Betting YES = Player gets {} or more {}", threshold, prop_type);
            }
        }
    } else if ticker.contains("HIGH") || ticker.contains("LOW") {
        // Temperature markets
        if ticker.contains("T") {
            let parts: Vec<&str> = ticker.split('-').collect();
            if let Some(threshold_part) = parts.last() {
                if threshold_part.starts_with('T') {
                    let temp = &threshold_part[1..];
                    let location = if ticker.contains("NY") { "NYC" }
                                  else if ticker.contains("LA") { "LA" }
                                  else if ticker.contains("CHI") { "Chicago" }
                                  else { "Location" };
                    
                    let metric = if ticker.contains("HIGH") { "High" } else { "Low" };
                    return format!("Betting YES = {} temp ≥ {}°F", metric, temp);
                }
            }
        }
    } else if ticker.contains("PRES") {
        // Presidential/election markets
        let parts: Vec<&str> = ticker.split('-').collect();
        if let Some(outcome) = parts.last() {
            return format!("Betting YES = {} wins", outcome.to_uppercase());
        }
    }
    
    // Default: try to extract outcome from last part
    let parts: Vec<&str> = ticker.split('-').collect();
    if let Some(outcome) = parts.last() {
        if outcome.len() <= 10 && outcome.chars().all(|c| c.is_alphanumeric()) {
            return format!("Betting YES = {} outcome", outcome.to_uppercase());
        }
    }
    
    String::from("Yes/No market - check ticker for details")
}
