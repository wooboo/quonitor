// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod db;
mod error;
mod crypto;
mod providers;
mod services;
mod api;
mod tray;

use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use db::Repository;
use crypto::CryptoService;
use providers::ProviderRegistry;
use services::{Aggregator, Notifier, Cache, Scheduler};
use api::{AppState, commands::*};

#[tokio::main]
async fn main() {
    // Set environment variable to fix rendering issues on some Linux configurations
    std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");

    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "quonitor=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Get data directory
    let data_dir = dirs::data_local_dir()
        .map(|p| p.join("quonitor"))
        .expect("Failed to get app data directory");

    std::fs::create_dir_all(&data_dir).expect("Failed to create data directory");

    let db_path = data_dir.join("quonitor.db");
    let db_url = format!("sqlite://{}?mode=rwc", db_path.display());

    // Initialize database
    let repo = Arc::new(
        Repository::new(&db_url)
            .await
            .expect("Failed to initialize database")
    );

    // Initialize crypto service
    let crypto = Arc::new(
        CryptoService::new()
            .expect("Failed to initialize crypto service")
    );

    // Initialize providers
    let providers = Arc::new(ProviderRegistry::new());

    // Initialize services
    let cache = Arc::new(Cache::new());
    let aggregator = Arc::new(Aggregator::new(
        repo.clone(),
        providers.clone(),
        crypto.clone(),
    ));
    let notifier = Arc::new(Notifier::new(repo.clone()));

    // Get refresh interval from settings
    let interval = repo
        .get_setting("refresh_interval_seconds")
        .await
        .ok()
        .flatten()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(300);

    let scheduler = Arc::new(Scheduler::new(
        aggregator.clone(),
        notifier.clone(),
        cache.clone(),
        interval,
    ));

    // Create app state
    let app_state = AppState {
        repo,
        aggregator,
        cache,
        scheduler: scheduler.clone(),
        crypto,
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(app_state)
        .setup(move |app| {
            // Create system tray
            let _tray = tray::create_tray(&app.handle())?;

            // Start scheduler
            let scheduler_clone = scheduler.clone();
            tauri::async_runtime::spawn(async move {
                scheduler_clone.start().await;
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            api::commands::get_accounts,
            api::commands::add_account,
            api::commands::remove_account,
            api::commands::get_historical_data,
            api::commands::get_model_usage_history,
            api::commands::update_settings,
            api::commands::get_settings,
            api::commands::manual_refresh,
            api::commands::google_auth_start,
            api::commands::google_auth_finish,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
