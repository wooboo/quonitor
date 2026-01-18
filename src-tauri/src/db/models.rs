use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub provider: String,
    pub name: String,
    #[serde(skip_serializing)]
    pub credentials_encrypted: Vec<u8>,
    pub created_at: i64,
    pub last_synced: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaSnapshot {
    pub id: Option<i64>,
    pub account_id: String,
    pub timestamp: i64,
    pub tokens_input: Option<i64>,
    pub tokens_output: Option<i64>,
    pub cost_usd: Option<f64>,
    pub quota_limit: Option<i64>,
    pub quota_remaining: Option<i64>,
    pub metadata: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelUsage {
    pub id: Option<i64>,
    pub account_id: String,
    pub model_name: String,
    pub timestamp: i64,
    pub tokens_input: i64,
    pub tokens_output: i64,
    pub cost_usd: f64,
    pub request_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationState {
    pub account_id: String,
    pub last_75_percent_notified: Option<i64>,
    pub last_90_percent_notified: Option<i64>,
    pub last_95_percent_notified: Option<i64>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Setting {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub api_key: Option<String>,
    pub oauth_token: Option<String>,
    pub oauth_refresh_token: Option<String>,
}

impl Credentials {
    #[allow(dead_code)]
    pub fn new_api_key(api_key: String) -> Self {
        Self {
            api_key: Some(api_key),
            oauth_token: None,
            oauth_refresh_token: None,
        }
    }

    #[allow(dead_code)]
    pub fn new_oauth(token: String, refresh_token: Option<String>) -> Self {
        Self {
            api_key: None,
            oauth_token: Some(token),
            oauth_refresh_token: refresh_token,
        }
    }
}
