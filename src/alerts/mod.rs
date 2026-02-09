pub mod anomaly;
pub mod display;
pub mod history;
pub mod sound;
pub mod webhook;

use crate::types;

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

    payload
}
