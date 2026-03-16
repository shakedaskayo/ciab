use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

pub type CiabResult<T> = Result<T, CiabError>;

#[derive(Debug, thiserror::Error)]
pub enum CiabError {
    // Sandbox errors
    #[error("sandbox not found: {0}")]
    SandboxNotFound(String),
    #[error("sandbox already exists: {0}")]
    SandboxAlreadyExists(String),
    #[error("sandbox in invalid state: {current}, expected: {expected}")]
    SandboxInvalidState { current: String, expected: String },
    #[error("sandbox creation failed: {0}")]
    SandboxCreationFailed(String),
    #[error("sandbox timeout: {0}")]
    SandboxTimeout(String),

    // Agent errors
    #[error("agent provider not found: {0}")]
    AgentProviderNotFound(String),
    #[error("agent not running")]
    AgentNotRunning,
    #[error("agent communication error: {0}")]
    AgentCommunicationError(String),

    // Session errors
    #[error("session not found: {0}")]
    SessionNotFound(String),
    #[error("session in invalid state: {0}")]
    SessionInvalidState(String),

    // Workspace errors
    #[error("workspace not found: {0}")]
    WorkspaceNotFound(String),
    #[error("workspace already exists: {0}")]
    WorkspaceAlreadyExists(String),
    #[error("workspace validation error: {0}")]
    WorkspaceValidationError(String),

    // Template errors
    #[error("template source not found: {0}")]
    TemplateSourceNotFound(String),
    #[error("template sync failed: {0}")]
    TemplateSyncFailed(String),

    // Credential errors
    #[error("credential not found: {0}")]
    CredentialNotFound(String),
    #[error("decryption failed: {0}")]
    DecryptionFailed(String),
    #[error("OAuth flow failed: {0}")]
    OAuthFlowFailed(String),
    #[error("OAuth token expired")]
    OAuthTokenExpired,

    // Runtime errors
    #[error("runtime unavailable: {0}")]
    RuntimeUnavailable(String),
    #[error("OpenSandbox error: {0}")]
    OpenSandboxError(String),
    #[error("kubernetes error: {0}")]
    KubernetesError(String),
    #[error("kubernetes pod not found: {0}")]
    KubernetesPodNotFound(String),

    // Provisioning errors
    #[error("provisioning failed: {0}")]
    ProvisioningFailed(String),
    #[error("git clone failed: {0}")]
    GitCloneFailed(String),
    #[error("script execution failed: {0}")]
    ScriptExecutionFailed(String),
    #[error("local mount failed: {0}")]
    LocalMountFailed(String),
    #[error("git worktree failed: {0}")]
    GitWorktreeFailed(String),
    #[error("agentfs error: {0}")]
    AgentFsError(String),

    // Stream errors
    #[error("stream buffer overflow")]
    StreamBufferOverflow,
    #[error("stream connection lost")]
    StreamConnectionLost,

    // IO errors
    #[error("file not found: {0}")]
    FileNotFound(String),
    #[error("exec failed: {0}")]
    ExecFailed(String),

    // Config errors
    #[error("configuration error: {0}")]
    ConfigError(String),
    #[error("configuration validation error: {0}")]
    ConfigValidationError(String),

    // Auth errors
    #[error("unauthorized: {0}")]
    Unauthorized(String),
    #[error("forbidden: {0}")]
    Forbidden(String),
    #[error("rate limited")]
    RateLimited,

    // Gateway errors
    #[error("tunnel not found: {0}")]
    TunnelNotFound(String),
    #[error("tunnel creation failed: {0}")]
    TunnelCreationFailed(String),
    #[error("client token not found: {0}")]
    ClientTokenNotFound(String),
    #[error("client token expired")]
    ClientTokenExpired,
    #[error("client token revoked")]
    ClientTokenRevoked,
    #[error("insufficient scope: {0}")]
    InsufficientScope(String),
    #[error("gateway not enabled")]
    GatewayNotEnabled,
    #[error("FRP error: {0}")]
    FrpError(String),
    #[error("tunnel provider error: {0}")]
    TunnelProviderError(String),
    #[error("tunnel provider not ready: {0}")]
    TunnelProviderNotReady(String),

    // Channel errors
    #[error("channel not found: {0}")]
    ChannelNotFound(String),
    #[error("channel adapter error: {0}")]
    ChannelAdapterError(String),
    #[error("channel sender not allowed: {0}")]
    ChannelSenderNotAllowed(String),

    // Generic errors
    #[error("internal error: {0}")]
    Internal(String),
    #[error("external error: {0}")]
    External(String),
    #[error("timeout: {0}")]
    Timeout(String),

