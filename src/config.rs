use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
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
    /// Polling interval in seconds
    #[serde(default = "default_interval")]
    pub interval_secs: u64,
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

impl Default for Config {
    fn default() -> Self {
        Self {
            kalshi_api_key_id: None,
            kalshi_private_key: None,
            webhook_url: None,
            categories: default_categories(),
            threshold: default_threshold(),
            platforms: default_platforms(),
            interval_secs: default_interval(),
            history_retention_days: default_retention_days(),
            max_odds: default_max_odds(),
            min_spread: default_min_spread(),
        }
    }
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

fn default_interval() -> u64 {
    5
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
    let app_config_dir = app_dir()?;
    Ok(app_config_dir.join("config.json"))
}

/// Returns the app data directory (`~/.config/poly` on Linux).
/// Migrates an existing `wwatcher` directory (and its database file) on first run.
pub fn app_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let config_dir = dirs::config_dir().ok_or("Could not determine config directory")?;

    let new_dir = config_dir.join("poly");
    let old_dir = config_dir.join("wwatcher");

    if !new_dir.exists() && old_dir.exists() {
        std::fs::rename(&old_dir, &new_dir)?;
        let old_db = new_dir.join("wwatcher.db");
        let new_db = new_dir.join("poly.db");
        if old_db.exists() && !new_db.exists() {
            std::fs::rename(&old_db, &new_db)?;
        }
    }

    fs::create_dir_all(&new_dir)?;
    Ok(new_dir)
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

pub fn config_exists() -> bool {
    config_path().map(|p| p.exists()).unwrap_or(false)
}

