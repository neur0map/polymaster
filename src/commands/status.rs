use colored::*;
use rusqlite::Connection;

use crate::db;

pub async fn show_status(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
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
            let cat_display = if cfg.categories.is_empty() || cfg.categories.iter().any(|s| s == "all") {
                "All markets".to_string()
            } else {
                cfg.categories.join(", ")
            };
            println!("  Categories:    {}", cat_display.green());
            println!("  Threshold:     {}", format!("${}", cfg.threshold).green());
            println!(
                "  Retention:     {}",
                if cfg.history_retention_days == 0 {
                    "Forever".to_string()
                } else {
                    format!("{} days", cfg.history_retention_days)
                }
                .green()
            );
        }
        Err(_) => {
            println!("No configuration found. Run 'wwatcher setup' to configure.");
        }
    }

    println!();
    println!("Database:");
    let alert_count = db::alert_count(conn);
    println!("  Alerts stored: {}", alert_count.to_string().bright_white());
    if let Ok(path) = db::db_path() {
        println!("  Location: {}", path.display().to_string().dimmed());
    }

    Ok(())
}
