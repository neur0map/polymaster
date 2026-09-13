mod alerts;
mod categories;
mod commands;
mod config;
mod db;
mod platforms;
mod types;
mod whale_profile;
mod ws;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "poly")]
#[command(about = "poly - Monitor whale activity on Polymarket and Kalshi", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Watch for large transactions (default threshold: $25,000)
    Watch {
        /// Minimum transaction size to alert on (in USD, overrides config)
        #[arg(short, long)]
        threshold: Option<u64>,

        /// Polling interval in seconds (overrides config)
        #[arg(short, long)]
        interval: Option<u64>,
    },
    /// View alert history
    History {
        /// Number of alerts to show (default: 20)
        #[arg(short, long, default_value = "20")]
        limit: usize,

        /// Filter by platform: polymarket, kalshi, or all (default: all)
        #[arg(short, long, default_value = "all")]
        platform: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,
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

    // Initialize database for commands that need it
    let conn = db::open_db()?;

    // Migrate old JSONL history on first run
    db::migrate_jsonl_if_exists(&conn);

    match cli.command {
        Commands::Setup => {
            commands::setup::setup_config().await?;
        }
        Commands::Status => {
            commands::status::show_status(&conn).await?;
        }
        Commands::Watch {
            threshold,
            interval,
        } => {
            let config = config::load_config().unwrap_or_default();
            let threshold = threshold.unwrap_or(config.threshold);
            let interval = interval.unwrap_or(config.interval_secs).max(1);
            commands::watch::watch_whales(threshold, interval, conn).await?;
        }
        Commands::History {
            limit,
            platform,
            json,
        } => {
            alerts::history::show_alert_history(limit, &platform, json, &conn)?;
        }
        Commands::TestSound => {
            commands::test::test_sound().await?;
        }
        Commands::TestWebhook => {
            commands::test::test_webhook().await?;
        }
    }

    Ok(())
}
