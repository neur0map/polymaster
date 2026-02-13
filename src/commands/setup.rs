use std::io::{self, Write};

use colored::*;

use crate::categories::CategoryRegistry;

fn read_line() -> String {
    let mut input = String::new();
    io::stdout().flush().unwrap();
    std::io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn print_step(step: u8, total: u8, title: &str) {
    println!();
    println!(
        "{}",
        format!(
            "┌─── Step {}/{} ─────────────────────────────────────────────┐",
            step, total
        )
        .bright_cyan()
    );
    println!(
        "{}",
        format!("│  {:<53}│", title).bright_cyan().bold()
    );
    println!(
        "{}",
        "└─────────────────────────────────────────────────────────┘".bright_cyan()
    );
    println!();
}

pub async fn setup_config() -> Result<(), Box<dyn std::error::Error>> {
    let total_steps = 6;

    println!(
        "{}",
        "═══════════════════════════════════════════════════════════"
            .bright_cyan()
    );
    println!(
        "{}",
        "                    WHALE WATCHER SETUP                     "
            .bright_cyan()
            .bold()
    );
    println!(
        "{}",
        "═══════════════════════════════════════════════════════════"
            .bright_cyan()
    );
    println!();
    println!("This wizard will guide you through configuring wwatcher.");
    println!("Most settings are optional — the tool works with public data.");
    println!("Press Enter at any prompt to use the default / skip.");
    println!();

    // Load existing config for defaults
    let existing = crate::config::load_config().unwrap_or_default();

    // ═══════════════════════════════════════════════════════════════════════
    // STEP 1: PLATFORMS
    // ═══════════════════════════════════════════════════════════════════════
    print_step(1, total_steps, "PLATFORMS");
    println!("Which prediction markets do you want to monitor?");
    println!();
    println!("  [1] Both Polymarket + Kalshi (recommended)");
    println!("  [2] Polymarket only");
    println!("  [3] Kalshi only");
    println!();

    let current_platforms = if existing.platforms.iter().any(|p| p == "all") {
        "1"
    } else if existing.platforms.contains(&"polymarket".into())
        && existing.platforms.contains(&"kalshi".into())
    {
        "1"
    } else if existing.platforms.contains(&"polymarket".into()) {
        "2"
    } else {
        "3"
    };
    print!(
        "Select [1-3] (current: {}): ",
        current_platforms.bright_green()
    );
    let platform_choice = read_line();
    let platforms: Vec<String> = match platform_choice.as_str() {
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

    // ═══════════════════════════════════════════════════════════════════════
    // STEP 2: MARKET CATEGORIES
    // ═══════════════════════════════════════════════════════════════════════
    print_step(2, total_steps, "MARKET CATEGORIES");
    println!("Choose which market categories to watch for whale alerts.");
    println!(
        "You can watch {} or pick specific categories.",
        "everything".bright_green()
    );
    println!();

    let all_cats = CategoryRegistry::all_categories();
    println!("  [0] {} — Watch all markets (recommended)", "ALL".bright_green().bold());
    for (i, (_, label)) in all_cats.iter().enumerate() {
        println!("  [{}] {}", i + 1, label);
    }
    println!();

    let current_cat_display = if existing.categories.iter().any(|c| c == "all") {
        "all".to_string()
    } else {
        existing.categories.join(", ")
    };
    println!(
        "Current: {}",
        current_cat_display.bright_green()
    );
    println!("Enter numbers separated by commas (e.g. 1,3,5) or 0 for all.");
    print!("Select: ");
    let cat_input = read_line();

    let categories: Vec<String> = if cat_input.is_empty() || cat_input == "0" {
        println!("{}", "Watching all markets".bright_green());
        vec!["all".into()]
    } else {
        let mut selected: Vec<String> = Vec::new();
        let indices: Vec<&str> = cat_input.split(',').map(|s| s.trim()).collect();

        for idx_str in &indices {
            if let Ok(idx) = idx_str.parse::<usize>() {
                if idx >= 1 && idx <= all_cats.len() {
                    let (cat_key, cat_label) = all_cats[idx - 1];
                    // Ask for subcategory selection
                    println!();
                    println!(
                        "{}",
                        format!("Subcategories for {}:", cat_label)
                            .bright_white()
                            .bold()
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
                        println!(
                            "{}",
                            format!("  Added: all {}", cat_key).bright_green()
                        );
                    } else {
                        let sub_indices: Vec<&str> =
                            sub_input.split(',').map(|s| s.trim()).collect();
                        for sidx_str in &sub_indices {
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

        if selected.is_empty() {
            println!("No valid selections, defaulting to all markets.");
            vec!["all".into()]
        } else {
            println!();
            println!(
                "Watching: {}",
                selected.join(", ").bright_green()
            );
            selected
        }
    };

    // ═══════════════════════════════════════════════════════════════════════
    // STEP 3: THRESHOLD + RETENTION
    // ═══════════════════════════════════════════════════════════════════════
    print_step(3, total_steps, "ALERT THRESHOLD");
    println!("Minimum transaction value (USD) to trigger a whale alert.");
    println!("Lower values = more alerts, higher = only major moves.");
    println!();
    println!("  Common values: $10,000 | $25,000 | $50,000 | $100,000");
    println!();
    print!(
        "Threshold in USD (current: ${}): ",
        existing.threshold.to_string().bright_green()
    );
    let threshold_input = read_line();
    let threshold: u64 = if threshold_input.is_empty() {
        existing.threshold
    } else {
        threshold_input
            .replace('$', "")
            .replace(',', "")
            .parse()
            .unwrap_or(existing.threshold)
    };
    println!(
        "{}",
        format!("Threshold set to ${}", threshold).bright_green()
    );

    println!();
    println!("How many days of alert history to keep? (0 = forever)");
    print!(
        "Retention days (current: {}): ",
        existing.history_retention_days.to_string().bright_green()
    );
    let retention_input = read_line();
    let history_retention_days: u32 = if retention_input.is_empty() {
        existing.history_retention_days
    } else {
        retention_input.parse().unwrap_or(existing.history_retention_days)
    };
    if history_retention_days == 0 {
        println!("{}", "Keeping alerts forever".bright_green());
    } else {
        println!(
            "{}",
            format!("Keeping {} days of history", history_retention_days).bright_green()
        );
    }

    // ═══════════════════════════════════════════════════════════════════════
    // STEP 4: API KEYS
    // ═══════════════════════════════════════════════════════════════════════
    print_step(4, total_steps, "API KEYS");

    // Kalshi
    println!("{}", "Kalshi API (optional)".bright_white().bold());
    println!("Authentication is optional — public data works without it.");
    println!(
        "Generate keys at: {}",
        "https://kalshi.com/profile/api-keys".bright_blue()
    );
    println!();

    let kalshi_current = if existing.kalshi_api_key_id.is_some() {
        "configured"
    } else {
        "not set"
    };
    print!(
        "Kalshi API Key ID ({}, Enter to keep): ",
        kalshi_current.bright_green()
    );
    let kalshi_key_id = read_line();

    let (kalshi_api_key_id, kalshi_private_key) = if !kalshi_key_id.is_empty() {
        print!("Kalshi Private Key: ");
        let pk = read_line();
        println!("{}", "Kalshi API configured".bright_green());
        (Some(kalshi_key_id), Some(pk))
    } else {
        // Keep existing
        (existing.kalshi_api_key_id.clone(), existing.kalshi_private_key.clone())
    };
    println!();

    // Webhook
    println!("{}", "Webhook / n8n (optional)".bright_white().bold());
    println!("Send alerts to a webhook (works with n8n, Zapier, Make, etc.)");
    println!();

    print!(
        "Webhook URL ({}, Enter to keep): ",
        if existing.webhook_url.is_some() {
            "configured".bright_green()
        } else {
            "not set".bright_green()
        }
    );
    let webhook_input = read_line();
    let webhook_url = if webhook_input.is_empty() {
        existing.webhook_url.clone()
    } else if webhook_input == "clear" || webhook_input == "none" {
        println!("Webhook cleared.");
        None
    } else {
        println!(
            "{}",
            format!("Webhook: {}", webhook_input).bright_green()
        );
        Some(webhook_input)
    };

    // ═══════════════════════════════════════════════════════════════════════
    // STEP 5: AI AGENT MODE
    // ═══════════════════════════════════════════════════════════════════════
    print_step(5, total_steps, "AI AGENT MODE (optional)");
    println!("Use wwatcher with an AI agent (OpenClaw, Claude Code)?");
    println!(
        "If enabled, RapidAPI and Perplexity keys are {} (enhance research).",
        "optional".bright_yellow()
    );
    println!();

    let ai_current = if existing.ai_agent_mode { "enabled" } else { "disabled" };
    print!(
        "Enable AI Agent mode? ({}, y/N): ",
        ai_current.bright_green()
    );
    let ai_input = read_line();
    let ai_agent_mode = if ai_input.is_empty() {
        existing.ai_agent_mode
    } else {
        ai_input.to_lowercase() == "y" || ai_input.to_lowercase() == "yes"
    };

    if ai_agent_mode {
        println!("{}", "AI Agent mode enabled".bright_green());
    } else {
        println!("AI Agent mode disabled");
    }

    // RapidAPI (always optional now)
    let rapidapi_key = if ai_agent_mode {
        println!();
        println!("{}", "RapidAPI Key (optional)".bright_white().bold());
        println!("Provides market data for AI research (crypto, sports, weather).");
        println!(
            "Get key at: {}",
            "https://rapidapi.com".bright_blue()
        );

        let rapid_current = if existing.rapidapi_key.is_some() {
            "configured"
        } else {
            "not set"
        };
        print!(
            "RapidAPI Key ({}, Enter to keep): ",
            rapid_current.bright_green()
        );
        let key = read_line();
        if key.is_empty() {
            existing.rapidapi_key.clone()
        } else {
            println!("{}", "RapidAPI key set".bright_green());
            Some(key)
        }
    } else {
        existing.rapidapi_key.clone()
    };

    // Perplexity (always optional now)
    let perplexity_api_key = if ai_agent_mode {
        println!();
        println!("{}", "Perplexity API Key (optional)".bright_white().bold());
        println!("Provides deep web research for market analysis.");
        println!(
            "Get key at: {}",
            "https://perplexity.ai/settings/api".bright_blue()
        );

        let perp_current = if existing.perplexity_api_key.is_some() {
            "configured"
        } else {
            "not set"
        };
        print!(
            "Perplexity Key ({}, Enter to keep): ",
            perp_current.bright_green()
        );
        let key = read_line();
        if key.is_empty() {
            existing.perplexity_api_key.clone()
        } else {
            println!("{}", "Perplexity key set".bright_green());
            Some(key)
        }
    } else {
        existing.perplexity_api_key.clone()
    };

    // ═══════════════════════════════════════════════════════════════════════
    // STEP 6: SAVE + SUMMARY
    // ═══════════════════════════════════════════════════════════════════════
    print_step(6, total_steps, "SAVE CONFIGURATION");

    let config = crate::config::Config {
        kalshi_api_key_id,
        kalshi_private_key,
        webhook_url,
        rapidapi_key: rapidapi_key.clone(),
        perplexity_api_key: perplexity_api_key.clone(),
        ai_agent_mode,
        categories: categories.clone(),
        threshold,
        platforms: platforms.clone(),
        history_retention_days,
        max_odds: existing.max_odds,
        min_spread: existing.min_spread,
    };

    crate::config::save_config(&config)?;

    // Also write to integration/.env if AI mode enabled
    if ai_agent_mode {
        if let Err(e) = crate::config::write_integration_env(&rapidapi_key, &perplexity_api_key) {
            println!(
                "{}",
                format!("Note: Could not write integration/.env: {}", e).bright_yellow()
            );
        }
    }

    println!(
        "{}",
        "═══════════════════════════════════════════════════════════"
            .bright_green()
    );
    println!(
        "{}",
        "                 CONFIGURATION SAVED                       "
            .bright_green()
            .bold()
    );
    println!(
        "{}",
        "═══════════════════════════════════════════════════════════"
            .bright_green()
    );
    println!();

    println!("{}", "Summary:".bright_white().bold());

    // Platforms
    let plat_display = if platforms.iter().any(|p| p == "all") {
        "Polymarket + Kalshi"
    } else {
        &platforms.join(", ")
    };
    println!("  Platforms:     {}", plat_display.bright_green());

    // Categories
    let cat_display = if categories.iter().any(|c| c == "all") {
        "All markets".to_string()
    } else {
        categories.join(", ")
    };
    println!("  Categories:    {}", cat_display.bright_green());

    // Threshold
    println!(
        "  Threshold:     {}",
        format!("${}", threshold).bright_green()
    );

    // Retention
    let ret_display = if history_retention_days == 0 {
        "Forever".to_string()
    } else {
        format!("{} days", history_retention_days)
    };
    println!("  Retention:     {}", ret_display.bright_green());

    // API keys
    println!(
        "  Kalshi API:    {}",
        if config.kalshi_api_key_id.is_some() {
            "Configured".bright_green()
        } else {
            "Skipped".dimmed()
        }
    );
    println!(
        "  Webhook:       {}",
        if config.webhook_url.is_some() {
            "Configured".bright_green()
        } else {
            "Skipped".dimmed()
        }
    );
    println!(
        "  AI Agent Mode: {}",
        if config.ai_agent_mode {
            "Enabled".bright_green()
        } else {
            "Disabled".dimmed()
        }
    );
    if ai_agent_mode {
        println!(
            "  RapidAPI:      {}",
            if config.rapidapi_key.is_some() {
                "Configured".bright_green()
            } else {
                "Skipped".dimmed()
            }
        );
        println!(
            "  Perplexity:    {}",
            if config.perplexity_api_key.is_some() {
                "Configured".bright_green()
            } else {
                "Skipped".dimmed()
            }
        );
    }
    println!();

    println!(
        "Run {} to start watching for whale transactions.",
        "wwatcher watch".bright_cyan()
    );
    println!(
        "Or override threshold: {}",
        "wwatcher watch --threshold 50000".bright_cyan()
    );

    if ai_agent_mode {
        println!();
        println!("{}", "AI Agent Setup:".bright_white().bold());
        println!("  1. Build the CLI: cd integration && npm install && npm run build");
        println!("  2. Test: node dist/cli.js status");
        println!("  3. Research: node dist/cli.js research \"Bitcoin above 100k\"");
    }

    Ok(())
}
