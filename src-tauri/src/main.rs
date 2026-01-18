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
use tauri::Manager;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use db::Repository;
use crypto::CryptoService;
use providers::ProviderRegistry;
use services::{Aggregator, Notifier, Cache, Scheduler};
use api::{AppState, commands::*};

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "quonitor=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Get data directory
    let data_dir = tauri::api::path::app_data_dir(&tauri::Config::default())
        .expect("Failed to get app data directory");

    std::fs::create_dir_all(&data_dir).expect("Failed to create data directory");

    let db_path = data_dir.join("quonitor.db");
    let db_url = format!("sqlite://{}", db_path.display());

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
        .setup(|app| {
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
            add_account,
            remove_account,
            get_all_accounts,
            get_all_quotas,
            get_quota,
            refresh_now,
            refresh_account,
            get_historical_snapshots,
            get_model_usage_history,
            get_setting,
            set_setting,
            cleanup_old_data,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
