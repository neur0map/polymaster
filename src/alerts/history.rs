use std::io::Write;

use colored::*;

use super::AlertData;

pub fn get_history_file_path() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let config_dir = dirs::config_dir().ok_or("Could not determine config directory")?;
    let wwatcher_dir = config_dir.join("wwatcher");
    std::fs::create_dir_all(&wwatcher_dir)?;
    Ok(wwatcher_dir.join("alert_history.jsonl"))
}

pub fn log_alert(alert: &AlertData) {
    if let Ok(history_file) = get_history_file_path() {
        let log_entry = super::build_alert_payload(alert, false);

        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&history_file)
        {
            if let Ok(json_line) = serde_json::to_string(&log_entry) {
                let _ = writeln!(file, "{}", json_line);
            }
        }
    }
}

pub fn show_alert_history(
    limit: usize,
    platform_filter: &str,
    as_json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use serde_json::Value;

    let history_file = get_history_file_path()?;

    if !history_file.exists() {
        println!("No alert history found.");
        println!(
            "Run {} to start monitoring and logging alerts.",
            "wwatcher watch".bright_cyan()
        );
        return Ok(());
    }

    let contents = std::fs::read_to_string(&history_file)?;
    let mut alerts: Vec<Value> = contents
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();

    if platform_filter != "all" {
        let filter_lower = platform_filter.to_lowercase();
        alerts.retain(|alert| {
            alert
                .get("platform")
                .and_then(|p| p.as_str())
                .map(|p| p.to_lowercase() == filter_lower)
                .unwrap_or(false)
        });
    }

    alerts.reverse();

    let alerts_to_show: Vec<&Value> = alerts.iter().take(limit).collect();

    if alerts_to_show.is_empty() {
        println!("No alerts found matching filters.");
        return Ok(());
    }

    if as_json {
        println!("{}", serde_json::to_string_pretty(&alerts_to_show)?);
    } else {
        println!("{}", "ALERT HISTORY".bright_cyan().bold());
        println!("Showing {} most recent alerts", alerts_to_show.len());
        if platform_filter != "all" {
            println!("Platform filter: {}", platform_filter);
        }
        println!();

        for (i, alert) in alerts_to_show.iter().enumerate() {
            let platform = alert
                .get("platform")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown");
            let alert_type = alert
                .get("alert_type")
                .and_then(|v| v.as_str())
                .unwrap_or("UNKNOWN");
            let action = alert
                .get("action")
                .and_then(|v| v.as_str())
                .unwrap_or("UNKNOWN");
            let value = alert.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let timestamp = alert
                .get("timestamp")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown");
            let market_title = alert
                .get("market_title")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown market");
            let outcome = alert.get("outcome").and_then(|v| v.as_str());

            let header = format!("#{} | {} | {}", i + 1, platform, alert_type);
            println!("{}", header.bright_yellow());
            println!("Time:   {}", timestamp.dimmed());
            println!("Market: {}", market_title);
            if let Some(out) = outcome {
                println!("Outcome: {}", out);
            }
            println!("Action: {} | Value: ${:.2}", action, value);

            if let Some(wallet_activity) = alert.get("wallet_activity") {
                if let Some(txns_hour) = wallet_activity
                    .get("transactions_last_hour")
                    .and_then(|v| v.as_u64())
                {
                    if txns_hour > 1 {
                        println!("Wallet: {} txns in last hour", txns_hour);
                    }
                }
            }

            println!();
        }

        println!(
            "View as JSON: {} --json",
            "wwatcher history".bright_cyan()
        );
        println!(
            "Filter by platform: {} --platform polymarket",
            "wwatcher history".bright_cyan()
        );
    }

    Ok(())
}
