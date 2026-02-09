mod alerts;
mod commands;
mod config;
mod platforms;
mod types;

use clap::{Parser, Subcommand};

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

    match cli.command {
        Commands::Setup => {
            commands::setup::setup_config().await?;
        }
        Commands::Status => {
            commands::status::show_status().await?;
        }
        Commands::Watch {
            threshold,
            interval,
        } => {
            commands::watch::watch_whales(threshold, interval).await?;
        }
        Commands::History {
            limit,
            platform,
            json,
        } => {
            alerts::history::show_alert_history(limit, &platform, json)?;
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
