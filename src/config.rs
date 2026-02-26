use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Config {
    pub kalshi_api_key_id: Option<String>,
    pub kalshi_private_key: Option<String>,
    pub webhook_url: Option<String>,
    /// Selected market categories (e.g. ["sports:nba", "crypto:all", "politics:us_elections"])
    /// Empty or ["all"] means watch everything
    #[serde(default = "default_categories")]
    pub categories: Vec<String>,
    /// Default whale alert threshold in USD
    #[serde(default = "default_threshold")]
    pub threshold: u64,
    /// Which platforms to monitor: ["polymarket", "kalshi"] or ["all"]
    #[serde(default = "default_platforms")]
    pub platforms: Vec<String>,
    /// Days to retain alerts in the database (0 = keep forever)
    #[serde(default = "default_retention_days")]
    pub history_retention_days: u32,
    /// Maximum odds to alert on (0.0-1.0). Skip if YES or NO price exceeds this.
    /// Default 0.95 filters out near-certainties with no edge.
    #[serde(default = "default_max_odds")]
    pub max_odds: f64,
    /// Minimum spread to alert on. Skip dead/settled markets with 0 spread.
    /// Default 0.0 (disabled).
    #[serde(default = "default_min_spread")]
    pub min_spread: f64,
}

fn default_categories() -> Vec<String> {
    vec!["all".into()]
}

fn default_threshold() -> u64 {
    25000
}

fn default_platforms() -> Vec<String> {
    vec!["all".into()]
}

fn default_retention_days() -> u32 {
    30
}

fn default_max_odds() -> f64 {
    0.95
}

fn default_min_spread() -> f64 {
    0.0
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

