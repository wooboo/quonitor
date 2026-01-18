use async_trait::async_trait;
use chrono::Utc;
use super::{QuotaProvider, QuotaData};
use crate::db::Credentials;
use crate::error::{QuonitorError, Result};

pub struct GitHubProvider {
    _client: reqwest::Client,
}

impl GitHubProvider {
    pub fn new() -> Self {
        Self {
            _client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl QuotaProvider for GitHubProvider {
    async fn fetch_quota(&self, credentials: &Credentials) -> Result<QuotaData> {
        // GitHub Copilot requires OAuth token or PAT
        let _token = credentials.oauth_token.as_ref()
            .or(credentials.api_key.as_ref())
            .ok_or_else(|| QuonitorError::Auth("GitHub requires OAuth token or PAT".to_string()))?;

        // TODO: Implement GitHub GraphQL API integration for Copilot metrics
        // This requires organization access and specific GraphQL queries

        let now = Utc::now();

        Ok(QuotaData {
            account_id: String::new(),
            timestamp: now.timestamp(),
            tokens_input: Some(0),
            tokens_output: Some(0),
            cost_usd: Some(0.0),
            quota_limit: None,
            quota_remaining: None,
            model_breakdown: vec![],
            metadata: Some("GitHub provider not fully implemented yet".to_string()),
        })
    }

    fn supports_oauth(&self) -> bool {
        true
    }

    fn provider_name(&self) -> &'static str {
        "GitHub Copilot"
    }
}
