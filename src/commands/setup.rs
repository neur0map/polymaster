use std::io::{self, Write};

use colored::*;

use crate::categories::CategoryRegistry;
use crate::config::Config;

fn read_line() -> String {
    let mut input = String::new();
    io::stdout().flush().unwrap();
    std::io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn thousands(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::new();
    let bytes = s.as_bytes();
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 && (bytes.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(*b as char);
    }
    out
}

fn saved_msg(what: &str) {
    println!("{}", format!("✓ Saved: {}", what).bright_green());
    println!();
}

fn display_menu(config: &Config) {
    let plat_display = if config.platforms.iter().any(|p| p == "all") {
        "Polymarket + Kalshi".to_string()
    } else {
        config
            .platforms
            .iter()
            .map(|p| {
                let mut c = p.chars();
                match c.next() {
                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                    None => String::new(),
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    };

    let cat_display = if config.categories.iter().any(|c| c == "all") {
        "All markets".to_string()
    } else {
        config.categories.join(", ")
    };

    let ret_display = if config.history_retention_days == 0 {
        "Forever".to_string()
    } else {
        format!("{} days", config.history_retention_days)
    };

    println!("{}", "Current settings:".bright_white().bold());
    println!();
    println!(
        "  [1] Platforms          {}",
        plat_display.bright_green()
    );
    println!(
        "  [2] Categories         {}",
        cat_display.bright_green()
    );
    println!(
        "  [3] Alert threshold    {}",
        format!("${}", thousands(config.threshold)).bright_green()
    );
    println!(
        "  [4] Poll interval      {}",
        format!("{} seconds", config.interval_secs).bright_green()
    );
    println!(
        "  [5] Max odds           {}",
        format!("{:.2}", config.max_odds).bright_green()
    );
    println!(
        "  [6] Min spread         {}",
        format!("{:.2}", config.min_spread).bright_green()
    );
    println!(
        "  [7] History retention  {}",
        ret_display.bright_green()
    );
    println!(
        "  [8] Kalshi API keys    {}",
        if config.kalshi_api_key_id.is_some() {
            "Configured".bright_green()
        } else {
            "Not set".dimmed()
        }
    );
    println!(
        "  [9] Webhook URL        {}",
        if config.webhook_url.is_some() {
            "Configured".bright_green()
        } else {
            "Not set".dimmed()
        }
    );
    println!();
    println!("  [t] Test notifications (sound + webhook)");
    println!("  [d] Reset all settings to defaults");
    println!("  [q] Done");
    println!();
    print!("{}", "Select a setting to change [1-9, t, d, q]: ".bright_cyan());
}

fn edit_platforms(config: &mut Config) {
    println!();
    println!("{}", "PLATFORMS".bright_white().bold());
    println!("Which prediction markets do you want to monitor?");
    println!();
    println!("  [1] Both Polymarket + Kalshi (recommended)");
    println!("  [2] Polymarket only");
    println!("  [3] Kalshi only");
    println!();
    let current = if config.platforms.iter().any(|p| p == "all") {
        "1"
    } else if config.platforms.contains(&"polymarket".into())
        && config.platforms.contains(&"kalshi".into())
    {
        "1"
    } else if config.platforms.contains(&"polymarket".into()) {
        "2"
    } else {
        "3"
    };
    print!("Select [1-3] (Enter for {}, current): ", current);
    let choice = read_line();
    let choice = if choice.is_empty() { current.to_string() } else { choice };
    config.platforms = match choice.as_str() {
        "2" => {
            println!("{}", "Monitoring Polymarket only".bright_green());
            vec!["polymarket".into()]
        }
        "3" => {
            println!("{}", "Monitoring Kalshi only".bright_green());
            vec!["kalshi".into()]
        }
        _ => {
            println!("{}", "Monitoring both platforms".bright_green());
            vec!["all".into()]
        }
    };
}

fn edit_categories(config: &mut Config) {
    println!();
    println!("{}", "MARKET CATEGORIES".bright_white().bold());
    println!("Choose which market categories to watch for alerts.");
    println!("Enter numbers separated by commas (e.g. 1,3,5), 0 for all.");
    println!("Press Enter to keep your current selection.");
    println!();

    let all_cats = CategoryRegistry::all_categories();
    println!("  [0] {} — Watch all markets (recommended)", "ALL".bright_green().bold());
    for (i, (_, label)) in all_cats.iter().enumerate() {
        println!("  [{}] {}", i + 1, label);
    }
    println!();

    let current_display = if config.categories.iter().any(|c| c == "all") {
        "all".to_string()
    } else {
        config.categories.join(", ")
    };
    println!("Current: {}", current_display.bright_green());
    print!("Select: ");
    let cat_input = read_line();

    if cat_input.is_empty() {
        return;
    }

    let categories: Vec<String> = if cat_input == "0" {
        vec!["all".into()]
    } else {
        let mut selected: Vec<String> = Vec::new();
        for idx_str in cat_input.split(',').map(|s| s.trim()) {
            if let Ok(idx) = idx_str.parse::<usize>() {
                if idx >= 1 && idx <= all_cats.len() {
                    let (cat_key, cat_label) = all_cats[idx - 1];
                    println!();
                    println!(
                        "{}",
                        format!("Subcategories for {}:", cat_label).bright_white().bold()
                    );
                    let subs = CategoryRegistry::subcategories(cat_key);
                    println!("  [0] All {}", cat_key);
                    for (j, (_, sub_label)) in subs.iter().enumerate() {
                        println!("  [{}] {}", j + 1, sub_label);
                    }
                    print!("Select subcategories (0 for all, or comma-separated): ");
                    let sub_input = read_line();

                    if sub_input.is_empty() || sub_input == "0" {
                        selected.push(format!("{}:all", cat_key));
                        println!("{}", format!("  Added: all {}", cat_key).bright_green());
                    } else {
                        for sidx_str in sub_input.split(',').map(|s| s.trim()) {
                            if let Ok(sidx) = sidx_str.parse::<usize>() {
                                if sidx >= 1 && sidx <= subs.len() {
                                    let (sub_key, sub_label) = subs[sidx - 1];
                                    selected.push(sub_key.to_string());
                                    println!(
                                        "{}",
                                        format!("  Added: {}", sub_label).bright_green()
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
        selected
    };

    if categories.is_empty() {
        println!("No valid selections, keeping current categories.");
        return;
    }

    config.categories = categories;
    println!(
        "Watching: {}",
        config.categories.join(", ").bright_green()
    );
}

fn edit_threshold(config: &mut Config) {
    println!();
    println!("{}", "ALERT THRESHOLD".bright_white().bold());
    println!("Minimum transaction value (USD) to trigger an alert.");
    println!("Lower values = more alerts, higher = only major moves.");
    println!("Common values: $10,000 | $25,000 | $50,000 | $100,000");
    println!();
    print!(
        "Threshold in USD (Enter for ${}): ",
        thousands(config.threshold)
    );
    let input = read_line();
    if input.is_empty() {
        return;
    }
    match input.replace('$', "").replace(',', "").parse::<u64>() {
        Ok(v) => config.threshold = v,
        Err(_) => println!("{}", "Invalid number, keeping current value.".yellow()),
    }
}

fn edit_interval(config: &mut Config) {
    println!();
    println!("{}", "POLL INTERVAL".bright_white().bold());
    println!("How often to poll for new transactions, in seconds.");
    println!("Lower = faster alerts, higher = less API traffic. Minimum: 1");
    println!();
    print!(
        "Interval in seconds (Enter for {}): ",
        config.interval_secs
    );
    let input = read_line();
    if input.is_empty() {
        return;
    }
    match input.parse::<u64>() {
        Ok(v) if v >= 1 => config.interval_secs = v,
        _ => println!("{}", "Enter a number of 1 or more, keeping current value.".yellow()),
    }
}

fn edit_max_odds(config: &mut Config) {
    println!();
    println!("{}", "MAX ODDS".bright_white().bold());
    println!("Skip alerts in markets where the YES or NO price exceeds this (0.0-1.0).");
    println!("Filters out near-certainties with no edge. Use 1 to disable.");
    println!();
    print!(
        "Max odds (Enter for {:.2}): ",
        config.max_odds
    );
    let input = read_line();
    if input.is_empty() {
        return;
    }
    match input.parse::<f64>() {
        Ok(v) if (0.0..=1.0).contains(&v) => config.max_odds = v,
        _ => println!("{}", "Enter a value between 0 and 1, keeping current value.".yellow()),
    }
}

fn edit_min_spread(config: &mut Config) {
    println!();
    println!("{}", "MIN SPREAD".bright_white().bold());
    println!("Skip markets with a bid/ask spread below this (0-1).");
    println!("Filters dead or settled markets. Use 0 to disable.");
    println!();
    print!(
        "Min spread (Enter for {:.2}): ",
        config.min_spread
    );
    let input = read_line();
    if input.is_empty() {
        return;
    }
    match input.parse::<f64>() {
        Ok(v) if (0.0..=1.0).contains(&v) => config.min_spread = v,
        _ => println!("{}", "Enter a value between 0 and 1, keeping current value.".yellow()),
    }
}

fn edit_retention(config: &mut Config) {
    println!();
    println!("{}", "HISTORY RETENTION".bright_white().bold());
    println!("How many days of alert history to keep in the database?");
    println!("Use 0 to keep alerts forever.");
    println!();
    print!(
        "Retention days (Enter for {}): ",
        config.history_retention_days
    );
    let input = read_line();
    if input.is_empty() {
        return;
    }
    match input.parse::<u32>() {
        Ok(v) => config.history_retention_days = v,
        Err(_) => println!("{}", "Invalid number, keeping current value.".yellow()),
    }
}

fn edit_kalshi(config: &mut Config) {
    println!();
    println!("{}", "KALSHI API (optional)".bright_white().bold());
    println!("Authentication is optional — public data works without it.");
    println!("Generate keys at: {}", "https://kalshi.com/profile/api-keys".bright_blue());
    println!("Enter 'clear' to remove stored keys.");
    println!();

    let current = if config.kalshi_api_key_id.is_some() {
        "configured"
    } else {
        "not set"
    };
    print!(
        "Kalshi API Key ID ({}, Enter to keep): ",
        current.bright_green()
    );
    let key_id = read_line();

    if key_id.is_empty() {
        return;
    }
    if key_id == "clear" || key_id == "none" {
        config.kalshi_api_key_id = None;
        config.kalshi_private_key = None;
        println!("{}", "Kalshi API keys cleared.".bright_green());
        return;
    }

    print!("Kalshi Private Key: ");
    let pk = read_line();
    config.kalshi_api_key_id = Some(key_id);
    config.kalshi_private_key = Some(pk);
    println!("{}", "Kalshi API configured".bright_green());
}

fn edit_webhook(config: &mut Config) {
    println!();
    println!("{}", "WEBHOOK / N8N (optional)".bright_white().bold());
    println!("Send alerts to a webhook (works with n8n, Zapier, Make, etc.)");
    println!("Enter 'clear' to remove the stored URL.");
    println!();

    let current = if config.webhook_url.is_some() {
        "configured"
    } else {
        "not set"
    };
    print!(
        "Webhook URL ({}, Enter to keep): ",
        current.bright_green()
    );
    let input = read_line();

    if input.is_empty() {
        return;
    }
    if input == "clear" || input == "none" {
        config.webhook_url = None;
        println!("Webhook cleared.");
        return;
    }
    config.webhook_url = Some(input);
    println!(
        "{}",
        format!("Webhook: {}", config.webhook_url.as_deref().unwrap_or("")).bright_green()
    );
}

pub async fn setup_config() -> Result<(), Box<dyn std::error::Error>> {
    let banner = "═══════════════════════════════════════════════════════════";
    println!("{}", banner.bright_cyan());
    println!("{}", "                        POLY SETUP                         ".bright_cyan().bold());
    println!("{}", banner.bright_cyan());
    println!();
    println!("Edit any setting below. Changes are saved immediately.");
    println!("Press Enter at any prompt to keep the current value.");
    println!();

    let first_run = !crate::config::config_exists();
    let mut config = crate::config::load_config().unwrap_or_default();

    if first_run {
        println!(
            "{}",
            "Welcome! No configuration found yet — pick the options that suit you.".bright_white()
        );
        println!();
    }

    loop {
        display_menu(&config);
        let choice = read_line();
        println!();

        match choice.as_str() {
            "1" => {
                edit_platforms(&mut config);
                crate::config::save_config(&config)?;
                saved_msg("platforms updated");
            }
            "2" => {
                edit_categories(&mut config);
                crate::config::save_config(&config)?;
                saved_msg("categories updated");
            }
            "3" => {
                edit_threshold(&mut config);
                crate::config::save_config(&config)?;
                saved_msg(&format!("threshold set to ${}", thousands(config.threshold)));
            }
            "4" => {
                edit_interval(&mut config);
                crate::config::save_config(&config)?;
                saved_msg(&format!("interval set to {} seconds", config.interval_secs));
            }
            "5" => {
                edit_max_odds(&mut config);
                crate::config::save_config(&config)?;
                saved_msg(&format!("max odds set to {:.2}", config.max_odds));
            }
            "6" => {
                edit_min_spread(&mut config);
                crate::config::save_config(&config)?;
                saved_msg(&format!("min spread set to {:.2}", config.min_spread));
            }
            "7" => {
                edit_retention(&mut config);
                crate::config::save_config(&config)?;
                let msg = if config.history_retention_days == 0 {
                    "history kept forever".to_string()
                } else {
                    format!("history kept for {} days", config.history_retention_days)
                };
                saved_msg(&msg);
            }
            "8" => {
                edit_kalshi(&mut config);
                crate::config::save_config(&config)?;
                saved_msg("Kalshi API settings updated");
            }
            "9" => {
                edit_webhook(&mut config);
                crate::config::save_config(&config)?;
                saved_msg("webhook settings updated");
            }
            "t" => {
                println!("{}", "Playing alert sound...".bright_cyan());
                crate::alerts::sound::play_alert_sound();
                println!("{}", "Sending test webhook...".bright_cyan());
                crate::commands::test::test_webhook().await?;
            }
            "d" => {
                print!("Reset ALL settings to defaults? [y/N]: ");
                io::stdout().flush().unwrap();
                let confirm = read_line();
                if confirm.eq_ignore_ascii_case("y") || confirm.eq_ignore_ascii_case("yes") {
                    config = Config::default();
                    crate::config::save_config(&config)?;
                    println!("{}", "✓ All settings reset to defaults".bright_green());
                    println!();
                }
            }
            "q" | "" => break,
            other => {
                println!(
                    "{}",
                    format!("Unknown option '{}'. Pick 1-9, t, d, or q.", other).yellow()
                );
                println!();
            }
        }
    }

    println!("{}", banner.bright_green());
    println!("{}", "                     SETTINGS SAVED                       ".bright_green().bold());
    println!("{}", banner.bright_green());
    println!();
    println!("Run {} to start watching for whale transactions.", "poly watch".bright_cyan());
    println!(
        "Or override the threshold once: {}",
        "poly watch --threshold 50000".bright_cyan()
    );

    Ok(())
}
