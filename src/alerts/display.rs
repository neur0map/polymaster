use colored::*;

use crate::platforms::{kalshi, polymarket};
use crate::types;

use super::anomaly;
use super::sound;

pub fn print_whale_alert(
    platform: &str,
    trade: &polymarket::Trade,
    value: f64,
    wallet_activity: Option<&types::WalletActivity>,
) {
    let is_sell = trade.side.to_uppercase() == "SELL";

    // Enhanced alert sound for repeat actors or sells
    if let Some(activity) = wallet_activity {
        if activity.is_repeat_actor || activity.is_heavy_actor {
            sound::play_triple_beep();
        } else {
            sound::play_alert_sound();
        }
    } else {
        sound::play_alert_sound();
    }

    println!();

    // Enhanced header for repeat actors or exits
    let header = if is_sell {
        if let Some(activity) = wallet_activity {
            if activity.is_heavy_actor {
                format!("[HIGH PRIORITY] WHALE EXITING POSITION - {}", platform)
            } else if activity.is_repeat_actor {
                format!("[ELEVATED ALERT] WHALE EXITING POSITION - {}", platform)
            } else {
                format!("[ALERT] WHALE EXITING POSITION - {}", platform)
            }
        } else {
            format!("[ALERT] WHALE EXITING POSITION - {}", platform)
        }
    } else if let Some(activity) = wallet_activity {
        if activity.is_heavy_actor {
            format!("[HIGH PRIORITY ALERT] REPEAT HEAVY ACTOR - {}", platform)
        } else if activity.is_repeat_actor {
            format!("[ELEVATED ALERT] REPEAT ACTOR - {}", platform)
        } else {
            format!("[ALERT] LARGE TRANSACTION DETECTED - {}", platform)
        }
    } else {
        format!("[ALERT] LARGE TRANSACTION DETECTED - {}", platform)
    };

    println!("{}", header.bright_red().bold());
    println!("{}", "=".repeat(70).dimmed());

    if let Some(ref title) = trade.market_title {
        println!("Question:   {}", title.bright_white().bold());

        if let Some(ref outcome) = trade.outcome {
            let action = if trade.side.to_uppercase() == "BUY" {
                format!("BUYING '{}' shares", outcome)
            } else {
                format!("SELLING '{}' shares (EXITING POSITION)", outcome)
            };
            let action_color = if trade.side.to_uppercase() == "SELL" {
                action.bright_red().bold()
            } else {
                action.bright_yellow().bold()
            };
            println!("Position:   {}", action_color);
            println!(
                "Prediction: Market believes '{}' has {:.1}% chance",
                outcome,
                trade.price * 100.0
            );
        }
    } else {
        println!(
            "Market:     Unknown (ID: {})",
            &trade.market[..20.min(trade.market.len())]
        );
    }

    println!();
    println!("{}", "TRANSACTION DETAILS".dimmed());
    println!(
        "Amount:     {}",
        format!("${:.2}", value).bright_yellow().bold()
    );
    println!("Contracts:  {:.2} @ ${:.4} each", trade.size, trade.price);
    let action_text = if is_sell {
        format!("{} shares", trade.side.to_uppercase()).bright_red()
    } else {
        format!("{} shares", trade.side.to_uppercase()).bright_magenta()
    };
    println!("Action:     {}", action_text);
    println!("Timestamp:  {}", trade.timestamp);

    if let Some(activity) = wallet_activity {
        if let Some(ref wallet_id) = trade.wallet_id {
            println!();
            println!("{}", "[WALLET ACTIVITY]".bright_cyan().bold());
            println!(
                "Wallet:   {}...{}",
                &wallet_id[..8.min(wallet_id.len())],
                if wallet_id.len() > 8 {
                    &wallet_id[wallet_id.len() - 6..]
                } else {
                    ""
                }
            );
            println!("Txns (1h):  {}", activity.transactions_last_hour);
            println!("Txns (24h): {}", activity.transactions_last_day);
            println!("Volume (1h):  ${:.2}", activity.total_value_hour);
            println!("Volume (24h): ${:.2}", activity.total_value_day);

            if activity.is_heavy_actor {
                println!(
                    "{}",
                    "Status: HEAVY ACTOR (5+ transactions in 24h)"
                        .bright_red()
                        .bold()
                );
            } else if activity.is_repeat_actor {
                println!(
                    "{}",
                    "Status: REPEAT ACTOR (multiple transactions detected)"
                        .yellow()
                        .bold()
                );
            }
        }
    }

    anomaly::detect_anomalies(trade.price, trade.size, value, wallet_activity);

    println!("Asset ID: {}", trade.asset_id.dimmed());
    println!("{}", "=".repeat(70).dimmed());
    println!();
}

