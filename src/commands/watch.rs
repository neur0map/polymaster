use std::time::Duration;

use colored::*;
use tokio::time;

use crate::alerts::AlertData;
use crate::alerts::display::{format_number, print_kalshi_alert, print_whale_alert};
use crate::alerts::history;
use crate::alerts::webhook;
use crate::platforms::kalshi;
use crate::platforms::polymarket;
use crate::types;

pub async fn watch_whales(threshold: u64, interval: u64) -> Result<(), Box<dyn std::error::Error>> {
    // Display disclaimer
    println!("{}", "=".repeat(70).bright_yellow());
    println!("{}", "DISCLAIMER".bright_yellow().bold());
    println!("This tool is for informational and research purposes only.");
    println!("I do not condone gambling or speculative trading.");
    println!("Use this data solely for informed decision-making and market analysis.");
    println!("Trade responsibly and within your means.");
    println!("{}", "=".repeat(70).bright_yellow());
    println!();

    println!("{}", "WHALE WATCHER ACTIVE".bright_cyan().bold());
    println!(
        "Threshold: {}",
        format!("${}", format_number(threshold)).bright_green()
    );
    println!("Interval:  {} seconds", interval);

    let config = crate::config::load_config().ok();

    if let Some(ref cfg) = config {
        if cfg.webhook_url.is_some() {
            println!("Webhook:   {}", "Enabled".bright_green());
        }
    }

    println!();

    let mut last_polymarket_trade_id: Option<String> = None;
    let mut last_kalshi_trade_id: Option<String> = None;

    let mut wallet_tracker = types::WalletTracker::new();

    let mut tick_interval = time::interval(Duration::from_secs(interval));

    loop {
        tick_interval.tick().await;

        // Check Polymarket
        match polymarket::fetch_recent_trades().await {
            Ok(mut trades) => {
                if let Some(first_trade) = trades.first() {
                    let new_last_id = first_trade.id.clone();

                    for trade in &mut trades {
                        if let Some(ref last_id) = last_polymarket_trade_id {
                            if trade.id == *last_id {
                                break;
                            }
                        }

                        let trade_value = trade.size * trade.price;
                        if trade_value >= threshold as f64 {
                            let wallet_activity = if let Some(ref wallet_id) = trade.wallet_id {
                                wallet_tracker.record_transaction(wallet_id, trade_value);
                                Some(wallet_tracker.get_activity(wallet_id))
                            } else {
                                None
                            };

                            print_whale_alert(
                                "Polymarket",
                                trade,
                                trade_value,
                                wallet_activity.as_ref(),
                            );

                            let alert_data = AlertData {
                                platform: "Polymarket",
                                market_title: trade.market_title.as_deref(),
                                outcome: trade.outcome.as_deref(),
                                side: &trade.side,
                                value: trade_value,
                                price: trade.price,
                                size: trade.size,
                                timestamp: &trade.timestamp,
                                wallet_id: trade.wallet_id.as_deref(),
                                wallet_activity: wallet_activity.as_ref(),
                            };

                            history::log_alert(&alert_data);

                            if let Some(ref cfg) = config {
                                if let Some(ref webhook_url) = cfg.webhook_url {
                                    webhook::send_webhook_alert(webhook_url, &alert_data).await;
                                }
                            }
                        }
                    }

                    last_polymarket_trade_id = Some(new_last_id);
                }
            }
            Err(e) => {
                eprintln!("{} {}", "[ERROR] Polymarket:".red(), e);
            }
        }

        // Check Kalshi
        match kalshi::fetch_recent_trades(config.as_ref()).await {
            Ok(mut trades) => {
                if let Some(first_trade) = trades.first() {
                    let new_last_id = first_trade.trade_id.clone();

                    for trade in &mut trades {
                        if let Some(ref last_id) = last_kalshi_trade_id {
                            if trade.trade_id == *last_id {
                                break;
                            }
                        }

                        let trade_value = (trade.yes_price / 100.0) * f64::from(trade.count);
                        if trade_value >= threshold as f64 {
                            if let Some(title) = kalshi::fetch_market_info(&trade.ticker).await {
                                trade.market_title = Some(title);
                            }

                            let outcome =
                                kalshi::parse_ticker_details(&trade.ticker, &trade.taker_side);

                            let action = trade.taker_side.to_uppercase();

                            print_kalshi_alert(trade, trade_value, None);

                            let alert_data = AlertData {
                                platform: "Kalshi",
                                market_title: trade.market_title.as_deref(),
                                outcome: Some(&outcome),
                                side: &action,
                                value: trade_value,
                                price: trade.yes_price / 100.0,
                                size: f64::from(trade.count),
                                timestamp: &trade.created_time,
                                wallet_id: None,
                                wallet_activity: None,
                            };

                            history::log_alert(&alert_data);

                            if let Some(ref cfg) = config {
                                if let Some(ref webhook_url) = cfg.webhook_url {
                                    webhook::send_webhook_alert(webhook_url, &alert_data).await;
                                }
                            }
                        }
                    }

                    last_kalshi_trade_id = Some(new_last_id);
                }
            }
            Err(e) => {
                eprintln!("{} {}", "[ERROR] Kalshi:".red(), e);
            }
        }
    }
}
