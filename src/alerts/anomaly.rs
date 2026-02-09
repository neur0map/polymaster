use crate::types;
use colored::*;

use super::sound;

pub fn detect_anomalies(
    price: f64,
    size: f64,
    value: f64,
    wallet_activity: Option<&types::WalletActivity>,
) {
    let mut anomalies = Vec::new();

    if let Some(activity) = wallet_activity {
        if activity.is_heavy_actor {
            anomalies.push(format!(
                "HEAVY ACTOR: {} transactions worth ${:.2} in last 24h",
                activity.transactions_last_day, activity.total_value_day
            ));
        }
        if activity.is_repeat_actor && !activity.is_heavy_actor {
            anomalies.push(format!(
                "Repeat actor: {} transactions in last hour",
                activity.transactions_last_hour
            ));
        }
        if activity.total_value_hour > 200000.0 {
            anomalies.push(format!(
                "Coordinated activity: ${:.0} volume in past hour",
                activity.total_value_hour
            ));
        }
    }

    if price > 0.95 {
        anomalies.push(format!(
            "Extreme confidence bet ({:.1}% probability)",
            price * 100.0
        ));
    } else if price < 0.05 {
        anomalies.push(format!(
            "Contrarian position ({:.1}% probability)",
            price * 100.0
        ));
    }

    if size > 100000.0 {
        anomalies.push("Exceptionally large position size".to_string());
    }

    if value > 100000.0 {
        anomalies.push(format!("Major capital deployment: ${:.0}", value));
    }

    if price > 0.90 && size > 50000.0 {
        anomalies.push("High conviction in likely outcome".to_string());
    }

    if price < 0.20 && value > 50000.0 {
        anomalies.push(
            "Significant bet on unlikely outcome - possible hedge or information asymmetry"
                .to_string(),
        );
    }

    if !anomalies.is_empty() {
        sound::play_anomaly_sound();

        println!();
        println!("{}", "[ANOMALY INDICATORS]".bright_red().bold());
        for anomaly in anomalies {
            println!("  - {}", anomaly.yellow());
        }
    }
}