pub fn print_kalshi_alert(
    trade: &kalshi::Trade,
    value: f64,
    wallet_activity: Option<&types::WalletActivity>,
) {
    let is_sell = trade.taker_side.to_lowercase() == "sell";

    if is_sell {
        sound::play_triple_beep();
    } else if let Some(activity) = wallet_activity {
        if activity.is_repeat_actor || activity.is_heavy_actor {
            sound::play_triple_beep();
        } else {
            sound::play_alert_sound();
        }
    } else {
        sound::play_alert_sound();
    }

    println!();

    let header = if is_sell {
        if let Some(activity) = wallet_activity {
            if activity.is_heavy_actor {
                "[HIGH PRIORITY] WHALE EXITING POSITION - Kalshi"
                    .bright_red()
                    .bold()
            } else if activity.is_repeat_actor {
                "[ELEVATED ALERT] WHALE EXITING POSITION - Kalshi"
                    .bright_red()
                    .bold()
            } else {
                "[ALERT] WHALE EXITING POSITION - Kalshi"
                    .bright_red()
                    .bold()
            }
        } else {
            "[ALERT] WHALE EXITING POSITION - Kalshi"
                .bright_red()
                .bold()
        }
    } else if let Some(activity) = wallet_activity {
        if activity.is_heavy_actor {
            "[HIGH PRIORITY ALERT] REPEAT HEAVY ACTOR - Kalshi"
                .bright_green()
                .bold()
        } else if activity.is_repeat_actor {
            "[ELEVATED ALERT] REPEAT ACTOR - Kalshi"
                .bright_green()
                .bold()
        } else {
            "[ALERT] LARGE TRANSACTION DETECTED - Kalshi"
                .bright_green()
                .bold()
        }
    } else {
        "[ALERT] LARGE TRANSACTION DETECTED - Kalshi"
            .bright_green()
            .bold()
    };

    println!("{}", header);
    println!("{}", "=".repeat(70).dimmed());

    if let Some(ref title) = trade.market_title {
        println!("Question:   {}", title.bright_white().bold());
    }

    let bet_details = kalshi::parse_ticker_details(&trade.ticker, &trade.taker_side);
    let bet_color = if is_sell {
        bet_details.bright_red().bold()
    } else {
        bet_details.bright_yellow().bold()
    };
    println!("Position:   {}", bet_color);

    let direction_text = if is_sell {
        format!(
            "{} (EXITING {} position)",
            trade.taker_side.to_uppercase(),
            trade.taker_side.to_uppercase()
        )
    } else {
        format!(
            "{} (buying {} outcome)",
            trade.taker_side.to_uppercase(),
            trade.taker_side.to_uppercase()
        )
    };
    let direction_color = if is_sell {
        direction_text.bright_red()
    } else {
        direction_text.bright_magenta()
    };
    println!("Direction:  {}", direction_color);

    println!();
    println!("{}", "TRANSACTION DETAILS".dimmed());
    println!(
        "Amount:     {}",
        format!("${:.2}", value).bright_yellow().bold()
    );
    println!(
        "Contracts:  {} @ ${:.2} avg",
        trade.count,
        value / trade.count as f64
    );
    println!(
        "Odds:       YES: {:.1}% | NO: {:.1}%",
        trade.yes_price, trade.no_price
    );
    println!("Timestamp:  {}", trade.created_time);
    println!();
    println!("{}", format!("Ticker: {}", trade.ticker).dimmed());

    if let Some(activity) = wallet_activity {
        println!();
        println!("{}", "[WALLET ACTIVITY]".bright_cyan().bold());
        println!("Note: Kalshi public API doesn't expose wallet IDs, but patterns suggest:");
        println!("Txns (1h):  {}", activity.transactions_last_hour);
        println!("Txns (24h): {}", activity.transactions_last_day);
        println!("Volume (1h):  ${:.2}", activity.total_value_hour);
        println!("Volume (24h): ${:.2}", activity.total_value_day);

        if activity.is_heavy_actor {
            println!(
                "{}",
                "Status: HEAVY ACTOR (5+ transactions in 24h)"
                    .bright_red()
                    .bold()
            );
        } else if activity.is_repeat_actor {
            println!(
                "{}",
                "Status: REPEAT ACTOR (multiple transactions detected)"
                    .yellow()
                    .bold()
            );
        }
    }

    let avg_price = (trade.yes_price + trade.no_price) / 2.0;
    anomaly::detect_anomalies(avg_price / 100.0, trade.count as f64, value, wallet_activity);

    println!("{}", "=".repeat(70).dimmed());
    println!();
}

pub fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.insert(0, ',');
        }
        result.insert(0, ch);
    }
    result
}
