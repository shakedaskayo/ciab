//! Claude Code hooks HTTP endpoint.
//!
//! When CIAB starts a Claude Code process, it configures HTTP hooks that POST
//! to this endpoint at various lifecycle points (PreToolUse, PostToolUse, Stop, etc.).
//!
//! The hook receives a JSON body (Claude Code POSTs it for HTTP hooks)
//! and returns a JSON response that Claude Code interprets as the hook result.

use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::Json;
use chrono::Utc;
use ciab_core::error::CiabError;
use ciab_core::types::stream::{StreamEvent, StreamEventType};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::state::AppState;

/// The JSON body Claude Code sends to an HTTP hook.
/// The shape depends on the hook event type but shares common fields.
#[derive(Debug, Deserialize)]
pub struct HookRequest {
    /// The hook event name: "PreToolUse", "PostToolUse", "Stop", etc.
    #[serde(default)]
    pub hook_event_name: String,

    /// Session ID from Claude Code
    #[serde(default)]
    pub session_id: Option<String>,

    /// Tool name (for PreToolUse / PostToolUse)
    #[serde(default)]
    pub tool_name: Option<String>,

    /// Tool input (for PreToolUse)
    #[serde(default)]
    pub tool_input: Option<serde_json::Value>,

    /// Tool result output (for PostToolUse)
    #[serde(default)]
    pub tool_output: Option<serde_json::Value>,

    /// Stop reason (for Stop hook)
    #[serde(default)]
    pub stop_reason: Option<String>,

