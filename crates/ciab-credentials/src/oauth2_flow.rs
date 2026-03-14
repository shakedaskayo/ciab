use std::fmt::Write;

use chrono::Utc;
use ciab_core::error::{CiabError, CiabResult};
use ciab_core::types::config::OAuthProviderConfig;
use ciab_core::types::credentials::OAuthToken;
use serde::{Deserialize, Serialize};

/// Simple percent-encoding for URL query parameters.
fn simple_encode(input: &str) -> String {
    let mut encoded = String::new();
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            _ => {
                let _ = write!(encoded, "%{:02X}", byte);
            }
        }
    }
    encoded
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
}

#[derive(Debug, Clone)]
pub enum OAuth2PollResult {
    Pending,
    Complete(OAuthToken),
    Error(String),
}

pub struct OAuth2Flow {
    client_id: String,
    client_secret: String,
    auth_url: String,
    token_url: String,
    redirect_uri: String,
    scopes: Vec<String>,
}

impl OAuth2Flow {
    pub fn new(config: &OAuthProviderConfig, client_secret: String) -> Self {
        Self {
            client_id: config.client_id.clone(),
            client_secret,
            auth_url: config.auth_url.clone(),
            token_url: config.token_url.clone(),
            redirect_uri: config.redirect_uri.clone(),
            scopes: config.scopes.clone(),
        }
    }

    pub fn authorization_url(&self, state: &str) -> String {
        let scopes = self.scopes.join(" ");
        format!(
            "{}?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}",
            self.auth_url,
            simple_encode(&self.client_id),
            simple_encode(&self.redirect_uri),
            simple_encode(&scopes),
            simple_encode(state),
        )
    }

    pub async fn exchange_code(&self, code: &str) -> CiabResult<OAuthToken> {
        let client = reqwest::Client::new();
        let params = [
            ("grant_type", "authorization_code"),
            ("code", code),
            ("client_id", &self.client_id),
            ("client_secret", &self.client_secret),
            ("redirect_uri", &self.redirect_uri),
        ];

        let response = client
            .post(&self.token_url)
            .form(&params)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| {
                CiabError::OAuthFlowFailed(format!("token exchange request failed: {e}"))
            })?;

        if !response.status().is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(CiabError::OAuthFlowFailed(format!(
                "token exchange failed: {body}"
            )));
        }

        let token_response: TokenResponse = response.json().await.map_err(|e| {
            CiabError::OAuthFlowFailed(format!("failed to parse token response: {e}"))
        })?;

        Ok(token_response.into_oauth_token())
    }

    pub async fn refresh_token(&self, refresh_token: &str) -> CiabResult<OAuthToken> {
        let client = reqwest::Client::new();
        let params = [
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("client_id", &self.client_id),
            ("client_secret", &self.client_secret),
        ];

        let response = client
            .post(&self.token_url)
            .form(&params)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| CiabError::OAuthFlowFailed(format!("refresh request failed: {e}")))?;

        if !response.status().is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(CiabError::OAuthFlowFailed(format!(
                "token refresh failed: {body}"
            )));
        }

        let token_response: TokenResponse = response.json().await.map_err(|e| {
            CiabError::OAuthFlowFailed(format!("failed to parse token response: {e}"))
        })?;

        Ok(token_response.into_oauth_token())
    }

    pub async fn device_code_request(&self) -> CiabResult<DeviceCodeResponse> {
        let client = reqwest::Client::new();
        let scopes = self.scopes.join(" ");
        let params = [("client_id", &self.client_id), ("scope", &scopes)];

        // Use auth_url with /device/code convention, or a well-known path
        let device_url = if self.auth_url.ends_with("/authorize") {
            self.auth_url.replace("/authorize", "/device/code")
        } else {
            format!("{}/device/code", self.auth_url.trim_end_matches('/'))
        };

        let response = client
            .post(&device_url)
            .form(&params)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| CiabError::OAuthFlowFailed(format!("device code request failed: {e}")))?;

        if !response.status().is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(CiabError::OAuthFlowFailed(format!(
                "device code request failed: {body}"
            )));
        }

        let device_response: DeviceCodeResponse = response.json().await.map_err(|e| {
            CiabError::OAuthFlowFailed(format!("failed to parse device code response: {e}"))
        })?;

        Ok(device_response)
    }

    pub async fn device_code_poll(&self, device_code: &str) -> CiabResult<OAuth2PollResult> {
        let client = reqwest::Client::new();
        let params = [
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ("device_code", device_code),
            ("client_id", &self.client_id),
        ];

        let response = client
            .post(&self.token_url)
            .form(&params)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| CiabError::OAuthFlowFailed(format!("device code poll failed: {e}")))?;

        let body: serde_json::Value = response.json().await.map_err(|e| {
            CiabError::OAuthFlowFailed(format!("failed to parse poll response: {e}"))
        })?;

        if let Some(error) = body.get("error").and_then(|e| e.as_str()) {
            match error {
                "authorization_pending" | "slow_down" => Ok(OAuth2PollResult::Pending),
                _ => {
                    let desc = body
                        .get("error_description")
                        .and_then(|d| d.as_str())
                        .unwrap_or(error);
                    Ok(OAuth2PollResult::Error(desc.to_string()))
                }
            }
        } else {
            // Success - parse token
            let token_response: TokenResponse = serde_json::from_value(body)?;
            Ok(OAuth2PollResult::Complete(
                token_response.into_oauth_token(),
            ))
        }
    }
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    #[serde(default)]
    refresh_token: Option<String>,
    #[serde(default = "default_token_type")]
    token_type: String,
    #[serde(default)]
    expires_in: Option<i64>,
    #[serde(default)]
    scope: Option<String>,
}

fn default_token_type() -> String {
    "bearer".to_string()
}

impl TokenResponse {
    fn into_oauth_token(self) -> OAuthToken {
        let expires_at = self
            .expires_in
            .map(|secs| Utc::now() + chrono::Duration::seconds(secs));
        let scopes = self
            .scope
            .map(|s| s.split_whitespace().map(String::from).collect())
            .unwrap_or_default();

        OAuthToken {
            access_token: self.access_token,
            refresh_token: self.refresh_token,
            token_type: self.token_type,
            expires_at,
            scopes,
        }
    }
}
