use rusqlite::{Connection, params};
use sha2::{Sha256, Digest};
use std::path::PathBuf;

pub fn wallet_hash(wallet_id: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(wallet_id.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn db_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let app_dir = crate::config::app_dir()?;
    Ok(app_dir.join("poly.db"))
}

pub fn open_db() -> Result<Connection, Box<dyn std::error::Error>> {
    let path = db_path()?;
    let conn = Connection::open(&path)?;

    // Performance pragmas
    conn.execute_batch(
        "PRAGMA journal_mode=WAL;
         PRAGMA synchronous=NORMAL;
         PRAGMA busy_timeout=5000;"
    )?;

    init_schema(&conn)?;
    Ok(conn)
}

fn init_schema(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS alerts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            platform TEXT NOT NULL,
            alert_type TEXT NOT NULL,
            action TEXT NOT NULL,
            category TEXT,
            subcategory TEXT,
            value REAL NOT NULL,
            price REAL NOT NULL,
            size REAL NOT NULL,
            market_title TEXT,
            market_id TEXT,
            outcome TEXT,
            wallet_hash TEXT,
            wallet_id TEXT,
            timestamp TEXT NOT NULL,
            market_context TEXT,
            wallet_activity TEXT,
            created_at INTEGER DEFAULT (strftime('%s', 'now'))
        );

        CREATE INDEX IF NOT EXISTS idx_alerts_wallet_hash ON alerts(wallet_hash);
        CREATE INDEX IF NOT EXISTS idx_alerts_timestamp ON alerts(created_at);
        CREATE INDEX IF NOT EXISTS idx_alerts_category ON alerts(category);
        CREATE INDEX IF NOT EXISTS idx_alerts_platform ON alerts(platform);

        CREATE TABLE IF NOT EXISTS wallet_memory (
            wallet_hash TEXT NOT NULL,
            wallet_id TEXT NOT NULL,
            market_title TEXT,
            market_id TEXT,
            outcome TEXT,
            action TEXT,
            value REAL NOT NULL,
            price REAL NOT NULL,
            platform TEXT NOT NULL,
            category TEXT,
            seen_at INTEGER NOT NULL,
            PRIMARY KEY (wallet_hash, market_id, seen_at)
        );

        CREATE INDEX IF NOT EXISTS idx_wallet_memory_hash ON wallet_memory(wallet_hash);
        CREATE INDEX IF NOT EXISTS idx_wallet_memory_seen ON wallet_memory(seen_at);

        CREATE TABLE IF NOT EXISTS metadata (
            key TEXT PRIMARY KEY,
            value TEXT
        );

        INSERT OR IGNORE INTO metadata (key, value) VALUES ('schema_version', '1');
        INSERT OR IGNORE INTO metadata (key, value) VALUES ('created_at', strftime('%s', 'now'));"
    )?;

    Ok(())
}

/// Insert an alert into the alerts table
pub fn insert_alert(
    conn: &Connection,
    platform: &str,
    alert_type: &str,
    action: &str,
    value: f64,
    price: f64,
    size: f64,
    market_title: Option<&str>,
    market_id: Option<&str>,
    outcome: Option<&str>,
    wallet_id: Option<&str>,
    timestamp: &str,
    market_context_json: Option<&str>,
    wallet_activity_json: Option<&str>,
) {
    let w_hash = wallet_id.map(wallet_hash);

    let result = conn.execute(
        "INSERT INTO alerts (platform, alert_type, action, value, price, size,
         market_title, market_id, outcome, wallet_hash, wallet_id, timestamp,
         market_context, wallet_activity)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
        params![
            platform,
            alert_type,
            action,
            value,
            price,
            size,
            market_title,
            market_id,
            outcome,
            w_hash,
            wallet_id,
            timestamp,
            market_context_json,
            wallet_activity_json,
        ],
    );

    if let Err(e) = result {
        eprintln!("Warning: Failed to log alert to database: {}", e);
    }
}

