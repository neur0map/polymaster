use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Config {
    pub kalshi_api_key_id: Option<String>,
    pub kalshi_private_key: Option<String>,
    pub webhook_url: Option<String>,
    pub rapidapi_key: Option<String>,
    pub perplexity_api_key: Option<String>,
    pub ai_agent_mode: bool,
}

fn config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let config_dir = dirs::config_dir().ok_or("Could not determine config directory")?;

    let app_config_dir = config_dir.join("wwatcher");
    fs::create_dir_all(&app_config_dir)?;

    Ok(app_config_dir.join("config.json"))
}

pub fn save_config(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let path = config_path()?;
    let json = serde_json::to_string_pretty(config)?;
    fs::write(path, json)?;
    Ok(())
}

pub fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let path = config_path()?;

    if !path.exists() {
        return Ok(Config::default());
    }

    let json = fs::read_to_string(path)?;
    let config: Config = serde_json::from_str(&json)?;
    Ok(config)
}

/// Get the integration .env path for AI agent keys
pub fn integration_env_path() -> Option<PathBuf> {
    // Try to find integration/.env relative to common locations
    let possible_paths = vec![
        PathBuf::from("integration/.env"),
        dirs::home_dir()?.join("polymaster/integration/.env"),
        dirs::home_dir()?.join("polymaster-test/integration/.env"),
    ];
    
    for path in possible_paths {
        if path.exists() {
            return Some(path);
        }
    }
    None
}

/// Write API keys to integration/.env for the AI agent CLI
pub fn write_integration_env(rapidapi_key: &Option<String>, perplexity_key: &Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let env_path = integration_env_path();
    
    if let Some(path) = env_path {
        let mut content = String::new();
        content.push_str("# wwatcher AI integration environment\n");
        content.push_str("WWATCHER_HISTORY_PATH=~/.config/wwatcher/alert_history.jsonl\n\n");
        
        if let Some(key) = rapidapi_key {
            content.push_str(&format!("RAPIDAPI_KEY={}\n", key));
        } else {
            content.push_str("# RAPIDAPI_KEY=\n");
        }
        
        if let Some(key) = perplexity_key {
            content.push_str(&format!("PERPLEXITY_API_KEY={}\n", key));
        } else {
            content.push_str("# PERPLEXITY_API_KEY=\n");
        }
        
        fs::write(path, content)?;
    }
    
    Ok(())
}
