use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Config {
    pub kalshi_api_key_id: Option<String>,
    pub kalshi_private_key: Option<String>,
    pub webhook_url: Option<String>,
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