/// Query recent alerts for display
pub fn query_alerts(
    conn: &Connection,
    limit: usize,
    platform_filter: &str,
) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
    let mut alerts = Vec::new();

    let (sql, filter_params): (String, Vec<Box<dyn rusqlite::types::ToSql>>) = if platform_filter == "all" {
        (
            "SELECT platform, alert_type, action, value, price, size,
                    market_title, outcome, wallet_id, timestamp,
                    wallet_activity, market_context
             FROM alerts ORDER BY created_at DESC LIMIT ?1".to_string(),
            vec![Box::new(limit as i64)],
        )
    } else {
        (
            "SELECT platform, alert_type, action, value, price, size,
                    market_title, outcome, wallet_id, timestamp,
                    wallet_activity, market_context
             FROM alerts WHERE LOWER(platform) = LOWER(?1)
             ORDER BY created_at DESC LIMIT ?2".to_string(),
            vec![
                Box::new(platform_filter.to_string()),
                Box::new(limit as i64),
            ],
        )
    };

    let params_refs: Vec<&dyn rusqlite::types::ToSql> = filter_params.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params_refs.as_slice(), |row| {
        let platform: String = row.get(0)?;
        let alert_type: String = row.get(1)?;
        let action: String = row.get(2)?;
        let value: f64 = row.get(3)?;
        let price: f64 = row.get(4)?;
        let size: f64 = row.get(5)?;
        let market_title: Option<String> = row.get(6)?;
        let outcome: Option<String> = row.get(7)?;
        let wallet_id: Option<String> = row.get(8)?;
        let timestamp: String = row.get(9)?;
        let wallet_activity_json: Option<String> = row.get(10)?;
        let market_context_json: Option<String> = row.get(11)?;

        let mut alert = serde_json::json!({
            "platform": platform,
            "alert_type": alert_type,
            "action": action,
            "value": value,
            "price": price,
            "size": size,
            "timestamp": timestamp,
            "market_title": market_title,
            "outcome": outcome,
        });

        if let Some(wid) = wallet_id {
            alert["wallet_id"] = serde_json::json!(wid);
        }

        if let Some(wa_json) = wallet_activity_json {
            if let Ok(wa) = serde_json::from_str::<serde_json::Value>(&wa_json) {
                alert["wallet_activity"] = wa;
            }
        }

        if let Some(mc_json) = market_context_json {
            if let Ok(mc) = serde_json::from_str::<serde_json::Value>(&mc_json) {
                alert["market_context"] = mc;
            }
        }

        Ok(alert)
    })?;

    for row in rows {
        if let Ok(alert) = row {
            alerts.push(alert);
        }
    }

    Ok(alerts)
}

/// Prune old alerts based on retention days (0 = keep forever)
pub fn prune_old_alerts(conn: &Connection, retention_days: u32) {
    if retention_days == 0 {
        return;
    }
    let seconds = retention_days as i64 * 86400;
    let result = conn.execute(
        "DELETE FROM alerts WHERE created_at < (strftime('%s', 'now') - ?1)",
        params![seconds],
    );
    if let Err(e) = result {
        eprintln!("Warning: Failed to prune old alerts: {}", e);
    }
}

/// Prune expired wallet memory (12h window)
pub fn prune_wallet_memory(conn: &Connection) {
    let result = conn.execute(
        "DELETE FROM wallet_memory WHERE seen_at < (strftime('%s', 'now') - 43200)",
        [],
    );
    if let Err(e) = result {
        eprintln!("Warning: Failed to prune wallet memory: {}", e);
    }
}

/// Migrate existing JSONL history to SQLite
pub fn migrate_jsonl_if_exists(conn: &Connection) {
    let jsonl_path = match crate::config::app_dir() {
        Ok(d) => d.join("alert_history.jsonl"),
        Err(_) => return,
    };
    if !jsonl_path.exists() {
        return;
    }

    let contents = match std::fs::read_to_string(&jsonl_path) {
        Ok(c) => c,
        Err(_) => return,
    };

    let mut count = 0u32;
    for line in contents.lines() {
        if let Ok(alert) = serde_json::from_str::<serde_json::Value>(line) {
            let platform = alert.get("platform").and_then(|v| v.as_str()).unwrap_or("Unknown");
            let alert_type = alert.get("alert_type").and_then(|v| v.as_str()).unwrap_or("UNKNOWN");
            let action = alert.get("action").and_then(|v| v.as_str()).unwrap_or("UNKNOWN");
            let value = alert.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let price = alert.get("price").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let size = alert.get("size").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let market_title = alert.get("market_title").and_then(|v| v.as_str());
            let outcome = alert.get("outcome").and_then(|v| v.as_str());
            let wallet_id = alert.get("wallet_id").and_then(|v| v.as_str());
            let timestamp = alert.get("timestamp").and_then(|v| v.as_str()).unwrap_or("");

            let wa_json = alert.get("wallet_activity").map(|v| v.to_string());

            insert_alert(
                conn,
                platform,
                alert_type,
                action,
                value,
                price,
                size,
                market_title,
                None,
                outcome,
                wallet_id,
                timestamp,
                None,
                wa_json.as_deref(),
            );
            count += 1;
        }
    }

    if count > 0 {
        let bak_path = match crate::config::app_dir() {
            Ok(d) => d.join("alert_history.jsonl.bak"),
            Err(_) => return,
        };
        if std::fs::rename(&jsonl_path, &bak_path).is_ok() {
            eprintln!("Migrated {} alerts from JSONL to SQLite database.", count);
            eprintln!("Old file backed up to: alert_history.jsonl.bak");
        }
    }
}

/// Get alert count in database
pub fn alert_count(conn: &Connection) -> i64 {
    conn.query_row("SELECT COUNT(*) FROM alerts", [], |row| row.get(0))
        .unwrap_or(0)
}