    /// Full raw hook payload — kept for forward compatibility
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// Response from CIAB's hook endpoint.
#[derive(Debug, Serialize)]
pub struct HookResponse {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

/// POST /api/v1/hooks/claude/{session_id}
///
/// Claude Code's HTTP hook endpoint. Claude Code posts hook events here.
/// CIAB checks the session's permission policy and returns the appropriate decision.
pub async fn claude_hook(
    State(state): State<AppState>,
    Path(ciab_session_id): Path<Uuid>,
    Json(body): Json<HookRequest>,
) -> Result<impl IntoResponse, CiabError> {
    let event_name = body.hook_event_name.as_str();

    tracing::debug!(
        session_id = %ciab_session_id,
        hook_event = %event_name,
        tool = ?body.tool_name,
        "Claude hook received"
    );

    match event_name {
        "PreToolUse" => handle_pre_tool_use(&state, ciab_session_id, &body).await,
        "PostToolUse" => handle_post_tool_use(&state, ciab_session_id, &body).await,
        "Stop" => handle_stop(&state, ciab_session_id, &body).await,
        _ => {
            // Unknown hook event — log and return empty response (no-op)
            tracing::debug!(hook_event = %event_name, "Unhandled hook event, returning empty response");
            Ok(Json(HookResponse { data: json!({}) }))
        }
    }
}

/// Handle PreToolUse hook: check permission policy and return allow/deny/ask.
///
/// Claude Code will:
/// - "allow": proceed without prompting
/// - "deny": skip the tool call
/// - "ask": show the normal permission prompt (falls through to control_request/stdin)
async fn handle_pre_tool_use(
    state: &AppState,
    ciab_session_id: Uuid,
    body: &HookRequest,
) -> Result<Json<HookResponse>, CiabError> {
    let tool_name = body.tool_name.as_deref().unwrap_or("");

    // Look up the session's permission policy
    let policy = {
        let perms = state.session_permissions.read().await;
        perms.get(&ciab_session_id).cloned().unwrap_or_default()
    };

    // Determine the decision based on the policy
    let decision = if !policy.requires_approval(tool_name) {
        // Policy says auto-approve — tell Claude Code to allow
        "allow"
    } else {
        // Policy says needs approval — tell Claude Code to "ask",
        // which will trigger the normal control_request → stdin → frontend flow
        "ask"
    };

    // Emit a hook event to the SSE stream for observability
    if let Ok(Some(session)) = state.db.get_session(&ciab_session_id).await {
        let _ = state
            .stream_handler
            .publish(StreamEvent {
                id: Uuid::new_v4().to_string(),
                sandbox_id: session.sandbox_id,
                session_id: Some(ciab_session_id),
                event_type: StreamEventType::LogLine,
                data: json!({
                    "hook": "PreToolUse",
                    "tool_name": tool_name,
                    "decision": decision,
                }),
                timestamp: Utc::now(),
            })
            .await;
    }

    // Claude Code PreToolUse hooks expect `permissionDecision` field in the response.
    // Valid values: "allow" (auto-approve), "deny" (block), "ask" (show permission prompt).
    Ok(Json(HookResponse {
        data: json!({
            "permissionDecision": decision,
        }),
    }))
}

/// Handle PostToolUse hook: emit tool completion and file change events.
///
/// For file-modifying tools (Edit, Write, NotebookEdit, MultiEdit, Bash),
/// we emit a `FileChanged` event so the frontend can show live file activity.
async fn handle_post_tool_use(
    state: &AppState,
    ciab_session_id: Uuid,
    body: &HookRequest,
) -> Result<Json<HookResponse>, CiabError> {
    let tool_name = body.tool_name.as_deref().unwrap_or("");

    if let Ok(Some(session)) = state.db.get_session(&ciab_session_id).await {
        let sandbox_id = session.sandbox_id;

        // Emit tool completion event
        let _ = state
            .stream_handler
            .publish(StreamEvent {
                id: Uuid::new_v4().to_string(),
                sandbox_id,
                session_id: Some(ciab_session_id),
                event_type: StreamEventType::ToolProgress,
                data: json!({
                    "hook": "PostToolUse",
                    "tool_name": tool_name,
                    "completed": true,
                }),
                timestamp: Utc::now(),
            })
            .await;

        // For file-related tools, emit a FileChanged event with file details.
        // This powers the live file activity panel in the chat UI.
        let file_path = extract_file_path(tool_name, body);
        if let Some(path) = file_path {
            let action = match tool_name {
                "Edit" | "MultiEdit" => "edited",
                "Write" | "NotebookEdit" => "written",
                "Read" => "read",
                "Bash" => "executed",
                "Grep" => "searched",
                "Glob" => "listed",
                _ => "accessed",
            };

            let _ = state
                .stream_handler
                .publish(StreamEvent {
                    id: Uuid::new_v4().to_string(),
                    sandbox_id,
                    session_id: Some(ciab_session_id),
                    event_type: StreamEventType::FileChanged,
                    data: json!({
                        "tool_name": tool_name,
                        "file_path": path,
                        "action": action,
                    }),
                    timestamp: Utc::now(),
                })
                .await;
        }
    }

    Ok(Json(HookResponse { data: json!({}) }))
}

/// Handle Stop hook: agent has finished responding.
async fn handle_stop(
    state: &AppState,
    ciab_session_id: Uuid,
    body: &HookRequest,
) -> Result<Json<HookResponse>, CiabError> {
    let stop_reason = body.stop_reason.as_deref().unwrap_or("unknown");

    if let Ok(Some(session)) = state.db.get_session(&ciab_session_id).await {
        let _ = state
            .stream_handler
            .publish(StreamEvent {
                id: Uuid::new_v4().to_string(),
                sandbox_id: session.sandbox_id,
                session_id: Some(ciab_session_id),
                event_type: StreamEventType::LogLine,
                data: json!({
                    "hook": "Stop",
                    "stop_reason": stop_reason,
                }),
                timestamp: Utc::now(),
            })
            .await;
    }

    Ok(Json(HookResponse { data: json!({}) }))
}

/// Extract the file path from a tool's input based on the tool type.
fn extract_file_path(tool_name: &str, body: &HookRequest) -> Option<String> {
    let input = body.tool_input.as_ref()?;

    match tool_name {
        "Edit" | "MultiEdit" | "Write" | "NotebookEdit" | "Read" => {
            // These tools have a "file_path" or "path" field in their input
            input
                .get("file_path")
                .or_else(|| input.get("path"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        }
        "Bash" => {
            // Extract the command for display
            input.get("command").and_then(|v| v.as_str()).map(|cmd| {
                // Truncate long commands
                if cmd.len() > 80 {
                    format!("{}...", &cmd[..77])
                } else {
                    cmd.to_string()
                }
            })
        }
        "Grep" => input
            .get("pattern")
            .and_then(|v| v.as_str())
            .map(|p| format!("grep: {}", p)),
        "Glob" => input
            .get("pattern")
            .and_then(|v| v.as_str())
            .map(|p| format!("glob: {}", p)),
        _ => None,
    }
}
