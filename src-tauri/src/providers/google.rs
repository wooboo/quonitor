use async_trait::async_trait;
use chrono::Utc;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use oauth2::{
    basic::BasicClient, AuthUrl, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope,
    TokenResponse, TokenUrl,
};
use url::Url;

use super::{QuotaProvider, QuotaData};
use crate::db::Credentials;
use crate::error::{QuonitorError, Result};

pub struct GoogleProvider {
    client: Client,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GoogleAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

impl GoogleProvider {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub fn get_auth_url(config: &GoogleAuthConfig) -> Result<(String, String)> {
        let client = BasicClient::new(
            ClientId::new(config.client_id.clone()),
            Some(ClientSecret::new(config.client_secret.clone())),
            AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
                .map_err(|e| QuonitorError::Config(format!("Invalid auth URL: {}", e)))?,
            Some(TokenUrl::new("https://oauth2.googleapis.com/token".to_string())
                .map_err(|e| QuonitorError::Config(format!("Invalid token URL: {}", e)))?)
        )
        .set_redirect_uri(RedirectUrl::new(config.redirect_uri.clone())
            .map_err(|e| QuonitorError::Config(format!("Invalid redirect URI: {}", e)))?);

        let (auth_url, csrf_token) = client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("https://www.googleapis.com/auth/cloud-platform".to_string()))
            .add_scope(Scope::new("email".to_string()))
            .url();

        Ok((auth_url.to_string(), csrf_token.secret().clone()))
    }

    pub async fn exchange_code(config: &GoogleAuthConfig, code: String) -> Result<String> {
        let client = BasicClient::new(
            ClientId::new(config.client_id.clone()),
            Some(ClientSecret::new(config.client_secret.clone())),
            AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())?,
            Some(TokenUrl::new("https://oauth2.googleapis.com/token".to_string())?)
        )
        .set_redirect_uri(RedirectUrl::new(config.redirect_uri.clone())?);

        let token_result = client
            .exchange_code(oauth2::AuthorizationCode::new(code))
            .request_async(oauth2::reqwest::async_http_client)
            .await
            .map_err(|e| QuonitorError::Auth(format!("Token exchange failed: {}", e)))?;

        Ok(token_result.access_token().secret().clone())
    }
}

#[async_trait]
impl QuotaProvider for GoogleProvider {
    async fn fetch_quota(&self, credentials: &Credentials) -> Result<QuotaData> {
        // OAuth token should be in credentials.oauth_token
        let token = credentials.oauth_token.as_ref()
            .ok_or_else(|| QuonitorError::Auth("Google requires OAuth token".to_string()))?;

        // Validate token and get project info
        // We use the 'userinfo' or 'tokeninfo' endpoint or just try listing projects
        // For now, let's hit the Cloud Resource Manager API to list projects as validation
        let url = "https://cloudresourcemanager.googleapis.com/v1/projects?pageSize=1";

        let response = self.client
            .get(url)
            .bearer_auth(token)
            .send()
            .await?;

        if !response.status().is_success() {
             return Err(QuonitorError::Provider(format!(
                "Google API error: {}", response.status()
            )));
        }

        // If successful, return 0 usage for now (placeholder until we hook up Billing)
        let now = Utc::now();

        Ok(QuotaData {
            account_id: "google-account".to_string(), // TODO: Fetch real user email/ID
            timestamp: now.timestamp(),
            tokens_input: Some(0),
            tokens_output: Some(0),
            cost_usd: Some(0.0),
            quota_limit: None,
            quota_remaining: None,
            model_breakdown: vec![],
            metadata: Some("Google Cloud tracking enabled".to_string()),
        })
    }

    fn supports_oauth(&self) -> bool {
        true
    }

    fn provider_name(&self) -> &'static str {
        "Google"
    }
}
