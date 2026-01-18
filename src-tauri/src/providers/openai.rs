use async_trait::async_trait;
use chrono::Utc;
use serde::Deserialize;
use super::{QuotaProvider, QuotaData, ModelData};
use crate::db::Credentials;
use crate::error::{QuonitorError, Result};

pub struct OpenAIProvider {
    client: reqwest::Client,
}

#[derive(Debug, Deserialize)]
struct UsageResponse {
    data: Vec<UsageDataPoint>,
}

#[derive(Debug, Deserialize)]
struct UsageDataPoint {
    #[serde(default)]
    _aggregation_timestamp: Option<i64>,
    #[serde(default)]
    n_requests: i64,
    #[serde(default)]
    _operation: Option<String>,
    #[serde(default)]
    _snapshot_id: Option<String>,
    #[serde(default)]
    n_context_tokens_total: i64,
    #[serde(default)]
    n_generated_tokens_total: i64,
    #[serde(default)]
    model: Option<String>,
}

impl OpenAIProvider {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    fn calculate_openai_cost(model: &str, input_tokens: i64, output_tokens: i64) -> f64 {
        // Pricing per million tokens (as of 2026)
        let (input_price, output_price) = match model {
            m if m.contains("gpt-4o") => (2.50, 10.00),
            m if m.contains("gpt-4-turbo") => (10.00, 30.00),
            m if m.contains("gpt-4") => (30.00, 60.00),
            m if m.contains("gpt-3.5-turbo") => (0.50, 1.50),
            m if m.contains("o1-preview") => (15.00, 60.00),
            m if m.contains("o1-mini") => (3.00, 12.00),
            _ => (1.00, 2.00), // Default fallback
        };

        let input_cost = (input_tokens as f64 / 1_000_000.0) * input_price;
        let output_cost = (output_tokens as f64 / 1_000_000.0) * output_price;

        input_cost + output_cost
    }
}

#[async_trait]
impl QuotaProvider for OpenAIProvider {
    async fn fetch_quota(&self, credentials: &Credentials) -> Result<QuotaData> {
        let api_key = credentials.api_key.as_ref()
            .ok_or_else(|| QuonitorError::Auth("OpenAI requires API key".to_string()))?;

        // Fetch usage data for the last day with per-model breakdown
        let now = Utc::now();
        let start_time = now - chrono::Duration::days(1);

        let url = format!(
            "https://api.openai.com/v1/organization/usage/completions?start_time={}&end_time={}&bucket_width=1d&group_by=model",
            start_time.timestamp(),
            now.timestamp()
        );

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(QuonitorError::Provider(format!(
                "OpenAI API error ({}): {}",
                status, error_text
            )));
        }

        let usage_response: UsageResponse = response.json().await?;

        // Aggregate by model
        let mut model_map: std::collections::HashMap<String, (i64, i64, i64)> = std::collections::HashMap::new();

        for data_point in usage_response.data {
            let model_name = data_point.model.unwrap_or_else(|| "unknown".to_string());
            let entry = model_map.entry(model_name).or_insert((0, 0, 0));
            entry.0 += data_point.n_context_tokens_total;
            entry.1 += data_point.n_generated_tokens_total;
            entry.2 += data_point.n_requests;
        }

        // Calculate totals and per-model data
        let mut total_input = 0i64;
        let mut total_output = 0i64;
        let mut total_cost = 0.0f64;
        let mut model_breakdown = Vec::new();

        for (model_name, (input, output, requests)) in model_map {
            total_input += input;
            total_output += output;

            let cost = Self::calculate_openai_cost(&model_name, input, output);
            total_cost += cost;

            model_breakdown.push(ModelData {
                model_name,
                tokens_input: input,
                tokens_output: output,
                cost_usd: cost,
                request_count: requests,
            });
        }

        Ok(QuotaData {
            account_id: String::new(), // Will be set by caller
            timestamp: now.timestamp(),
            tokens_input: Some(total_input),
            tokens_output: Some(total_output),
            cost_usd: Some(total_cost),
            quota_limit: None, // OpenAI doesn't expose hard limits via API
            quota_remaining: None,
            model_breakdown,
            metadata: None,
        })
    }

    fn supports_oauth(&self) -> bool {
        false
    }

    fn provider_name(&self) -> &'static str {
        "OpenAI"
    }
}
