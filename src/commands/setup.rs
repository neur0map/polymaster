use std::io::{self, Write};

use colored::*;

pub async fn setup_config() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "WHALE WATCHER SETUP".bright_cyan().bold());
    println!();

    println!("This tool monitors large transactions on Polymarket and Kalshi.");
    println!("API credentials are optional - the tool works with public data.");
    println!();

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

    let config = crate::config::Config {
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

    crate::config::save_config(&config)?;

    println!("{}", "Configuration saved successfully.".bright_green());
    println!();
    println!(
        "Run {} to start watching for whale transactions.",
        "wwatcher watch".bright_cyan()
    );

    Ok(())
}
