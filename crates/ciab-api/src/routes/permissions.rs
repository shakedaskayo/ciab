//! Permission management endpoints for session-level tool approval.
//!
//! - `set_permission_mode`: Configure the permission policy for a session.
//! - `respond_to_permission`: Approve or deny a pending permission request.

use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::Json;
use ciab_core::error::CiabError;
use ciab_core::types::agent::{PermissionMode, PermissionPolicy};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::state::AppState;

// ---------------------------------------------------------------------------
// set_permission_mode
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct SetPermissionModeRequest {
    pub mode: PermissionMode,
    #[serde(default)]
    pub always_require_approval: Vec<String>,
    #[serde(default)]
    pub always_allow: Vec<String>,
}

pub async fn set_permission_mode(
    State(state): State<AppState>,
    Path(sid): Path<Uuid>,
    Json(body): Json<SetPermissionModeRequest>,
) -> Result<impl IntoResponse, CiabError> {
    // Verify session exists.
    let _session = state
        .db
        .get_session(&sid)
        .await?
        .ok_or_else(|| CiabError::SessionNotFound(sid.to_string()))?;

    let policy = {
        let mut perms = state.session_permissions.write().await;
        let existing = perms.entry(sid).or_insert_with(PermissionPolicy::default);

        // Always update the mode.
        existing.mode = body.mode;

        // Merge always_require_approval: if caller provides a non-empty list,
        // add to existing rather than replacing (avoids wiping prior entries).
        for tool in &body.always_require_approval {
            if !existing.always_require_approval.contains(tool) {
                existing.always_require_approval.push(tool.clone());
            }
        }

        // Merge always_allow: append new tools without duplicates.
        for tool in &body.always_allow {
            if !existing.always_allow.contains(tool) {
                existing.always_allow.push(tool.clone());
            }
        }

        existing.clone()
    };

    Ok(Json(json!({
        "status": "ok",
        "mode": policy.mode,
    })))
}

// ---------------------------------------------------------------------------
// respond_to_permission
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct PermissionResponseRequest {
    pub approved: bool,
}

pub async fn respond_to_permission(
    State(state): State<AppState>,
    Path((sid, request_id)): Path<(Uuid, String)>,
    Json(body): Json<PermissionResponseRequest>,
) -> Result<impl IntoResponse, CiabError> {
    // Verify session exists.
    let _session = state
        .db
        .get_session(&sid)
        .await?
        .ok_or_else(|| CiabError::SessionNotFound(sid.to_string()))?;

    // Find and resolve the pending permission.
    let pending = {
        let mut pending_map = state.pending_permissions.write().await;
        pending_map.remove(&request_id)
    };

    match pending {
        Some(p) => {
            // Send the user's response. Ignore error if receiver already dropped.
            let _ = p.tx.send(body.approved);
            Ok(Json(json!({
                "status": "ok",
                "approved": body.approved,
            })))
        }
        None => Err(CiabError::SessionInvalidState(format!(
            "no pending permission request with id {}",
            request_id
        ))),
    }
}

// ---------------------------------------------------------------------------
// respond_to_user_input
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct UserInputResponse {
    pub answer: String,
}

pub async fn respond_to_user_input(
    State(state): State<AppState>,
    Path((_sid, request_id)): Path<(Uuid, String)>,
    Json(body): Json<UserInputResponse>,
) -> Result<impl IntoResponse, CiabError> {
    let pending = {
        let mut pending_map = state.pending_user_inputs.write().await;
        pending_map.remove(&request_id)
    };

    match pending {
        Some(p) => {
            let _ = p.tx.send(body.answer);
            Ok(Json(json!({"status": "ok"})))
        }
        None => Err(CiabError::SessionInvalidState(format!(
            "no pending user input request with id {}",
            request_id
        ))),
    }
}
