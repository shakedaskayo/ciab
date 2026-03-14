use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamEvent {
    pub id: String,
    pub sandbox_id: Uuid,
    #[serde(default)]
    pub session_id: Option<Uuid>,
    pub event_type: StreamEventType,
    pub data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StreamEventType {
    Connected,
    Reconnect,
    Keepalive,
    TextDelta,
    TextComplete,
    ThinkingDelta,
    SubagentStart,
    SubagentEnd,
    ToolUseStart,
    ToolInputDelta,
    ToolUseComplete,
    ToolResult,
    SandboxStateChanged,
    ProvisioningStep,
    ProvisioningComplete,
    ProvisioningFailed,
    SessionCreated,
    SessionCompleted,
    SessionFailed,
    PermissionRequest,
    PermissionResponse,
    Error,
    Stats,
    LogLine,
    UserInputRequest,
    ToolProgress,
    ResultError,
    QueueUpdated,
    FileChanged,
}
