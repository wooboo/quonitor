use std::sync::Arc;
use chrono::Utc;
use crate::db::{Repository, Credentials, QuotaSnapshot, ModelUsage};
use crate::providers::{ProviderRegistry, QuotaData};
use crate::crypto::CryptoService;
use crate::error::Result;
use tracing::{info, error};

pub struct Aggregator {
    repo: Arc<Repository>,
    providers: Arc<ProviderRegistry>,
    crypto: Arc<CryptoService>,
}

impl Aggregator {
    pub fn new(repo: Arc<Repository>, providers: Arc<ProviderRegistry>, crypto: Arc<CryptoService>) -> Self {
        Self {
            repo,
            providers,
            crypto,
        }
    }

    pub async fn fetch_all_quotas(&self) -> Vec<QuotaData> {
        let accounts = match self.repo.get_all_accounts().await {
            Ok(accounts) => accounts,
            Err(e) => {
                error!("Failed to get accounts: {}", e);
                return vec![];
            }
        };

        let mut quotas = Vec::new();

        for account in accounts {
            match self.fetch_account_quota(&account.id).await {
                Ok(quota) => quotas.push(quota),
                Err(e) => {
                    error!("Failed to fetch quota for account {}: {}", account.id, e);
                }
            }
        }

        quotas
    }

    pub async fn validate_credentials(&self, provider_id: &str, credentials: &Credentials) -> Result<QuotaData> {
        let provider = self.providers.get(provider_id)
            .ok_or_else(|| crate::error::QuonitorError::Config(format!("Provider {} not found", provider_id)))?;

        let quota = provider.fetch_quota(credentials).await?;
        Ok(quota)
    }

    pub async fn fetch_account_quota(&self, account_id: &str) -> Result<QuotaData> {
        let account = self.repo.get_account(account_id).await?
            .ok_or_else(|| crate::error::QuonitorError::Config(format!("Account {} not found", account_id)))?;

        let provider = self.providers.get(&account.provider)
            .ok_or_else(|| crate::error::QuonitorError::Config(format!("Provider {} not found", account.provider)))?;

        // Decrypt credentials
        let creds_json = self.crypto.decrypt(&account.credentials_encrypted)?;
        let credentials: Credentials = serde_json::from_str(&creds_json)?;

        // Fetch quota from provider
        let mut quota = provider.fetch_quota(&credentials).await?;
        quota.account_id = account_id.to_string();

        // Store in database
        self.store_quota(&quota).await?;

        // Update account sync time
        self.repo.update_account_sync_time(account_id, Utc::now().timestamp()).await?;

        info!("Fetched quota for account {}: {} models", account_id, quota.model_breakdown.len());

        Ok(quota)
    }

    async fn store_quota(&self, quota: &QuotaData) -> Result<()> {
        // Store account-level snapshot
        let snapshot = QuotaSnapshot {
            id: None,
            account_id: quota.account_id.clone(),
            timestamp: quota.timestamp,
            tokens_input: quota.tokens_input,
            tokens_output: quota.tokens_output,
            cost_usd: quota.cost_usd,
            quota_limit: quota.quota_limit,
            quota_remaining: quota.quota_remaining,
            metadata: quota.metadata.clone(),
        };

        self.repo.insert_quota_snapshot(&snapshot).await?;

        // Store per-model usage
        for model in &quota.model_breakdown {
            let usage = ModelUsage {
                id: None,
                account_id: quota.account_id.clone(),
                model_name: model.model_name.clone(),
                timestamp: quota.timestamp,
                tokens_input: model.tokens_input,
                tokens_output: model.tokens_output,
                cost_usd: model.cost_usd,
                request_count: model.request_count,
            };

            self.repo.insert_model_usage(&usage).await?;
        }

        Ok(())
    }
}
