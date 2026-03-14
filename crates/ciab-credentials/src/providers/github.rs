use ciab_core::error::CiabResult;
use ciab_core::types::config::OAuthProviderConfig;
use ciab_core::types::credentials::OAuthToken;

use crate::oauth2_flow::{DeviceCodeResponse, OAuth2Flow, OAuth2PollResult};

pub struct GitHubOAuth {
    flow: OAuth2Flow,
}

impl GitHubOAuth {
    pub fn new(config: &OAuthProviderConfig, client_secret: String) -> Self {
        Self {
            flow: OAuth2Flow::new(config, client_secret),
        }
    }

    pub fn device_code_url() -> &'static str {
        "https://github.com/login/device/code"
    }

    pub fn token_url() -> &'static str {
        "https://github.com/login/oauth/access_token"
    }

    pub fn authorization_url(&self, state: &str) -> String {
        self.flow.authorization_url(state)
    }

    pub async fn exchange_code(&self, code: &str) -> CiabResult<OAuthToken> {
        self.flow.exchange_code(code).await
    }

    pub async fn refresh_token(&self, refresh_token: &str) -> CiabResult<OAuthToken> {
        self.flow.refresh_token(refresh_token).await
    }

    pub async fn device_code_request(&self) -> CiabResult<DeviceCodeResponse> {
        self.flow.device_code_request().await
    }

    pub async fn device_code_poll(&self, device_code: &str) -> CiabResult<OAuth2PollResult> {
        self.flow.device_code_poll(device_code).await
    }
}
