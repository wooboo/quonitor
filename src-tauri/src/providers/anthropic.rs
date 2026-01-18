use async_trait::async_trait;
use chrono::Utc;
use serde::Deserialize;
use super::{QuotaProvider, QuotaData, ModelData};
use crate::db::Credentials;
use crate::error::{QuonitorError, Result};

pub struct AnthropicProvider {
    client: reqwest::Client,
}

#[derive(Debug, Deserialize)]
struct UsageResponse {
    #[serde(default)]
    data: Vec<UsageDataPoint>,
}

#[derive(Debug, Deserialize)]
struct UsageDataPoint {
    #[serde(default)]
    _timestamp: Option<String>,
    #[serde(default)]
    input_tokens: i64,
    #[serde(default)]
    output_tokens: i64,
    #[serde(default)]
    model: Option<String>,
}

impl AnthropicProvider {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    fn calculate_anthropic_cost(model: &str, input_tokens: i64, output_tokens: i64) -> f64 {
        // Pricing per million tokens (as of 2026)
        let (input_price, output_price) = match model {
            m if m.contains("opus") => (15.00, 75.00),
            m if m.contains("sonnet") => (3.00, 15.00),
            m if m.contains("haiku") => (0.25, 1.25),
            _ => (3.00, 15.00), // Default to Sonnet pricing
        };

        let input_cost = (input_tokens as f64 / 1_000_000.0) * input_price;
        let output_cost = (output_tokens as f64 / 1_000_000.0) * output_price;

        input_cost + output_cost
    }
}

#[async_trait]
impl QuotaProvider for AnthropicProvider {
    async fn fetch_quota(&self, credentials: &Credentials) -> Result<QuotaData> {
        let api_key = credentials.api_key.as_ref()
            .ok_or_else(|| QuonitorError::Auth("Anthropic requires API key".to_string()))?;

        // Anthropic does not currently provide a public API for retrieving historical usage/cost.
        // We validate the key by listing models, and return 0 usage.
        
        let url = "https://api.anthropic.com/v1/models?limit=1";

        let response = self.client
            .get(url)
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(QuonitorError::Provider(format!(
                "Anthropic API error ({}): {}",
                status, error_text
            )));
        }

        // Key is valid if we got here.
        // Return placeholder data since we can't fetch real usage.
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
            metadata: Some("Anthropic API does not support usage tracking yet".to_string()),
        })
    }

    fn supports_oauth(&self) -> bool {
        false
    }

    fn provider_name(&self) -> &'static str {
        "Anthropic"
    }
}
