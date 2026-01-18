use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::db::Credentials;
use crate::error::Result;

pub mod openai;
pub mod anthropic;
pub mod google;
pub mod github;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaData {
    pub account_id: String,
    pub timestamp: i64,
    pub tokens_input: Option<i64>,
    pub tokens_output: Option<i64>,
    pub cost_usd: Option<f64>,
    pub quota_limit: Option<i64>,
    pub quota_remaining: Option<i64>,
    pub model_breakdown: Vec<ModelData>,
    pub metadata: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelData {
    pub model_name: String,
    pub tokens_input: i64,
    pub tokens_output: i64,
    pub cost_usd: f64,
    pub request_count: i64,
}

#[async_trait]
pub trait QuotaProvider: Send + Sync {
    async fn fetch_quota(&self, credentials: &Credentials) -> Result<QuotaData>;
    #[allow(dead_code)]
    fn supports_oauth(&self) -> bool;
    #[allow(dead_code)]
    fn provider_name(&self) -> &'static str;
}

pub struct ProviderRegistry {
    providers: std::collections::HashMap<String, Box<dyn QuotaProvider>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        let mut providers: std::collections::HashMap<String, Box<dyn QuotaProvider>> = std::collections::HashMap::new();

        providers.insert("openai".to_string(), Box::new(openai::OpenAIProvider::new()));
        providers.insert("anthropic".to_string(), Box::new(anthropic::AnthropicProvider::new()));
        providers.insert("google".to_string(), Box::new(google::GoogleProvider::new()));
        providers.insert("github".to_string(), Box::new(github::GitHubProvider::new()));

        Self { providers }
    }

    pub fn get(&self, provider: &str) -> Option<&Box<dyn QuotaProvider>> {
        self.providers.get(provider)
    }

    #[allow(dead_code)]
    pub fn list_providers(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }
}
