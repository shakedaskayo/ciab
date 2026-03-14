use axum::extract::{FromRef, FromRequestParts, Request, State};
use axum::http::request::Parts;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use ciab_core::error::CiabError;
use ciab_gateway::types::TokenScope;
use uuid::Uuid;

use crate::state::AppState;

/// Token context inserted into request extensions after successful auth.
#[derive(Clone, Debug)]
pub struct TokenContext {
    pub token_id: Option<Uuid>,
    pub scopes: Vec<TokenScope>,
    pub is_admin: bool,
}

/// API key authentication extractor.
///
/// Can be used as an axum extractor in individual handlers, or via
/// the [`auth_middleware`] function as a tower layer on a router.
///
/// Checks credentials in this order:
/// 1. `Authorization: Bearer <key>` header
/// 2. `X-API-Key` header
/// 3. `?token=<key>` query parameter (for SSE / EventSource which cannot set headers)
///
/// If no API keys are configured (empty list) and no gateway tokens exist, all requests
/// are allowed (dev mode).
///
/// When a scoped client token is used, a [`TokenContext`] is inserted into the request
/// extensions so downstream handlers can enforce per-sandbox/workspace scopes.
pub struct ApiKeyAuth;

impl<S> FromRequestParts<S> for ApiKeyAuth
where
    S: Send + Sync,
    AppState: axum::extract::FromRef<S>,
{
    type Rejection = CiabError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);
        validate_api_key(parts, &app_state).await
    }
}

/// Axum middleware function for API key authentication.
///
/// Apply to a router via:
/// ```ignore
/// router.layer(axum::middleware::from_fn_with_state(state.clone(), auth_middleware))
/// ```
pub async fn auth_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let (mut parts, body) = request.into_parts();

    if let Err(err) = validate_api_key(&mut parts, &state).await {
        return err.into_response();
    }

    let request = Request::from_parts(parts, body);
    next.run(request).await
}

/// Extract the raw bearer/api-key token from the request.
fn extract_token(parts: &Parts) -> Option<String> {
    // 1. Try Authorization: Bearer <key>
    if let Some(auth_header) = parts.headers.get("authorization") {
        if let Ok(value) = auth_header.to_str() {
            if let Some(token) = value.strip_prefix("Bearer ") {
                return Some(token.trim().to_string());
            }
        }
    }

    // 2. Try X-API-Key header
    if let Some(api_key_header) = parts.headers.get("x-api-key") {
        if let Ok(value) = api_key_header.to_str() {
            return Some(value.trim().to_string());
        }
    }

    // 3. Try ?token=<key> query parameter (for EventSource/SSE clients)
    if let Some(query) = parts.uri.query() {
        for pair in query.split('&') {
            if let Some(token) = pair.strip_prefix("token=") {
                return Some(token.trim().to_string());
            }
        }
    }

    None
}

/// Shared validation logic used by both the extractor and the middleware.
async fn validate_api_key(parts: &mut Parts, state: &AppState) -> Result<ApiKeyAuth, CiabError> {
    let api_keys = &state.config.security.api_keys;

    // Dev mode: if no API keys are configured, allow all requests (admin).
    if api_keys.is_empty() {
        parts.extensions.insert(TokenContext {
            token_id: None,
            scopes: vec![TokenScope::FullAccess],
            is_admin: true,
        });
        return Ok(ApiKeyAuth);
    }

    let token_str = match extract_token(parts) {
        Some(t) => t,
        None => {
            return Err(CiabError::Unauthorized(
                "missing or invalid API key".to_string(),
            ))
        }
    };

    // 1. Check flat admin API keys first.
    if api_keys.iter().any(|k| k == &token_str) {
        parts.extensions.insert(TokenContext {
            token_id: None,
            scopes: vec![TokenScope::FullAccess],
            is_admin: true,
        });
        return Ok(ApiKeyAuth);
    }

    // 2. Check scoped client tokens (if gateway is available).
    let gateway_guard = state.gateway.read().await;
    if let Some(ref gateway) = *gateway_guard {
        match gateway.validate_token(&token_str).await {
            Ok(client_token) => {
                parts.extensions.insert(TokenContext {
                    token_id: Some(client_token.id),
                    scopes: client_token.scopes,
                    is_admin: false,
                });
                return Ok(ApiKeyAuth);
            }
            Err(CiabError::ClientTokenExpired) => return Err(CiabError::ClientTokenExpired),
            Err(CiabError::ClientTokenRevoked) => return Err(CiabError::ClientTokenRevoked),
            Err(_) => {
                // Token not found in gateway, fall through to unauthorized
            }
        }
    }

    Err(CiabError::Unauthorized(
        "missing or invalid API key".to_string(),
    ))
}
