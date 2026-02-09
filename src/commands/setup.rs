use std::io::{self, Write};

use colored::*;

pub async fn setup_config() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "═══════════════════════════════════════════════════════════".bright_cyan());
    println!("{}", "                    WHALE WATCHER SETUP                     ".bright_cyan().bold());
    println!("{}", "═══════════════════════════════════════════════════════════".bright_cyan());
    println!();

    println!("This tool monitors large transactions on Polymarket and Kalshi.");
    println!("Most settings are optional - the tool works with public data.");
    println!();

    // ═══════════════════════════════════════════════════════════════════════
    // AI AGENT MODE
    // ═══════════════════════════════════════════════════════════════════════
    println!("{}", "┌─────────────────────────────────────────────────────────┐".bright_magenta());
    println!("{}", "│              AI AGENT INTEGRATION                       │".bright_magenta().bold());
    println!("{}", "└─────────────────────────────────────────────────────────┘".bright_magenta());
    println!();
    println!("Will you use wwatcher with an AI agent (OpenClaw, Claude Code)?");
    println!("If yes, RapidAPI and Perplexity keys are {} for research.", "required".bright_yellow());
    println!();

    print!("Enable AI Agent mode? (y/N): ");
    io::stdout().flush()?;
    let mut ai_mode_input = String::new();
    std::io::stdin().read_line(&mut ai_mode_input)?;
    let ai_agent_mode = ai_mode_input.trim().to_lowercase() == "y" || ai_mode_input.trim().to_lowercase() == "yes";

    if ai_agent_mode {
        println!("{}", "✓ AI Agent mode enabled".bright_green());
    } else {
        println!("AI Agent mode disabled (can enable later with 'wwatcher setup')");
    }
    println!();

    // ═══════════════════════════════════════════════════════════════════════
    // KALSHI (always optional)
    // ═══════════════════════════════════════════════════════════════════════
    println!("{}", "┌─────────────────────────────────────────────────────────┐".bright_yellow());
    println!("{}", "│              KALSHI API (optional)                      │".bright_yellow().bold());
    println!("{}", "└─────────────────────────────────────────────────────────┘".bright_yellow());
    println!();
    println!("Kalshi authentication is optional - public data works without it.");
    println!("Generate API keys at: {}", "https://kalshi.com/profile/api-keys".bright_blue());
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
        println!("{}", "✓ Kalshi API configured".bright_green());
        key.trim().to_string()
    } else {
        println!("Skipping Kalshi API configuration.");
        String::new()
    };
    println!();

    // ═══════════════════════════════════════════════════════════════════════
    // WEBHOOK (always optional)
    // ═══════════════════════════════════════════════════════════════════════
    println!("{}", "┌─────────────────────────────────────────────────────────┐".bright_yellow());
    println!("{}", "│              WEBHOOK / n8n (optional)                   │".bright_yellow().bold());
    println!("{}", "└─────────────────────────────────────────────────────────┘".bright_yellow());
    println!();
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
        println!("{}", format!("✓ Webhook configured: {}", webhook_url).bright_green());
    }
    println!();

    // ═══════════════════════════════════════════════════════════════════════
    // RAPIDAPI (optional, but required for AI agent mode)
    // ═══════════════════════════════════════════════════════════════════════
    let rapidapi_required = ai_agent_mode;
    let rapidapi_label = if rapidapi_required { "REQUIRED for AI mode" } else { "optional" };
    
    println!("{}", "┌─────────────────────────────────────────────────────────┐".bright_yellow());
    println!("{}", format!("│              RAPIDAPI KEY ({})              │", rapidapi_label).bright_yellow().bold());
    println!("{}", "└─────────────────────────────────────────────────────────┘".bright_yellow());
    println!();
    println!("RapidAPI provides market data (crypto prices, sports odds, weather).");
    println!("Get your key at: {}", "https://rapidapi.com".bright_blue());
    println!();
    println!("Subscribe to these free APIs:");
    println!("  • Coinranking (crypto): rapidapi.com/Coinranking/api/coinranking1");
    println!("  • NBA API (sports): rapidapi.com/api-sports/api/nba-api-free-data");
    println!("  • Meteostat (weather): rapidapi.com/meteostat/api/meteostat");
    println!();

    let rapidapi_key = loop {
        let prompt = if rapidapi_required {
            "Enter RapidAPI Key: "
        } else {
            "Enter RapidAPI Key (or press Enter to skip): "
        };
        print!("{}", prompt);
        io::stdout().flush()?;
        
        let mut key = String::new();
        std::io::stdin().read_line(&mut key)?;
        let key = key.trim().to_string();
        
        if key.is_empty() {
            if rapidapi_required {
                println!("{}", "⚠ RapidAPI key is required for AI Agent mode.".bright_red());
                continue;
            } else {
                println!("Skipping RapidAPI configuration.");
                break None;
            }
        } else {
            println!("{}", "✓ RapidAPI key configured".bright_green());
            break Some(key);
        }
    };
    println!();

    // ═══════════════════════════════════════════════════════════════════════
    // PERPLEXITY (optional, but required for AI agent mode)
    // ═══════════════════════════════════════════════════════════════════════
    let perplexity_required = ai_agent_mode;
    let perplexity_label = if perplexity_required { "REQUIRED for AI mode" } else { "optional" };
    
    println!("{}", "┌─────────────────────────────────────────────────────────┐".bright_yellow());
    println!("{}", format!("│           PERPLEXITY API KEY ({})         │", perplexity_label).bright_yellow().bold());
    println!("{}", "└─────────────────────────────────────────────────────────┘".bright_yellow());
    println!();
    println!("Perplexity provides deep web research for market analysis.");
    println!("Get your key at: {}", "https://perplexity.ai/settings/api".bright_blue());
    println!();

    let perplexity_key = loop {
        let prompt = if perplexity_required {
            "Enter Perplexity API Key: "
        } else {
            "Enter Perplexity API Key (or press Enter to skip): "
        };
        print!("{}", prompt);
        io::stdout().flush()?;
        
        let mut key = String::new();
        std::io::stdin().read_line(&mut key)?;
        let key = key.trim().to_string();
        
        if key.is_empty() {
            if perplexity_required {
                println!("{}", "⚠ Perplexity key is required for AI Agent mode.".bright_red());
                continue;
            } else {
                println!("Skipping Perplexity configuration.");
                break None;
            }
        } else {
            println!("{}", "✓ Perplexity API key configured".bright_green());
            break Some(key);
        }
    };
    println!();

    // ═══════════════════════════════════════════════════════════════════════
    // SAVE CONFIG
    // ═══════════════════════════════════════════════════════════════════════
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
        rapidapi_key: rapidapi_key.clone(),
        perplexity_api_key: perplexity_key.clone(),
        ai_agent_mode,
    };

    crate::config::save_config(&config)?;
    
    // Also write to integration/.env if AI mode enabled
    if ai_agent_mode {
        if let Err(e) = crate::config::write_integration_env(&rapidapi_key, &perplexity_key) {
            println!("{}", format!("Note: Could not write integration/.env: {}", e).bright_yellow());
            println!("You may need to manually set API keys in integration/.env");
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // SUMMARY
    // ═══════════════════════════════════════════════════════════════════════
    println!("{}", "═══════════════════════════════════════════════════════════".bright_green());
    println!("{}", "                 CONFIGURATION SAVED                       ".bright_green().bold());
    println!("{}", "═══════════════════════════════════════════════════════════".bright_green());
    println!();
    
    println!("{}", "Summary:".bright_white().bold());
    println!("  Kalshi API:    {}", if config.kalshi_api_key_id.is_some() { "✓ Configured".bright_green() } else { "○ Skipped".dimmed() });
    println!("  Webhook:       {}", if config.webhook_url.is_some() { "✓ Configured".bright_green() } else { "○ Skipped".dimmed() });
    println!("  RapidAPI:      {}", if config.rapidapi_key.is_some() { "✓ Configured".bright_green() } else { "○ Skipped".dimmed() });
    println!("  Perplexity:    {}", if config.perplexity_api_key.is_some() { "✓ Configured".bright_green() } else { "○ Skipped".dimmed() });
    println!("  AI Agent Mode: {}", if config.ai_agent_mode { "✓ Enabled".bright_green() } else { "○ Disabled".dimmed() });
    println!();

    println!(
        "Run {} to start watching for whale transactions.",
        "wwatcher watch".bright_cyan()
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
