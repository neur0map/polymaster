use std::time::Duration;

use colored::*;
use rusqlite::Connection;
use tokio::time;

use crate::alerts::AlertData;
use crate::alerts::display::{self, format_number, print_kalshi_alert, print_market_context, print_order_book, print_top_holders, print_whale_alert, print_whale_profile};
use crate::alerts::history;
use crate::alerts::webhook;
use crate::categories::CategoryRegistry;
use crate::db;
use crate::platforms::kalshi;
use crate::platforms::polymarket;
use crate::types;
use crate::whale_profile;

pub async fn watch_whales(threshold: u64, interval: u64, conn: Connection) -> Result<(), Box<dyn std::error::Error>> {
    // Display disclaimer
    println!("{}", "=".repeat(70).bright_yellow());
    println!("{}", "DISCLAIMER".bright_yellow().bold());
    println!("This tool is for informational and research purposes only.");
    println!("I do not condone gambling or speculative trading.");
    println!("Use this data solely for informed decision-making and market analysis.");
    println!("Trade responsibly and within your means.");
    println!("{}", "=".repeat(70).bright_yellow());
    println!();

    println!("{}", "WHALE WATCHER ACTIVE".bright_cyan().bold());
    println!(
        "Threshold: {}",
        format!("${}", format_number(threshold)).bright_green()
    );
    println!("Interval:  {} seconds", interval);

    let config = crate::config::load_config().ok();

    // Initialize category filtering
    let category_registry = CategoryRegistry::new();
    let selected_categories: Vec<String> = config
        .as_ref()
        .map(|c| c.categories.clone())
        .unwrap_or_else(|| vec!["all".into()]);

    if selected_categories.iter().any(|s| s == "all") {
        println!("Categories: {}", "All markets".bright_green());
    } else {
        println!(
            "Categories: {}",
            selected_categories.join(", ").bright_green()
        );
    }

    // Platform filtering
    let selected_platforms: Vec<String> = config
        .as_ref()
        .map(|c| c.platforms.clone())
        .unwrap_or_else(|| vec!["all".into()]);
    let watch_polymarket = selected_platforms.iter().any(|p| p == "all" || p == "polymarket");
    let watch_kalshi = selected_platforms.iter().any(|p| p == "all" || p == "kalshi");

    if watch_polymarket && watch_kalshi {
        println!("Platforms:  {}", "Polymarket + Kalshi".bright_green());
    } else if watch_polymarket {
        println!("Platforms:  {}", "Polymarket only".bright_green());
    } else {
        println!("Platforms:  {}", "Kalshi only".bright_green());
    }

    if let Some(ref cfg) = config {
        if cfg.webhook_url.is_some() {
            println!("Webhook:   {}", "Enabled".bright_green());
        }
    }

    // Show DB info
    let alert_count = db::alert_count(&conn);
    println!("Database:  {} alerts stored", alert_count.to_string().bright_white());
    println!();

    let mut last_polymarket_trade_id: Option<String> = None;
    let mut last_kalshi_trade_id: Option<String> = None;

    let mut wallet_tracker = types::WalletTracker::new();
    let mut whale_cache = whale_profile::WhaleProfileCache::new();

    // Start Kalshi WebSocket if watching Kalshi
    let mut kalshi_ws_rx = if watch_kalshi {
        println!("Kalshi WS:  {}", "Connecting...".bright_cyan());
        Some(crate::ws::kalshi::spawn_kalshi_ws())
    } else {
        None
    };
    // Track whether WS is producing trades (for fallback)
    let mut kalshi_ws_last_trade = std::time::Instant::now();
    let kalshi_ws_fallback_threshold = Duration::from_secs(interval * 12); // fall back to HTTP if no WS trades in ~1 min

    let mut tick_interval = time::interval(Duration::from_secs(interval));

    // Prune counter - prune every 60 cycles (~5 min at 5s interval)
    let mut prune_counter: u32 = 0;

    loop {
        tick_interval.tick().await;

        // Periodic cleanup and cache refresh
        prune_counter += 1;
        if prune_counter >= 60 {
            prune_counter = 0;
            db::prune_wallet_memory(&conn);
            let retention = config.as_ref().map(|c| c.history_retention_days).unwrap_or(30);
            db::prune_old_alerts(&conn, retention);
            whale_cache.prune();
        }
        wallet_tracker.maybe_refresh_cache(&conn);

        // Drain Kalshi WebSocket trades (non-blocking)
        if let Some(ref mut rx) = kalshi_ws_rx {
            while let Ok(ws_trade) = rx.try_recv() {
                kalshi_ws_last_trade = std::time::Instant::now();

                let trade_value = (ws_trade.yes_price / 100.0) * f64::from(ws_trade.count);
                if trade_value < threshold as f64 {
                    continue;
                }

                // Build a kalshi::Trade from the WS trade for display compatibility
                let mut trade = kalshi::Trade {
                    trade_id: ws_trade.trade_id.clone(),
                    ticker: ws_trade.ticker.clone(),
                    price: ws_trade.yes_price / 100.0,
                    count: ws_trade.count,
                    yes_price: ws_trade.yes_price,
                    no_price: ws_trade.no_price,
                    taker_side: ws_trade.taker_side.clone(),
                    created_time: ws_trade.created_time.clone(),
                    market_title: None,
                };

                // Fetch full market info (title + native category)
                let market_info = kalshi::fetch_market_info_full(&trade.ticker).await;
                if let Some(ref info) = market_info {
                    trade.market_title = Some(info.title.clone());
                }

                // Category filter
                if let Some(ref title) = trade.market_title {
                    let has_native_match = market_info.as_ref()
                        .and_then(|info| info.category.as_ref())
                        .map(|cat| category_registry.matches_native_category(cat, &selected_categories))
                        .unwrap_or(false);

                    if !has_native_match {
                        if category_registry
                            .matches_selection(title, &selected_categories)
                            .is_none()
                        {
                            continue;
                        }
                    }
                }

                let outcome = kalshi::parse_ticker_details(&trade.ticker, &trade.taker_side);
                let action = trade.taker_side.to_uppercase();

                // Fetch market context early for filtering
                let market_ctx = kalshi::fetch_market_context(&trade.ticker).await;

                // Odds and spread filter
                if let Some(ref cfg) = config {
                    if let Some(ref ctx) = market_ctx {
                        // Skip if odds too high (near-certainty)
                        if ctx.yes_price > cfg.max_odds || ctx.no_price > cfg.max_odds {
                            continue;
                        }
                        // Skip if spread too low (dead market)
                        if cfg.min_spread > 0.0 && ctx.spread < cfg.min_spread {
                            continue;
                        }
                    }
                }

                print_kalshi_alert(&trade, trade_value, None);

                if let Some(ref ctx) = market_ctx {
                    print_market_context(ctx);
                }

                let order_book = kalshi::fetch_order_book(&trade.ticker).await;
                if let Some(ref ob) = order_book {
                    print_order_book(ob);
                }

                let alert_data = AlertData {
                    platform: "Kalshi",
                    market_title: trade.market_title.as_deref(),
                    outcome: Some(&outcome),
                    side: &action,
                    value: trade_value,
                    price: trade.yes_price / 100.0,
                    size: f64::from(trade.count),
                    timestamp: &trade.created_time,
                    wallet_id: None,
                    wallet_activity: None,
                    market_context: market_ctx.as_ref(),
                    whale_profile: None,
                    order_book: order_book.as_ref(),
                    top_holders: None,
                };

                history::log_alert(&alert_data, &conn);

                if let Some(ref cfg) = config {
                    if let Some(ref webhook_url) = cfg.webhook_url {
                        webhook::send_webhook_alert(webhook_url, &alert_data).await;
                    }
                }
            }
        }

        // Determine if we should use HTTP polling for Kalshi (fallback if WS is silent)
        let kalshi_ws_active = kalshi_ws_rx.is_some()
            && kalshi_ws_last_trade.elapsed() < kalshi_ws_fallback_threshold;

        // Check Polymarket
        if watch_polymarket { match polymarket::fetch_recent_trades(Some(threshold)).await {
            Ok(mut trades) => {
                if let Some(first_trade) = trades.first() {
                    let new_last_id = first_trade.id.clone();

                    for trade in &mut trades {
                        if let Some(ref last_id) = last_polymarket_trade_id {
                            if trade.id == *last_id {
                                break;
                            }
                        }

                        let trade_value = trade.size * trade.price;
                        if trade_value >= threshold as f64 {
                            // Category filter: skip if market doesn't match selected categories
                            if let Some(ref title) = trade.market_title {
                                if category_registry
                                    .matches_selection(title, &selected_categories)
                                    .is_none()
                                {
                                    continue;
                                }
                            }

                            let wallet_activity = if let Some(ref wallet_id) = trade.wallet_id {
                                wallet_tracker.record_transaction(wallet_id, trade_value);
                                Some(wallet_tracker.get_activity(wallet_id))
                            } else {
                                None
                            };

                            // Check for returning whale (12h memory)
                            let whale_scenario = trade.wallet_id.as_deref().and_then(|wid| {
                                wallet_tracker.classify_whale_return(
                                    &conn,
                                    wid,
                                    Some(&trade.asset_id),
                                    trade.outcome.as_deref(),
                                )
                            });

                            // Fetch market context early for filtering
                            let market_ctx = polymarket::fetch_market_context(&trade.market).await;

                            // Odds and spread filter
                            if let Some(ref cfg) = config {
                                if let Some(ref ctx) = market_ctx {
                                    // Skip if odds too high (near-certainty)
                                    if ctx.yes_price > cfg.max_odds || ctx.no_price > cfg.max_odds {
                                        continue;
                                    }
                                    // Skip if spread too low (dead market)
                                    if cfg.min_spread > 0.0 && ctx.spread < cfg.min_spread {
                                        continue;
                                    }
                                }
                            }

                            // Print returning whale info if detected
                            if let Some(ref scenario) = whale_scenario {
                                display::print_returning_whale(scenario, "Polymarket");
                            }

                            print_whale_alert(
                                "Polymarket",
                                trade,
                                trade_value,
                                wallet_activity.as_ref(),
                            );

                            if let Some(ref ctx) = market_ctx {
                                print_market_context(ctx);
                            }

                            // Fetch whale profile (Polymarket only - on-chain wallets)
                            let wp = if let Some(ref wallet_id) = trade.wallet_id {
                                whale_profile::fetch_whale_profile(wallet_id, &mut whale_cache).await
                            } else {
                                None
                            };
                            if let Some(ref profile) = wp {
                                print_whale_profile(profile);
                            }

                            // Fetch order book depth
                            let order_book = polymarket::fetch_order_book(&trade.asset_id).await;
                            if let Some(ref ob) = order_book {
                                print_order_book(ob);
                            }

                            // Fetch top holders
                            let top_holders = polymarket::fetch_top_holders(&trade.market).await;
                            if let Some(ref th) = top_holders {
                                print_top_holders(th);
                            }

                            let alert_data = AlertData {
                                platform: "Polymarket",
                                market_title: trade.market_title.as_deref(),
                                outcome: trade.outcome.as_deref(),
                                side: &trade.side,
                                value: trade_value,
                                price: trade.price,
                                size: trade.size,
                                timestamp: &trade.timestamp,
                                wallet_id: trade.wallet_id.as_deref(),
                                wallet_activity: wallet_activity.as_ref(),
                                market_context: market_ctx.as_ref(),
                                whale_profile: wp.as_ref(),
                                order_book: order_book.as_ref(),
                                top_holders: top_holders.as_ref(),
                            };

                            history::log_alert(&alert_data, &conn);

                            // Record to wallet memory DB
                            if let Some(ref wallet_id) = trade.wallet_id {
                                wallet_tracker.record_to_db(
                                    &conn,
                                    wallet_id,
                                    trade.market_title.as_deref(),
                                    Some(&trade.asset_id),
                                    trade.outcome.as_deref(),
                                    &trade.side,
                                    trade_value,
                                    trade.price,
                                    "Polymarket",
                                );
                            }

                            if let Some(ref cfg) = config {
                                if let Some(ref webhook_url) = cfg.webhook_url {
                                    webhook::send_webhook_alert(webhook_url, &alert_data).await;
                                }
                            }
                        }
                    }

                    last_polymarket_trade_id = Some(new_last_id);
                }
            }
            Err(e) => {
                eprintln!("{} {}", "[ERROR] Polymarket:".red(), e);
            }
        } } // end if watch_polymarket

        // Check Kalshi (HTTP polling fallback — only when WebSocket isn't active)
        if watch_kalshi && !kalshi_ws_active { match kalshi::fetch_recent_trades(config.as_ref()).await {
            Ok(mut trades) => {
                if let Some(first_trade) = trades.first() {
                    let new_last_id = first_trade.trade_id.clone();

                    for trade in &mut trades {
                        if let Some(ref last_id) = last_kalshi_trade_id {
                            if trade.trade_id == *last_id {
                                break;
                            }
                        }

                        let trade_value = (trade.yes_price / 100.0) * f64::from(trade.count);
                        if trade_value >= threshold as f64 {
                            // Fetch full market info (title + native category)
                            let market_info = kalshi::fetch_market_info_full(&trade.ticker).await;
                            if let Some(ref info) = market_info {
                                trade.market_title = Some(info.title.clone());
                            }

                            // Category filter: use native Kalshi category when available,
                            // fall back to keyword matching on title
                            if let Some(ref title) = trade.market_title {
                                let has_native_match = market_info.as_ref()
                                    .and_then(|info| info.category.as_ref())
                                    .map(|cat| category_registry.matches_native_category(cat, &selected_categories))
                                    .unwrap_or(false);

                                if !has_native_match {
                                    if category_registry
                                        .matches_selection(title, &selected_categories)
                                        .is_none()
                                    {
                                        continue;
                                    }
                                }
                            }

                            let outcome =
                                kalshi::parse_ticker_details(&trade.ticker, &trade.taker_side);

                            let action = trade.taker_side.to_uppercase();

                            // Fetch market context early for filtering
                            let market_ctx = kalshi::fetch_market_context(&trade.ticker).await;

                            // Odds and spread filter
                            if let Some(ref cfg) = config {
                                if let Some(ref ctx) = market_ctx {
                                    // Skip if odds too high (near-certainty)
                                    if ctx.yes_price > cfg.max_odds || ctx.no_price > cfg.max_odds {
                                        continue;
                                    }
                                    // Skip if spread too low (dead market)
                                    if cfg.min_spread > 0.0 && ctx.spread < cfg.min_spread {
                                        continue;
                                    }
                                }
                            }

                            print_kalshi_alert(trade, trade_value, None);

                            if let Some(ref ctx) = market_ctx {
                                print_market_context(ctx);
                            }

                            // Fetch order book depth for Kalshi
                            let order_book = kalshi::fetch_order_book(&trade.ticker).await;
                            if let Some(ref ob) = order_book {
                                print_order_book(ob);
                            }

                            let alert_data = AlertData {
                                platform: "Kalshi",
                                market_title: trade.market_title.as_deref(),
                                outcome: Some(&outcome),
                                side: &action,
                                value: trade_value,
                                price: trade.yes_price / 100.0,
                                size: f64::from(trade.count),
                                timestamp: &trade.created_time,
                                wallet_id: None,
                                wallet_activity: None,
                                market_context: market_ctx.as_ref(),
                                whale_profile: None,
                                order_book: order_book.as_ref(),
                                top_holders: None,
                            };

                            history::log_alert(&alert_data, &conn);

                            if let Some(ref cfg) = config {
                                if let Some(ref webhook_url) = cfg.webhook_url {
                                    webhook::send_webhook_alert(webhook_url, &alert_data).await;
                                }
                            }
                        }
                    }

                    last_kalshi_trade_id = Some(new_last_id);
                }
            }
            Err(e) => {
                eprintln!("{} {}", "[ERROR] Kalshi:".red(), e);
            }
        } } // end if watch_kalshi
    }
}
