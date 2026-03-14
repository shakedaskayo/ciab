//! Permission gate for tool execution in streaming sessions.
//!
//! This module encapsulates the logic for intercepting `ToolUseStart` events,
//! checking if user approval is required, and pausing the stream until the
//! user responds (or timeout).

use std::sync::Arc;

use chrono::Utc;
use ciab_core::traits::runtime::SandboxRuntime;
use ciab_core::traits::stream::StreamHandler;
use ciab_core::types::agent::PermissionPolicy;
use ciab_core::types::session::SessionState;
use ciab_core::types::stream::{StreamEvent, StreamEventType};
use serde_json::json;
use uuid::Uuid;

use crate::state::{AppState, PendingPermission};

/// Result of a permission gate check on a tool use event.
pub enum PermissionGateResult {
    /// Tool is allowed — continue publishing the event.
    Allowed,
    /// Tool was denied — the agent should be interrupted.
    Denied,
    /// No approval needed (auto-approve mode or tool not gated).
    NotRequired,
}

/// Check whether a `ToolUseStart` event requires permission, and if so,
/// pause the stream and wait for user response.
///
/// Returns `PermissionGateResult` indicating how the caller should proceed.
pub async fn check_tool_permission(
    state: &AppState,
    policy: &PermissionPolicy,
    stream_handler: &Arc<dyn StreamHandler>,
    runtime: &Arc<dyn SandboxRuntime>,
    sandbox_id: Uuid,
    session_id: Uuid,
    event: &StreamEvent,
) -> PermissionGateResult {
    let tool_name = event
        .data
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if !policy.requires_approval(tool_name) {
        return PermissionGateResult::NotRequired;
    }

    let request_id = Uuid::new_v4();
    let risk = PermissionPolicy::risk_level(tool_name);

    // Emit permission_request SSE event.
    let perm_event = StreamEvent {
        id: Uuid::new_v4().to_string(),
        sandbox_id,
        session_id: Some(session_id),
        event_type: StreamEventType::PermissionRequest,
        data: json!({
            "request_id": request_id,
            "tool_name": tool_name,
            "tool_input": event.data.get("input"),
            "risk_level": risk,
        }),
        timestamp: Utc::now(),
    };
    let _ = stream_handler.publish(perm_event).await;

    // Update session state to WaitingForInput.
    let _ = state
        .db
        .update_session_state(&session_id, &SessionState::WaitingForInput)
        .await;

    // Create oneshot channel and store in pending_permissions.
    let (tx, rx) = tokio::sync::oneshot::channel::<bool>();
    {
        let mut pending = state.pending_permissions.write().await;
        pending.insert(request_id.to_string(), PendingPermission { tx });
    }

    // Await user response with 5-minute timeout.
    let approved = tokio::time::timeout(std::time::Duration::from_secs(300), rx)
        .await
        .unwrap_or(Ok(false)) // timeout → deny
        .unwrap_or(false); // channel closed → deny

    // Restore session state to Processing.
    let _ = state
        .db
        .update_session_state(&session_id, &SessionState::Processing)
        .await;

    // Emit permission_response SSE event.
    let resp_event = StreamEvent {
        id: Uuid::new_v4().to_string(),
        sandbox_id,
        session_id: Some(session_id),
        event_type: StreamEventType::PermissionResponse,
        data: json!({
            "request_id": request_id,
            "tool_name": tool_name,
            "approved": approved,
        }),
        timestamp: Utc::now(),
    };
    let _ = stream_handler.publish(resp_event).await;

    if approved {
        PermissionGateResult::Allowed
    } else {
        // Kill the active exec process.
        let _ = runtime.kill_exec(&sandbox_id).await;
        PermissionGateResult::Denied
    }
}
