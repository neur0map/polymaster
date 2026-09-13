use colored::*;

use super::AlertData;

/// Sanitize text for messaging platforms that use Markdown/HTML parsing
pub fn escape_special_chars(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | ' ' | ',' | ':' | '?' | '.' => c,
            '(' | '[' | '{' => '(',
            ')' | ']' | '}' => ')',
            _ => ' ',
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
}

pub async fn send_webhook_alert(webhook_url: &str, alert: &AlertData<'_>) {
    let payload = super::build_alert_payload(alert, true);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap();

    match client.post(webhook_url).json(&payload).send().await {
        Ok(response) => {
            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                eprintln!(
                    "{} Webhook failed with status: {} | response: {}",
                    "[WEBHOOK ERROR]".red(),
                    status,
                    if body.is_empty() { "<no body>".to_string() } else { body[..body.len().min(500)].to_string() }
                );
            }
        }
        Err(e) => {
            eprintln!("{} Failed to send webhook: {}", "[WEBHOOK ERROR]".red(), e);
        }
    }
}
