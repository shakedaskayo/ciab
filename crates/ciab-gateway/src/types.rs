use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A tunnel exposing a local port to a public URL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayTunnel {
    pub id: Uuid,
    pub sandbox_id: Option<Uuid>,
    pub tunnel_type: TunnelType,
    pub public_url: String,
    pub local_port: u16,
    pub state: TunnelState,
    pub config_json: serde_json::Value,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TunnelType {
    Frp,
    Bore,
    Cloudflare,
    Ngrok,
    Lan,
    Manual,
}

impl std::fmt::Display for TunnelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Frp => write!(f, "frp"),
            Self::Bore => write!(f, "bore"),
            Self::Cloudflare => write!(f, "cloudflare"),
            Self::Ngrok => write!(f, "ngrok"),
            Self::Lan => write!(f, "lan"),
            Self::Manual => write!(f, "manual"),
        }
    }
}

impl std::str::FromStr for TunnelType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "frp" => Ok(Self::Frp),
            "bore" => Ok(Self::Bore),
            "cloudflare" => Ok(Self::Cloudflare),
            "ngrok" => Ok(Self::Ngrok),
            "lan" => Ok(Self::Lan),
            "manual" => Ok(Self::Manual),
            other => Err(format!("unknown tunnel type: {}", other)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TunnelState {
    Active,
    Stopped,
    Error,
}

impl std::fmt::Display for TunnelState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Stopped => write!(f, "stopped"),
            Self::Error => write!(f, "error"),
        }
    }
}

impl std::str::FromStr for TunnelState {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "active" => Ok(Self::Active),
            "stopped" => Ok(Self::Stopped),
            "error" => Ok(Self::Error),
            other => Err(format!("unknown tunnel state: {}", other)),
        }
    }
}

/// A scoped client token for gateway access.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientToken {
    pub id: Uuid,
    pub name: String,
    /// SHA-256 hash of the raw token (raw token never stored).
    pub token_hash: String,
    pub scopes: Vec<TokenScope>,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TokenScope {
    FullAccess,
    SandboxAccess { sandbox_id: Uuid },
    WorkspaceAccess { workspace_id: Uuid },
    ReadOnly,
    ChatOnly { sandbox_id: Uuid },
}

impl TokenScope {
    /// Check if this scope allows access to a given sandbox.
    pub fn allows_sandbox(&self, sandbox_id: &Uuid) -> bool {
        match self {
            Self::FullAccess => true,
            Self::SandboxAccess { sandbox_id: sid } => sid == sandbox_id,
            Self::ChatOnly { sandbox_id: sid } => sid == sandbox_id,
            Self::ReadOnly => true,
            Self::WorkspaceAccess { .. } => {
                // Workspace scope requires external lookup; default to false here.
                // The caller must resolve workspace membership.
                false
            }
        }
    }

    /// Check if this scope permits write operations.
    pub fn allows_write(&self) -> bool {
        match self {
            Self::FullAccess => true,
            Self::SandboxAccess { .. } => true,
            Self::WorkspaceAccess { .. } => true,
            Self::ChatOnly { .. } => true,
            Self::ReadOnly => false,
        }
    }
}

/// Overall gateway status returned by the status endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayStatus {
    pub enabled: bool,
    /// Currently active tunnel provider name
    pub active_provider: String,
    pub lan: LanStatus,
    pub providers: Vec<TunnelProviderInfo>,
    pub active_tunnels: usize,
    pub active_tokens: usize,
    // Legacy fields for backward compat
    pub frp: FrpStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanStatus {
    pub enabled: bool,
    pub mdns_name: Option<String>,
    pub local_addresses: Vec<String>,
    pub advertise_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrpStatus {
    pub enabled: bool,
    pub process_running: bool,
    pub server_addr: Option<String>,
    pub proxy_count: usize,
}

/// Status info for a tunnel provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelProviderInfo {
    /// Provider name: "frp", "bore", "cloudflare", "ngrok"
    pub name: String,
    pub enabled: bool,
    pub installed: bool,
    pub binary_path: Option<String>,
    pub version: Option<String>,
    pub process_running: bool,
    pub tunnel_count: usize,
}

/// Result from preparing (installing/validating) a tunnel provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderPrepareResult {
    pub provider: String,
    pub installed: bool,
    pub binary_path: String,
    pub version: Option<String>,
    pub message: String,
}
