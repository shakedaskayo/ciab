use std::collections::HashMap;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use chrono::{DateTime, Utc};
use ciab_core::error::CiabError;
use ciab_core::types::credentials::CredentialType;
use serde::Deserialize;
use uuid::Uuid;

use crate::state::AppState;

// ---------------------------------------------------------------------------
// create_credential
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateCredentialRequest {
    pub name: String,
    #[serde(default = "default_cred_type")]
    pub credential_type: CredentialType,
    /// The secret value to store (will be encrypted).
    pub value: String,
    #[serde(default)]
    pub labels: HashMap<String, String>,
    #[serde(default)]
    pub expires_at: Option<DateTime<Utc>>,
}

fn default_cred_type() -> CredentialType {
    CredentialType::ApiKey
}

pub async fn create_credential(
    State(state): State<AppState>,
    Json(body): Json<CreateCredentialRequest>,
) -> Result<impl IntoResponse, CiabError> {
    let cred = state
        .credentials
        .store_credential(
            &body.name,
            body.credential_type,
            body.value.as_bytes(),
            body.labels,
            body.expires_at,
        )
        .await?;
    Ok((StatusCode::CREATED, Json(cred)))
}

// ---------------------------------------------------------------------------
// list_credentials
// ---------------------------------------------------------------------------

pub async fn list_credentials(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, CiabError> {
    let credentials = state.credentials.list_credentials().await?;
    Ok(Json(credentials))
}

// ---------------------------------------------------------------------------
// get_credential (metadata only, no secret)
// ---------------------------------------------------------------------------

pub async fn get_credential(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, CiabError> {
    let (cred, _data) = state.credentials.get_credential(&id).await?;
    Ok(Json(cred))
}

// ---------------------------------------------------------------------------
// delete_credential
// ---------------------------------------------------------------------------

pub async fn delete_credential(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, CiabError> {
    state.credentials.delete_credential(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}
