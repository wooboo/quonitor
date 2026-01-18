use std::sync::Arc;
use chrono::Utc;
use uuid::Uuid;
use tauri::State;
use serde::{Deserialize, Serialize};

use crate::db::{Repository, Account, Credentials, QuotaSnapshot, ModelUsage};
use crate::services::{Aggregator, Cache, Scheduler};
use crate::crypto::CryptoService;
use crate::providers::QuotaData;
use crate::error::{QuonitorError, Result};

pub struct AppState {
    pub repo: Arc<Repository>,
    pub aggregator: Arc<Aggregator>,
    pub cache: Arc<Cache>,
    pub scheduler: Arc<Scheduler>,
    pub crypto: Arc<CryptoService>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddAccountRequest {
    pub provider: String,
    pub name: String,
    pub credentials: Credentials,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountResponse {
    pub id: String,
    pub provider: String,
    pub name: String,
    pub created_at: i64,
    pub last_synced: Option<i64>,
}

impl From<Account> for AccountResponse {
    fn from(account: Account) -> Self {
        Self {
            id: account.id,
            provider: account.provider,
            name: account.name,
            created_at: account.created_at,
            last_synced: account.last_synced,
        }
    }
}

#[tauri::command]
pub async fn add_account(
    request: AddAccountRequest,
    state: State<'_, AppState>,
) -> Result<AccountResponse> {
    let account_id = Uuid::new_v4().to_string();

    let initial_quota = state.aggregator.validate_credentials(&request.provider, &request.credentials).await?;

    let creds_json = serde_json::to_string(&request.credentials)?;
    let encrypted_creds = state.crypto.encrypt(&creds_json)?;

    let account = Account {
        id: account_id.clone(),
        provider: request.provider,
        name: request.name,
        credentials_encrypted: encrypted_creds,
        created_at: Utc::now().timestamp(),
        last_synced: None,
    };

    state.repo.insert_account(&account).await
        .map_err(|e| QuonitorError::Database(e))?;

    let mut quota_to_store = initial_quota;
    quota_to_store.account_id = account_id.clone();
    
    state.cache.set(account_id.clone(), quota_to_store).await;

    tokio::spawn({
        let aggregator = state.aggregator.clone();
        let cache = state.cache.clone();
        let id = account_id.clone();
        async move {
            match aggregator.fetch_account_quota(&id).await {
                Ok(quota) => {
                    cache.set(quota.account_id.clone(), quota).await;
                }
                Err(e) => {
                    tracing::error!("Failed to fetch quota for new account: {}", e);
                }
            }
        }
    });

    Ok(AccountResponse::from(account))
}

#[tauri::command]
pub async fn remove_account(
    account_id: String,
    state: State<'_, AppState>,
) -> Result<()> {
    state.repo.delete_account(&account_id).await
        .map_err(|e| QuonitorError::Database(e))?;
    state.cache.remove(&account_id).await;
    Ok(())
}

#[tauri::command]
pub async fn get_all_accounts(
    state: State<'_, AppState>,
) -> Result<Vec<AccountResponse>> {
    let accounts = state.repo.get_all_accounts().await
        .map_err(|e| QuonitorError::Database(e))?;
    Ok(accounts.into_iter().map(AccountResponse::from).collect())
}

#[tauri::command]
pub async fn get_all_quotas(
    state: State<'_, AppState>,
) -> Result<Vec<QuotaData>> {
    Ok(state.cache.get_all().await)
}

#[tauri::command]
pub async fn get_quota(
    account_id: String,
    state: State<'_, AppState>,
) -> Result<Option<QuotaData>> {
    Ok(state.cache.get(&account_id).await)
}

#[tauri::command]
pub async fn refresh_now(
    state: State<'_, AppState>,
) -> Result<()> {
    state.scheduler.run_fetch_cycle().await;
    Ok(())
}

#[tauri::command]
pub async fn refresh_account(
    account_id: String,
    state: State<'_, AppState>,
) -> Result<QuotaData> {
    let quota = state.aggregator.fetch_account_quota(&account_id).await?;
    state.cache.set(quota.account_id.clone(), quota.clone()).await;
    Ok(quota)
}

#[tauri::command]
pub async fn get_historical_snapshots(
    account_id: String,
    days: u32,
    state: State<'_, AppState>,
) -> Result<Vec<QuotaSnapshot>> {
    let since = Utc::now().timestamp() - (days as i64 * 86400);
    state.repo.get_snapshots_since(&account_id, since).await
        .map_err(|e| QuonitorError::Database(e))
}

#[tauri::command]
pub async fn get_model_usage_history(
    account_id: String,
    days: u32,
    state: State<'_, AppState>,
) -> Result<Vec<ModelUsage>> {
    let since = Utc::now().timestamp() - (days as i64 * 86400);
    state.repo.get_model_usage_since(&account_id, since).await
        .map_err(|e| QuonitorError::Database(e))
}

#[tauri::command]
pub async fn get_setting(
    key: String,
    state: State<'_, AppState>,
) -> Result<Option<String>> {
    state.repo.get_setting(&key).await
        .map_err(|e| QuonitorError::Database(e))
}

#[tauri::command]
pub async fn set_setting(
    key: String,
    value: String,
    state: State<'_, AppState>,
) -> Result<()> {
    state.repo.set_setting(&key, &value).await
        .map_err(|e| QuonitorError::Database(e))?;

    // Handle special settings
    if key == "refresh_interval_seconds" {
        if let Ok(seconds) = value.parse::<u64>() {
            state.scheduler.set_interval(seconds).await;
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn cleanup_old_data(
    days: i64,
    state: State<'_, AppState>,
) -> Result<()> {
    state.repo.cleanup_old_data(days).await
        .map_err(|e| QuonitorError::Database(e))
}
