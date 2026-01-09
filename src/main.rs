mod config;
mod kalshi;
mod polymarket;
mod types;

use clap::{Parser, Subcommand};
use colored::*;
use std::io::{self, Write};
use std::time::Duration;
use tokio::time;

#[derive(Parser)]
#[command(name = "wwatcher")]
#[command(about = "Whale Watcher - Monitor large transactions on Polymarket and Kalshi", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Watch for large transactions (default threshold: $25,000)
    Watch {
        /// Minimum transaction size to alert on (in USD)
        #[arg(short, long, default_value = "25000")]
        threshold: u64,

        /// Polling interval in seconds
        #[arg(short, long, default_value = "5")]
        interval: u64,
    },
    /// Configure API credentials
    Setup,
    /// Show current configuration
    Status,
    /// Test alert sound
    TestSound,
    /// Test webhook notification
    TestWebhook,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Setup => {
            setup_config().await?;
        }
        Commands::Status => {
            show_status().await?;
        }
        Commands::Watch {
            threshold,
            interval,
        } => {
            watch_whales(threshold, interval).await?;
        }
        Commands::TestSound => {
            test_sound().await?;
        }
        Commands::TestWebhook => {
            test_webhook().await?;
        }
    }

    Ok(())
}

async fn setup_config() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "WHALE WATCHER SETUP".bright_cyan().bold());
    println!();

    println!("This tool monitors large transactions on Polymarket and Kalshi.");
    println!("API credentials are optional - the tool works with public data.");
    println!();

    // Get Kalshi credentials (optional)
    println!("{}", "Kalshi Configuration (optional):".bright_yellow());
    println!("Generate API keys at: https://kalshi.com/profile/api-keys");
    println!("Press Enter to skip if you don't have credentials.");
    println!();

    print!("Enter Kalshi API Key ID (or press Enter to skip): ");
    io::stdout().flush()?;
    let mut kalshi_key_id = String::new();
    std::io::stdin().read_line(&mut kalshi_key_id)?;
    let kalshi_key_id = kalshi_key_id.trim().to_string();

    let kalshi_private_key = if !kalshi_key_id.is_empty() {
        print!("Enter Kalshi Private Key: ");
        io::stdout().flush()?;
        let mut key = String::new();
        std::io::stdin().read_line(&mut key)?;
        key.trim().to_string()
    } else {
        println!("Skipping Kalshi API configuration.");
        String::new()
    };

    println!();
    println!("{}", "Webhook Configuration (optional):".bright_yellow());
    println!("Send alerts to a webhook URL (works with n8n, Zapier, Make, etc.)");
    println!("Example: https://your-n8n-instance.com/webhook/whale-alerts");
    println!();

    print!("Enter Webhook URL (or press Enter to skip): ");
    io::stdout().flush()?;
    let mut webhook_url = String::new();
    std::io::stdin().read_line(&mut webhook_url)?;
    let webhook_url = webhook_url.trim().to_string();

    if webhook_url.is_empty() {
        println!("Skipping webhook configuration.");
    } else {
        println!("Webhook configured: {}", webhook_url.bright_green());
    }

    println!();

    // Save configuration
    let config = config::Config {
        kalshi_api_key_id: if kalshi_key_id.is_empty() {
            None
        } else {
            Some(kalshi_key_id)
        },
        kalshi_private_key: if kalshi_private_key.is_empty() {
            None
        } else {
            Some(kalshi_private_key)
        },
        webhook_url: if webhook_url.is_empty() {
            None
        } else {
            Some(webhook_url)
        },
    };

    config::save_config(&config)?;

    println!("{}", "Configuration saved successfully.".bright_green());
    println!();
    println!(
        "Run {} to start watching for whale transactions.",
        "wwatcher watch".bright_cyan()
    );

    Ok(())
}

async fn test_sound() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "TESTING ALERT SOUND".bright_cyan().bold());
    println!();
    println!("Playing single alert...");
    play_alert_sound();

    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    println!("Playing triple alert (for repeat actors)...");
    play_alert_sound();
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    play_alert_sound();
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    play_alert_sound();

    println!();
    println!("{}", "Sound test complete.".bright_green());
    println!("If you didn't hear anything, check:");
    println!("  1. System volume is not muted");
    println!("  2. Sound file exists: /System/Library/Sounds/Ping.aiff");
    println!("  3. Try: afplay /System/Library/Sounds/Ping.aiff");

    Ok(())
}

