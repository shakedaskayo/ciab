use axum::extract::{Path, Query, State};
use axum::response::{IntoResponse, Redirect};
use axum::Json;
use ciab_core::error::CiabError;
use ciab_credentials::oauth2_flow::{OAuth2Flow, OAuth2PollResult};
use serde::Deserialize;
use uuid::Uuid;

use crate::state::AppState;

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

fn get_oauth_flow(state: &AppState, provider: &str) -> Result<OAuth2Flow, CiabError> {
    let oauth_config = state
        .config
        .oauth
        .as_ref()
        .ok_or_else(|| CiabError::ConfigError("OAuth not configured".to_string()))?;

    let provider_config = oauth_config.providers.get(provider).ok_or_else(|| {
        CiabError::ConfigError(format!("OAuth provider '{}' not configured", provider))
    })?;

    let client_secret = std::env::var(&provider_config.client_secret_env).map_err(|_| {
        CiabError::ConfigError(format!(
            "OAuth client secret env var '{}' not set",
            provider_config.client_secret_env
        ))
    })?;

    Ok(OAuth2Flow::new(provider_config, client_secret))
}

// ---------------------------------------------------------------------------
// authorize — redirect to OAuth provider
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct AuthorizeQuery {
    #[serde(default = "default_state")]
    pub state: String,
}

fn default_state() -> String {
    Uuid::new_v4().to_string()
}

pub async fn authorize(
    State(state): State<AppState>,
    Path(provider): Path<String>,
    Query(params): Query<AuthorizeQuery>,
) -> Result<impl IntoResponse, CiabError> {
    let flow = get_oauth_flow(&state, &provider)?;
    let url = flow.authorization_url(&params.state);
    Ok(Redirect::temporary(&url))
}

// ---------------------------------------------------------------------------
// callback — handle OAuth callback, exchange code, store token
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CallbackQuery {
    pub code: String,
    #[serde(default)]
    pub state: Option<String>,
}

pub async fn callback(
    State(state): State<AppState>,
    Path(provider): Path<String>,
    Query(params): Query<CallbackQuery>,
) -> Result<impl IntoResponse, CiabError> {
    let flow = get_oauth_flow(&state, &provider)?;
    let token = flow.exchange_code(&params.code).await?;

    // Store the token as a credential.
    let cred = state
        .credentials
        .store_credential(
            &provider,
            ciab_core::types::credentials::CredentialType::OAuthToken,
            serde_json::to_string(&token)
                .map_err(|e| CiabError::Internal(e.to_string()))?
                .as_bytes(),
            {
                let mut labels = std::collections::HashMap::new();
                labels.insert("provider".to_string(), provider.clone());
                labels
            },
            token.expires_at,
        )
        .await?;

    // Also store in the oauth token table.
    state
        .credentials
        .store_oauth_token(&provider, &cred.id, &token)
        .await?;

    Ok(Json(serde_json::json!({
        "status": "authenticated",
        "credential_id": cred.id,
    })))
}

// ---------------------------------------------------------------------------
// device_code — start device code flow
// ---------------------------------------------------------------------------

pub async fn device_code(
    State(state): State<AppState>,
    Path(provider): Path<String>,
) -> Result<impl IntoResponse, CiabError> {
    let flow = get_oauth_flow(&state, &provider)?;
    let response = flow.device_code_request().await?;
    Ok(Json(response))
}

// ---------------------------------------------------------------------------
// device_poll — poll device code status
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct DevicePollRequest {
    pub device_code: String,
}

pub async fn device_poll(
    State(state): State<AppState>,
    Path(provider): Path<String>,
    Json(body): Json<DevicePollRequest>,
) -> Result<impl IntoResponse, CiabError> {
    let flow = get_oauth_flow(&state, &provider)?;
    let result = flow.device_code_poll(&body.device_code).await?;

    match result {
        OAuth2PollResult::Pending => Ok(Json(serde_json::json!({"status": "pending"}))),
        OAuth2PollResult::Complete(token) => {
            // Store the token.
            let cred = state
                .credentials
                .store_credential(
                    &provider,
                    ciab_core::types::credentials::CredentialType::OAuthToken,
                    serde_json::to_string(&token)
                        .map_err(|e| CiabError::Internal(e.to_string()))?
                        .as_bytes(),
                    {
                        let mut labels = std::collections::HashMap::new();
                        labels.insert("provider".to_string(), provider.clone());
                        labels
                    },
                    token.expires_at,
                )
                .await?;

            state
                .credentials
                .store_oauth_token(&provider, &cred.id, &token)
                .await?;

            Ok(Json(serde_json::json!({
                "status": "complete",
                "credential_id": cred.id,
            })))
        }
        OAuth2PollResult::Error(msg) => Err(CiabError::OAuthFlowFailed(msg)),
    }
}

// ---------------------------------------------------------------------------
// refresh_token — force refresh
// ---------------------------------------------------------------------------

pub async fn refresh_token(
    State(state): State<AppState>,
    Path(provider): Path<String>,
) -> Result<impl IntoResponse, CiabError> {
    let flow = get_oauth_flow(&state, &provider)?;

    let existing = state
        .credentials
        .get_oauth_token(&provider)
        .await?
        .ok_or_else(|| {
            CiabError::CredentialNotFound(format!("no OAuth token found for {}", provider))
        })?;

    let refresh = existing
        .refresh_token
        .as_deref()
        .ok_or(CiabError::OAuthFlowFailed(
            "no refresh token available".to_string(),
        ))?;

    let new_token = flow.refresh_token(refresh).await?;

    // Store the refreshed token.
    let cred = state
        .credentials
        .store_credential(
            &provider,
            ciab_core::types::credentials::CredentialType::OAuthToken,
            serde_json::to_string(&new_token)
                .map_err(|e| CiabError::Internal(e.to_string()))?
                .as_bytes(),
            {
                let mut labels = std::collections::HashMap::new();
                labels.insert("provider".to_string(), provider.clone());
                labels
            },
            new_token.expires_at,
        )
        .await?;

    state
        .credentials
        .store_oauth_token(&provider, &cred.id, &new_token)
        .await?;

    Ok(Json(serde_json::json!({
        "status": "refreshed",
        "credential_id": cred.id,
    })))
}
