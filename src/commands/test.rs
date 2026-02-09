use colored::*;

use crate::alerts::sound;
use crate::alerts::{AlertData};
use crate::alerts::webhook;
use crate::types;

pub async fn test_sound() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "TESTING ALERT SOUND".bright_cyan().bold());
    println!();
    println!("Playing single alert...");
    sound::play_alert_sound();

    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    println!("Playing triple alert (for repeat actors)...");
    sound::play_triple_beep();

    println!();
    println!("{}", "Sound test complete.".bright_green());
    println!("If you didn't hear anything, check:");
    println!("  1. System volume is not muted");
    println!("  2. Sound file exists: /System/Library/Sounds/Ping.aiff");
    println!("  3. Try: afplay /System/Library/Sounds/Ping.aiff");

    Ok(())
}

pub async fn test_webhook() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "TESTING WEBHOOK".bright_cyan().bold());
    println!();

    let config = match crate::config::load_config() {
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

    let test_activity = types::WalletActivity {
        transactions_last_hour: 2,
        transactions_last_day: 5,
        total_value_hour: 125000.0,
        total_value_day: 380000.0,
        is_repeat_actor: true,
        is_heavy_actor: true,
    };

    // Test BUY alert
    let timestamp = chrono::Utc::now().to_rfc3339();
    let buy_alert = AlertData {
        platform: "Polymarket",
        market_title: Some("Will Bitcoin reach $100k by end of 2026?"),
        outcome: Some("Yes"),
        side: "BUY",
        value: 50000.0,
        price: 0.65,
        size: 76923.08,
        timestamp: &timestamp,
        wallet_id: Some("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb"),
        wallet_activity: Some(&test_activity),
    };
    webhook::send_webhook_alert(&webhook_url, &buy_alert).await;

    println!("Test BUY alert sent!");

    // Test SELL alert
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    let timestamp2 = chrono::Utc::now().to_rfc3339();
    let sell_alert = AlertData {
        platform: "Kalshi",
        market_title: Some("Bitcoin price on Jan 16, 2026?"),
        outcome: Some("Bitcoin (BTC) price < $96999.99 at expiry"),
        side: "SELL",
        value: 35000.0,
        price: 0.54,
        size: 64814.81,
        timestamp: &timestamp2,
        wallet_id: None,
        wallet_activity: None,
    };
    webhook::send_webhook_alert(&webhook_url, &sell_alert).await;

    println!("Test SELL alert sent!");
    println!();
    println!("{}", "Test webhooks sent!".bright_green());
    println!("Check your n8n workflow to see if it received the data.");
    println!();
    println!("The webhooks should receive JSON payloads with:");
    println!("  Test 1 - Polymarket BUY:");
    println!("    - alert_type: WHALE_ENTRY");
    println!("    - action: BUY");
    println!("    - value: $50,000");
    println!("  Test 2 - Kalshi SELL:");
    println!("    - alert_type: WHALE_EXIT");
    println!("    - action: SELL");
    println!("    - value: $35,000");

    Ok(())
}