async fn test_webhook() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "TESTING WEBHOOK".bright_cyan().bold());
    println!();

    let config = match config::load_config() {
        Ok(cfg) => cfg,
        Err(_) => {
            println!(
                "{}",
                "No configuration found. Run 'wwatcher setup' first.".red()
            );
            return Ok(());
        }
    };

    let webhook_url = match config.webhook_url {
        Some(url) => url,
        None => {
            println!(
                "{}",
                "No webhook configured. Run 'wwatcher setup' to add a webhook URL.".red()
            );
            return Ok(());
        }
    };

    println!("Sending test alert to: {}", webhook_url.bright_green());
    println!();

    // Create a test alert
    let test_activity = types::WalletActivity {
        transactions_last_hour: 2,
        transactions_last_day: 5,
        total_value_hour: 125000.0,
        total_value_day: 380000.0,
        is_repeat_actor: true,
        is_heavy_actor: true,
    };

    send_webhook_alert(
        &webhook_url,
        WebhookAlert {
            platform: "Polymarket",
            market_title: Some("Will Bitcoin reach $100k by end of 2026?"),
            outcome: Some("Yes"),
            side: "BUY",
            value: 50000.0,
            price: 0.65,
            size: 76923.08,
            timestamp: &chrono::Utc::now().to_rfc3339(),
            wallet_id: Some("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb"),
            wallet_activity: Some(&test_activity),
        },
    )
    .await;

    println!();
    println!("{}", "Test webhook sent!".bright_green());
    println!("Check your n8n workflow to see if it received the data.");
    println!();
    println!("The webhook should receive a JSON payload with:");
    println!("  - platform: Polymarket");
    println!("  - alert_type: WHALE_ENTRY");
    println!("  - action: BUY");
    println!("  - value: $50,000");
    println!("  - Wallet activity with repeat actor flag");

    Ok(())
}

async fn show_status() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "WHALE WATCHER STATUS".bright_cyan().bold());
    println!();

    match config::load_config() {
        Ok(cfg) => {
            println!("Configuration:");
            println!(
                "  Kalshi API: {}",
                if cfg.kalshi_api_key_id.is_some() {
                    "Configured".green()
                } else {
                    "Not configured (using public data)".yellow()
                }
            );
            println!(
                "  Polymarket API: {}",
                "Public access (no key needed)".green()
            );
            println!(
                "  Webhook: {}",
                if cfg.webhook_url.is_some() {
                    format!("Configured ({})", cfg.webhook_url.as_ref().unwrap()).green()
                } else {
                    "Not configured".yellow()
                }
            );
        }
        Err(_) => {
            println!("No configuration found. Run 'wwatcher setup' to configure.");
        }
    }

    Ok(())
}