    // Wrapping
    #[error("database error: {0}")]
    Database(String),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

impl CiabError {
    pub fn status_code(&self) -> u16 {
        match self {
            Self::SandboxNotFound(_)
            | Self::SessionNotFound(_)
            | Self::WorkspaceNotFound(_)
            | Self::CredentialNotFound(_)
            | Self::FileNotFound(_)
            | Self::AgentProviderNotFound(_)
            | Self::TemplateSourceNotFound(_)
            | Self::TunnelNotFound(_)
            | Self::ClientTokenNotFound(_)
            | Self::ChannelNotFound(_) => 404,
            Self::SandboxAlreadyExists(_) | Self::WorkspaceAlreadyExists(_) => 409,
            Self::SandboxInvalidState { .. } | Self::SessionInvalidState(_) => 409,
            Self::Unauthorized(_) => 401,
            Self::Forbidden(_)
            | Self::ClientTokenExpired
            | Self::ClientTokenRevoked
            | Self::InsufficientScope(_)
            | Self::ChannelSenderNotAllowed(_) => 403,
            Self::GatewayNotEnabled | Self::TunnelProviderNotReady(_) => 503,
            Self::RateLimited => 429,
            Self::ConfigError(_)
            | Self::ConfigValidationError(_)
            | Self::WorkspaceValidationError(_)
            | Self::AgentCommunicationError(_)
            | Self::TunnelProviderError(_) => 400,
            Self::SandboxTimeout(_) | Self::Timeout(_) => 504,
            _ => 500,
        }
    }

    pub fn error_code(&self) -> &str {
        match self {
            Self::SandboxNotFound(_) => "sandbox_not_found",
            Self::SandboxAlreadyExists(_) => "sandbox_already_exists",
            Self::SandboxInvalidState { .. } => "sandbox_invalid_state",
            Self::SandboxCreationFailed(_) => "sandbox_creation_failed",
            Self::SandboxTimeout(_) => "sandbox_timeout",
            Self::AgentProviderNotFound(_) => "agent_provider_not_found",
            Self::AgentNotRunning => "agent_not_running",
            Self::AgentCommunicationError(_) => "agent_communication_error",
            Self::SessionNotFound(_) => "session_not_found",
            Self::SessionInvalidState(_) => "session_invalid_state",
            Self::WorkspaceNotFound(_) => "workspace_not_found",
            Self::WorkspaceAlreadyExists(_) => "workspace_already_exists",
            Self::WorkspaceValidationError(_) => "workspace_validation_error",
            Self::TemplateSourceNotFound(_) => "template_source_not_found",
            Self::TemplateSyncFailed(_) => "template_sync_failed",
            Self::CredentialNotFound(_) => "credential_not_found",
            Self::DecryptionFailed(_) => "decryption_failed",
            Self::OAuthFlowFailed(_) => "oauth_flow_failed",
            Self::OAuthTokenExpired => "oauth_token_expired",
            Self::RuntimeUnavailable(_) => "runtime_unavailable",
            Self::OpenSandboxError(_) => "opensandbox_error",
            Self::KubernetesError(_) => "kubernetes_error",
            Self::KubernetesPodNotFound(_) => "kubernetes_pod_not_found",
            Self::ProvisioningFailed(_) => "provisioning_failed",
            Self::GitCloneFailed(_) => "git_clone_failed",
            Self::ScriptExecutionFailed(_) => "script_execution_failed",
            Self::LocalMountFailed(_) => "local_mount_failed",
            Self::GitWorktreeFailed(_) => "git_worktree_failed",
            Self::AgentFsError(_) => "agentfs_error",
            Self::StreamBufferOverflow => "stream_buffer_overflow",
            Self::StreamConnectionLost => "stream_connection_lost",
            Self::FileNotFound(_) => "file_not_found",
            Self::ExecFailed(_) => "exec_failed",
            Self::ConfigError(_) => "config_error",
            Self::ConfigValidationError(_) => "config_validation_error",
            Self::Unauthorized(_) => "unauthorized",
            Self::Forbidden(_) => "forbidden",
            Self::RateLimited => "rate_limited",
            Self::TunnelNotFound(_) => "tunnel_not_found",
            Self::TunnelCreationFailed(_) => "tunnel_creation_failed",
            Self::ClientTokenNotFound(_) => "client_token_not_found",
            Self::ClientTokenExpired => "client_token_expired",
            Self::ClientTokenRevoked => "client_token_revoked",
            Self::InsufficientScope(_) => "insufficient_scope",
            Self::GatewayNotEnabled => "gateway_not_enabled",
            Self::FrpError(_) => "frp_error",
            Self::TunnelProviderError(_) => "tunnel_provider_error",
            Self::TunnelProviderNotReady(_) => "tunnel_provider_not_ready",
            Self::ChannelNotFound(_) => "channel_not_found",
            Self::ChannelAdapterError(_) => "channel_adapter_error",
            Self::ChannelSenderNotAllowed(_) => "channel_sender_not_allowed",
            Self::Internal(_) => "internal_error",
            Self::External(_) => "external_error",
            Self::Timeout(_) => "timeout",
            Self::Database(_) => "database_error",
            Self::Serialization(_) => "serialization_error",
        }
    }
}

impl IntoResponse for CiabError {
    fn into_response(self) -> Response {
        let status =
            StatusCode::from_u16(self.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        let body = json!({
            "error": {
                "code": self.error_code(),
                "message": self.to_string(),
            }
        });
        (status, axum::Json(body)).into_response()
    }
}
