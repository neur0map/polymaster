pub mod anomaly;
pub mod display;
pub mod history;
pub mod sound;
pub mod webhook;

use crate::types;
use crate::whale_profile::WhaleProfile;

/// Market context data fetched per whale alert for edge detection
#[derive(Debug, Clone)]
pub struct MarketContext {
    pub yes_price: f64,
    pub no_price: f64,
    pub spread: f64,
    pub volume_24h: f64,
    pub open_interest: f64,
    pub price_change_24h: f64,
    pub liquidity: f64,
    pub tags: Vec<String>,
    /// Direct link to the market page (Polymarket; Kalshi exposes none)
    pub url: Option<String>,
}

/// Order book depth summary
#[derive(Debug, Clone)]
pub struct OrderBookSummary {
    pub best_bid: f64,
    pub best_ask: f64,
    pub bid_depth_10pct: f64,
    pub ask_depth_10pct: f64,
    pub bid_levels: u32,
    pub ask_levels: u32,
}

/// Top holders summary for a Polymarket market
#[derive(Debug, Clone)]
pub struct TopHoldersSummary {
    pub top_holders: Vec<TopHolder>,
    pub total_shares: f64,
}

#[derive(Debug, Clone)]
pub struct TopHolder {
    pub wallet: String,
    pub shares: f64,
    pub value: f64,
}

/// Shared alert data structure used by webhook, logging, and display
pub struct AlertData<'a> {
    pub platform: &'a str,
    pub market_title: Option<&'a str>,
    pub outcome: Option<&'a str>,
    pub side: &'a str,
    pub value: f64,
    pub price: f64,
    pub size: f64,
    pub timestamp: &'a str,
    pub wallet_id: Option<&'a str>,
    pub wallet_activity: Option<&'a types::WalletActivity>,
    pub market_context: Option<&'a MarketContext>,
    pub whale_profile: Option<&'a WhaleProfile>,
    pub order_book: Option<&'a OrderBookSummary>,
    pub top_holders: Option<&'a TopHoldersSummary>,
}

impl<'a> AlertData<'a> {
    pub fn is_sell(&self) -> bool {
        self.side.to_uppercase() == "SELL"
    }

    pub fn alert_type(&self) -> &'static str {
        if self.is_sell() { "WHALE_EXIT" } else { "WHALE_ENTRY" }
    }
}

/// Build a serde_json::Value payload from AlertData. Used by both webhook and history logging.
pub fn build_alert_payload(alert: &AlertData, escape_text: bool) -> serde_json::Value {
    use serde_json::json;

    let market_title = if escape_text {
        alert.market_title.map(webhook::escape_special_chars)
    } else {
        alert.market_title.map(|s| s.to_string())
    };

    let outcome = if escape_text {
        alert.outcome.map(webhook::escape_special_chars)
    } else {
        alert.outcome.map(|s| s.to_string())
    };

    let mut payload = json!({
        "platform": alert.platform,
        "alert_type": alert.alert_type(),
        "action": alert.side.to_uppercase(),
        "value": alert.value,
        "price": alert.price,
        "price_percent": (alert.price * 100.0).round() as i32,
        "size": alert.size,
        "timestamp": alert.timestamp,
        "market_title": market_title,
        "outcome": outcome,
    });

    if let Some(wallet) = alert.wallet_id {
        payload["wallet_id"] = json!(wallet);
    }

    if let Some(activity) = alert.wallet_activity {
        payload["wallet_activity"] = json!({
            "transactions_last_hour": activity.transactions_last_hour,
            "transactions_last_day": activity.transactions_last_day,
            "total_value_hour": activity.total_value_hour,
            "total_value_day": activity.total_value_day,
            "is_repeat_actor": activity.is_repeat_actor,
            "is_heavy_actor": activity.is_heavy_actor,
        });
    }

    if let Some(ctx) = alert.market_context {
        payload["market_context"] = json!({
            "yes_price": ctx.yes_price,
            "no_price": ctx.no_price,
            "spread": ctx.spread,
            "volume_24h": ctx.volume_24h,
            "open_interest": ctx.open_interest,
            "price_change_24h": ctx.price_change_24h,
            "liquidity": ctx.liquidity,
            "tags": ctx.tags,
        });
        if let Some(u) = &ctx.url {
            payload["market_context"]["url"] = json!(u);
        }
    }

    if let Some(wp) = alert.whale_profile {
        let mut wp_json = json!({});
        if let Some(v) = wp.portfolio_value { wp_json["portfolio_value"] = json!(v); }
        if let Some(r) = wp.leaderboard_rank { wp_json["leaderboard_rank"] = json!(r); }
        if let Some(p) = wp.leaderboard_profit { wp_json["leaderboard_profit"] = json!(p); }
        if let Some(w) = wp.win_rate { wp_json["win_rate"] = json!(w); }
        if let Some(m) = wp.markets_traded { wp_json["markets_traded"] = json!(m); }
        if let Some(c) = wp.positions_count { wp_json["positions_count"] = json!(c); }
        payload["whale_profile"] = wp_json;
    }

    if let Some(ob) = alert.order_book {
        payload["order_book"] = json!({
            "best_bid": ob.best_bid,
            "best_ask": ob.best_ask,
            "bid_depth_10pct": ob.bid_depth_10pct,
            "ask_depth_10pct": ob.ask_depth_10pct,
            "bid_levels": ob.bid_levels,
            "ask_levels": ob.ask_levels,
        });
    }

    if let Some(th) = alert.top_holders {
        let holders: Vec<serde_json::Value> = th.top_holders.iter().map(|h| {
            json!({
                "wallet": h.wallet,
                "shares": h.shares,
                "value": h.value,
            })
        }).collect();
        payload["top_holders"] = json!({
            "holders": holders,
            "total_shares": th.total_shares,
        });
    }

    payload
}