async fn watch_whales(threshold: u64, interval: u64) -> Result<(), Box<dyn std::error::Error>> {
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

    // Load config (optional credentials)
    let config = config::load_config().ok();

    if let Some(ref cfg) = config {
        if cfg.webhook_url.is_some() {
            println!("Webhook:   {}", "Enabled".bright_green());
        }
    }

    println!();

    let mut last_polymarket_trade_id: Option<String> = None;
    let mut last_kalshi_trade_id: Option<String> = None;

    // Initialize wallet tracker
    let mut wallet_tracker = types::WalletTracker::new();

    let mut tick_interval = time::interval(Duration::from_secs(interval));

    loop {
        tick_interval.tick().await;

        // Check Polymarket
        match polymarket::fetch_recent_trades().await {
            Ok(mut trades) => {
                // Update last seen trade ID first
                if let Some(first_trade) = trades.first() {
                    let new_last_id = first_trade.id.clone();

                    for trade in &mut trades {
                        // Skip if we've already seen this trade
                        if let Some(ref last_id) = last_polymarket_trade_id {
                            if trade.id == *last_id {
                                break;
                            }
                        }

                        let trade_value = trade.size * trade.price;
                        if trade_value >= threshold as f64 {
                            // Fetch market details
                            if let Some((title, outcome)) =
                                polymarket::fetch_market_info(&trade.market).await
                            {
                                trade.market_title = Some(title);
                                trade.outcome = Some(outcome);
                            }

                            // Track wallet activity
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

                            // Send webhook notification
                            if let Some(ref cfg) = config {
                                if let Some(ref webhook_url) = cfg.webhook_url {
                                    send_webhook_alert(
                                        webhook_url,
                                        WebhookAlert {
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
                                        },
                                    )
                                    .await;
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
                // Update last seen trade ID first
                if let Some(first_trade) = trades.first() {
                    let new_last_id = first_trade.trade_id.clone();

                    for trade in &mut trades {
                        // Skip if we've already seen this trade
                        if let Some(ref last_id) = last_kalshi_trade_id {
                            if trade.trade_id == *last_id {
                                break;
                            }
                        }

                        // Kalshi prices are in cents, count is number of contracts
                        let trade_value = (trade.yes_price / 100.0) * f64::from(trade.count);
                        if trade_value >= threshold as f64 {
                            // Fetch market details
                            if let Some(title) = kalshi::fetch_market_info(&trade.ticker).await {
                                trade.market_title = Some(title);
                            }
                            
                            // Extract outcome from ticker
                            let outcome = kalshi::parse_ticker_details(&trade.ticker);
                            
                            // Note: Kalshi doesn't expose wallet IDs in public API
                            print_kalshi_alert(trade, trade_value, None);

                            // Send webhook notification
                            if let Some(ref cfg) = config {
                                if let Some(ref webhook_url) = cfg.webhook_url {
                                    send_webhook_alert(
                                        webhook_url,
                                        WebhookAlert {
                                            platform: "Kalshi",
                                            market_title: trade.market_title.as_deref(),
                                            outcome: Some(&outcome),
                                            side: &trade.taker_side,
                                            value: trade_value,
                                            price: trade.yes_price / 100.0,
                                            size: f64::from(trade.count),
                                            timestamp: &trade.created_time,
                                            wallet_id: None,
                                            wallet_activity: None,
                                        },
                                    )
                                    .await;
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

fn print_whale_alert(
    platform: &str,
    trade: &polymarket::Trade,
    value: f64,
    wallet_activity: Option<&types::WalletActivity>,
) {
    let is_sell = trade.side.to_uppercase() == "SELL";

    // Enhanced alert sound for repeat actors or sells
    if let Some(activity) = wallet_activity {
        if activity.is_repeat_actor || activity.is_heavy_actor {
            // Triple beep for repeat/heavy actors
            play_alert_sound();
            std::thread::sleep(std::time::Duration::from_millis(100));
            play_alert_sound();
            std::thread::sleep(std::time::Duration::from_millis(100));
            play_alert_sound();
        } else {
            play_alert_sound();
        }
    } else {
        play_alert_sound();
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

    // Display market title if available
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

    // Display wallet activity if available
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

    // Anomaly detection
    detect_anomalies(trade.price, trade.size, value, wallet_activity);

    println!("Asset ID: {}", trade.asset_id.dimmed());
    println!("{}", "=".repeat(70).dimmed());
    println!();
}

fn print_kalshi_alert(
    trade: &kalshi::Trade,
    value: f64,
    _wallet_activity: Option<&types::WalletActivity>,
) {
    let is_sell = trade.taker_side.to_lowercase() == "sell";

    // Play alert sound immediately
    play_alert_sound();

    println!();

    let header = if is_sell {
        "[ALERT] WHALE EXITING POSITION - Kalshi"
            .bright_red()
            .bold()
    } else {
        "[ALERT] LARGE TRANSACTION DETECTED - Kalshi"
            .bright_green()
            .bold()
    };

    println!("{}", header);
    println!("{}", "=".repeat(70).dimmed());

    // Display market title if available
    if let Some(ref title) = trade.market_title {
        println!("Question:   {}", title.bright_white().bold());
    }

    // Parse and display what the bet means
    let bet_details = kalshi::parse_ticker_details(&trade.ticker);
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

    // Anomaly detection
    let avg_price = (trade.yes_price + trade.no_price) / 2.0;
    detect_anomalies(avg_price / 100.0, trade.count as f64, value, None);

    println!("{}", "=".repeat(70).dimmed());
    println!();
}

fn play_alert_sound() {
    play_sound_internal("/System/Library/Sounds/Ping.aiff");
}

fn play_anomaly_sound() {
    // Use more attention-grabbing sound for anomalies
    play_sound_internal("/System/Library/Sounds/Funk.aiff");
}

fn play_sound_internal(sound_file: &str) {
    // macOS: Use afplay with system sound
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("afplay")
            .arg(sound_file)
            .spawn()
            .ok();
    }

    // Linux: Use paplay or aplay
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("paplay")
            .arg("/usr/share/sounds/freedesktop/stereo/message.oga")
            .spawn()
            .or_else(|_| {
                std::process::Command::new("aplay")
                    .arg("/usr/share/sounds/alsa/Front_Center.wav")
                    .spawn()
            })
            .ok();
    }

    // Windows: Use powershell beep
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("powershell")
            .arg("-c")
            .arg("[console]::beep(800,300)")
            .spawn()
            .ok();
    }

    // Fallback: terminal bell
    print!("\x07");
    io::stdout().flush().ok();
}

fn detect_anomalies(
    price: f64,
    size: f64,
    value: f64,
    wallet_activity: Option<&types::WalletActivity>,
) {
    let mut anomalies = Vec::new();

    // Wallet-based anomalies (highest priority)
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

    // Extreme confidence (very high or very low probability)
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

    // Unusual size relative to typical market activity
    if size > 100000.0 {
        anomalies.push("Exceptionally large position size".to_string());
    }

    // Very large single transaction
    if value > 100000.0 {
        anomalies.push(format!("Major capital deployment: ${:.0}", value));
    }

    // Edge case: betting on near-certain outcomes with large size
    if price > 0.90 && size > 50000.0 {
        anomalies.push("High conviction in likely outcome".to_string());
    }

    // Edge case: large bet on unlikely outcome (potential insider info or hedge)
    if price < 0.20 && value > 50000.0 {
        anomalies.push(
            "Significant bet on unlikely outcome - possible hedge or information asymmetry"
                .to_string(),
        );
    }

    // Display anomalies
    if !anomalies.is_empty() {
        // Play distinctive anomaly sound
        play_anomaly_sound();

        println!();
        println!("{}", "[ANOMALY INDICATORS]".bright_red().bold());
        for anomaly in anomalies {
            println!("  - {}", anomaly.yellow());
        }
    }
}

struct WebhookAlert<'a> {
    platform: &'a str,
    market_title: Option<&'a str>,
    outcome: Option<&'a str>,
    side: &'a str,
    value: f64,
    price: f64,
    size: f64,
    timestamp: &'a str,
    wallet_id: Option<&'a str>,
    wallet_activity: Option<&'a types::WalletActivity>,
}

// Escape special characters that might cause issues with HTML/Markdown parsers
// These characters have special meaning in Telegram/Discord/Slack markdown
fn escape_special_chars(s: &str) -> String {
    s.replace('&', "and")
        .replace('<', "less than")
        .replace('>', "greater than")
        .replace('≥', ">=")
        .replace('≤', "<=")
        .replace('°', " degrees")
        .replace('|', "-")
        .replace('[', "(")
        .replace(']', ")")
        .replace('_', " ")
        .replace('*', "")
        .replace('`', "'")
        .replace('~', "")
        .replace('#', "")
        .replace('+', "plus")
        .replace('-', " ")
        .replace('=', " equals ")
        .replace('{', "(")
        .replace('}', ")")
        .replace('!', "")
        .replace('.', " ")
}

async fn send_webhook_alert(webhook_url: &str, alert: WebhookAlert<'_>) {
    use serde_json::json;

    let is_sell = alert.side.to_uppercase() == "SELL";
    let alert_type = if is_sell { "WHALE_EXIT" } else { "WHALE_ENTRY" };

    let mut payload = json!({
        "platform": alert.platform,
        "alert_type": alert_type,
        "action": alert.side.to_uppercase(),
        "value": alert.value,
        "price": alert.price,
        "size": alert.size,
        "timestamp": alert.timestamp,
        "market_title": alert.market_title.map(escape_special_chars),
        "outcome": alert.outcome.map(escape_special_chars),
    });

    // Add wallet information if available
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

    // Send POST request to webhook
    // For self-hosted instances with self-signed certs, accept invalid certs
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap();

    match client.post(webhook_url).json(&payload).send().await {
        Ok(response) => {
            if !response.status().is_success() {
                eprintln!(
                    "{} Webhook failed with status: {}",
                    "[WEBHOOK ERROR]".red(),
                    response.status()
                );
            }
        }
        Err(e) => {
            eprintln!("{} Failed to send webhook: {}", "[WEBHOOK ERROR]".red(), e);
        }
    }
}

fn format_number(n: u64) -> String {
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
