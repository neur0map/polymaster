use colored::*;

pub async fn show_status() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "WHALE WATCHER STATUS".bright_cyan().bold());
    println!();

    match crate::config::load_config() {
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
