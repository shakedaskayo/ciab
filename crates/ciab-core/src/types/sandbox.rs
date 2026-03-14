use std::collections::HashMap;
use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::agent::AgentConfig;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SandboxState {
    Pending,
    Creating,
    Running,
    Pausing,
    Paused,
    Stopping,
    Stopped,
    Terminated,
    Failed,
}

impl fmt::Display for SandboxState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Pending => "pending",
            Self::Creating => "creating",
            Self::Running => "running",
            Self::Pausing => "pausing",
            Self::Paused => "paused",
            Self::Stopping => "stopping",
            Self::Stopped => "stopped",
            Self::Terminated => "terminated",
            Self::Failed => "failed",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SandboxPersistence {
    Ephemeral,
    Persistent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxSpec {
    #[serde(default)]
    pub name: Option<String>,
    pub agent_provider: String,
    #[serde(default)]
    pub image: Option<String>,
    #[serde(default)]
    pub resource_limits: Option<ResourceLimits>,
    #[serde(default = "default_persistence")]
    pub persistence: SandboxPersistence,
    #[serde(default)]
    pub network: Option<NetworkSpec>,
    #[serde(default)]
    pub env_vars: HashMap<String, String>,
    #[serde(default)]
    pub volumes: Vec<VolumeMount>,
    #[serde(default)]
    pub ports: Vec<PortMapping>,
    #[serde(default)]
    pub git_repos: Vec<GitRepoSpec>,
    #[serde(default)]
    pub credentials: Vec<String>,
    #[serde(default)]
    pub provisioning_scripts: Vec<String>,
    #[serde(default)]
    pub labels: HashMap<String, String>,
    #[serde(default)]
    pub timeout_secs: Option<u32>,
    #[serde(default)]
    pub agent_config: Option<AgentConfig>,
    #[serde(default)]
    pub local_mounts: Vec<LocalMountSpec>,
    /// Override runtime backend for this sandbox (from workspace config)
    #[serde(default)]
    pub runtime_backend: Option<String>,
}

fn default_persistence() -> SandboxPersistence {
    SandboxPersistence::Ephemeral
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxInfo {
    pub id: Uuid,
    #[serde(default)]
    pub name: Option<String>,
    pub state: SandboxState,
    pub persistence: SandboxPersistence,
    pub agent_provider: String,
    #[serde(default)]
    pub endpoint_url: Option<String>,
    #[serde(default)]
    pub resource_stats: Option<ResourceStats>,
    #[serde(default)]
    pub labels: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub spec: SandboxSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub cpu_cores: f32,
    pub memory_mb: u32,
    pub disk_mb: u32,
    #[serde(default)]
    pub max_processes: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceStats {
    pub cpu_usage_percent: f32,
    pub memory_used_mb: u32,
    pub memory_limit_mb: u32,
    pub disk_used_mb: u32,
    pub disk_limit_mb: u32,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSpec {
    pub enabled: bool,
    #[serde(default)]
    pub allowed_hosts: Vec<String>,
    #[serde(default)]
    pub dns_servers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecRequest {
    pub command: Vec<String>,
    #[serde(default)]
    pub workdir: Option<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub stdin: Option<String>,
    #[serde(default)]
    pub timeout_secs: Option<u32>,
    #[serde(default)]
    pub tty: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
    pub size: u64,
    pub is_dir: bool,
    pub mode: u32,
    #[serde(default)]
    pub modified_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalMountSpec {
    pub source: String,
    pub dest_path: String,
    pub sync_mode: String,
    pub exclude_patterns: Vec<String>,
    pub writeback: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitRepoSpec {
    pub url: String,
    #[serde(default)]
    pub branch: Option<String>,
    #[serde(default)]
    pub tag: Option<String>,
    #[serde(default)]
    pub commit: Option<String>,
    #[serde(default)]
    pub dest_path: Option<String>,
    #[serde(default)]
    pub credential_id: Option<String>,
    #[serde(default)]
    pub depth: Option<u32>,
    #[serde(default)]
    pub sparse_paths: Vec<String>,
    #[serde(default)]
    pub submodules: bool,
    /// Clone strategy: "clone" or "worktree"
    #[serde(default)]
    pub strategy: Option<String>,
    /// Host path to shared bare clone (for worktree strategy)
    #[serde(default)]
    pub worktree_base_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeMount {
    pub source: String,
    pub dest: String,
    #[serde(default)]
    pub read_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortMapping {
    pub host_port: u16,
    pub container_port: u16,
    #[serde(default = "default_protocol")]
    pub protocol: String,
}

fn default_protocol() -> String {
    "tcp".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogOptions {
    #[serde(default)]
    pub follow: bool,
    #[serde(default)]
    pub tail: Option<u32>,
    #[serde(default)]
    pub since: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxFilters {
    #[serde(default)]
    pub state: Option<SandboxState>,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub labels: HashMap<String, String>,
}
