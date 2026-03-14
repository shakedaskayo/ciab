use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::agent::AgentConfig;
use super::sandbox::{GitRepoSpec, NetworkSpec, PortMapping, ResourceLimits, VolumeMount};

// ---------------------------------------------------------------------------
// Per-workspace runtime backend
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeBackend {
    /// Inherit from server config.toml
    #[default]
    Default,
    /// Run agents as local processes
    Local,
    /// Run agents in OpenSandbox containers
    OpenSandbox,
    /// Run agents in Docker containers
    Docker,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceRuntimeConfig {
    /// Which runtime backend to use (default = inherit from server config)
    #[serde(default)]
    pub backend: RuntimeBackend,
    /// Override working directory for local backend
    #[serde(default)]
    pub local_workdir: Option<String>,
}

impl Default for WorkspaceRuntimeConfig {
    fn default() -> Self {
        Self {
            backend: RuntimeBackend::Default,
            local_workdir: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Git clone strategy
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GitCloneStrategy {
    /// Full git clone (default)
    #[default]
    Clone,
    /// Lightweight worktree from a shared bare clone
    Worktree,
}

// ---------------------------------------------------------------------------
// AgentFS config
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentFsConfig {
    /// Whether AgentFS CoW isolation is enabled
    #[serde(default)]
    pub enabled: bool,
    /// Path to the agentfs binary
    #[serde(default = "default_agentfs_binary")]
    pub binary: String,
    /// Path to the AgentFS SQLite database
    #[serde(default)]
    pub db_path: Option<String>,
    /// Whether to log filesystem operations
    #[serde(default = "default_true")]
    pub operation_logging: bool,
}

fn default_agentfs_binary() -> String {
    "agentfs".to_string()
}

impl Default for AgentFsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            binary: default_agentfs_binary(),
            db_path: None,
            operation_logging: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalMount {
    /// Absolute path on the host machine
    pub source: String,
    /// Destination path inside the sandbox (default: /workspace/<dir-name>)
    #[serde(default)]
    pub dest_path: Option<String>,
    /// Sync mode: "copy" (isolated copy), "link" (symlink), "bind" (bind mount for docker)
    #[serde(default)]
    pub sync_mode: SyncMode,
    /// Glob patterns to exclude from copy
    #[serde(default)]
    pub exclude_patterns: Vec<String>,
    /// Whether to sync changes back to source on sandbox stop (only for "copy" mode)
    #[serde(default)]
    pub writeback: bool,
    /// Watch for changes and auto-sync (bidirectional)
    #[serde(default)]
    pub watch: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SyncMode {
    #[default]
    Copy,
    Link,
    Bind,
}

/// A workspace is a reusable, composable environment definition.
/// It groups repos, skills, credentials, filesystem settings, prompts,
/// and agent configuration into a single deployable unit.
/// Can be defined as TOML for CI/single-run usage or managed via API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: Uuid,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub spec: WorkspaceSpec,
    #[serde(default)]
    pub labels: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// The full specification of a workspace - this is what gets serialized to TOML.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkspaceSpec {
    /// Human-readable name
    #[serde(default)]
    pub name: Option<String>,

    /// Description of the workspace purpose
    #[serde(default)]
    pub description: Option<String>,

    // --- Repositories ---
    /// Git repositories to clone into the environment
    #[serde(default)]
    pub repositories: Vec<WorkspaceRepo>,

    // --- Skills ---
    /// Skills to install (from skills.sh or custom)
    #[serde(default)]
    pub skills: Vec<WorkspaceSkill>,

    // --- Pre-commands ---
    /// Commands to run before the agent starts (setup, installs, etc.)
    #[serde(default)]
    pub pre_commands: Vec<PreCommand>,

    // --- Binaries ---
    /// Additional binaries to install in the environment
    #[serde(default)]
    pub binaries: Vec<BinaryInstall>,

    // --- Filesystem ---
    /// Filesystem settings (isolation, mounts, working directory)
    #[serde(default)]
    pub filesystem: FilesystemConfig,

    // --- Agent ---
    /// Agent provider and configuration
    #[serde(default)]
    pub agent: Option<WorkspaceAgentConfig>,

    // --- Subagents ---
    /// Additional agent instances that can be spawned
    #[serde(default)]
    pub subagents: Vec<SubagentConfig>,

    // --- Credentials ---
    /// Credential references (IDs or vault paths)
    #[serde(default)]
    pub credentials: Vec<WorkspaceCredential>,

    // --- Environment ---
    /// Environment variables to set
    #[serde(default)]
    pub env_vars: HashMap<String, String>,

    // --- Resources ---
    /// Resource limits for the sandbox
    #[serde(default)]
    pub resource_limits: Option<ResourceLimits>,

    // --- Network ---
    /// Network configuration
    #[serde(default)]
    pub network: Option<NetworkSpec>,

    // --- Volumes ---
    /// Volume mounts
    #[serde(default)]
    pub volumes: Vec<VolumeMount>,

    // --- Ports ---
    /// Port mappings
    #[serde(default)]
    pub ports: Vec<PortMapping>,

    // --- Labels ---
    /// Metadata labels
    #[serde(default)]
    pub labels: HashMap<String, String>,

    // --- Local mounts ---
    /// Local directories to mount into the sandbox
    #[serde(default)]
    pub local_mounts: Vec<LocalMount>,

    // --- Environment file ---
    /// Path to a .env file to load environment variables from
    #[serde(default)]
    pub env_file: Option<String>,

    // --- Sandbox settings ---
    /// Timeout for the entire sandbox
    #[serde(default)]
    pub timeout_secs: Option<u32>,

    /// Container image override
    #[serde(default)]
    pub image: Option<String>,

    // --- Runtime ---
    /// Per-workspace runtime backend selection
    #[serde(default)]
    pub runtime: Option<WorkspaceRuntimeConfig>,
}

/// A repository associated with the workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceRepo {
    /// Git URL (HTTPS or SSH)
    pub url: String,
    /// Branch to checkout (default: main)
    #[serde(default)]
    pub branch: Option<String>,
    /// Tag to checkout (takes precedence over branch)
    #[serde(default)]
    pub tag: Option<String>,
    /// Specific commit hash
    #[serde(default)]
    pub commit: Option<String>,
    /// Where to clone inside the sandbox (default: /workspace/<repo-name>)
    #[serde(default)]
    pub dest_path: Option<String>,
    /// Shallow clone depth (omit for full clone)
    #[serde(default)]
    pub depth: Option<u32>,
    /// Credential ID for private repos
    #[serde(default)]
    pub credential_id: Option<String>,
    /// Sparse checkout paths (clone only specific directories)
    #[serde(default)]
    pub sparse_paths: Vec<String>,
    /// Submodules: initialize and update
    #[serde(default)]
    pub submodules: bool,
    /// Clone strategy: full clone or worktree
    #[serde(default)]
    pub strategy: GitCloneStrategy,
}

impl From<&WorkspaceRepo> for GitRepoSpec {
    fn from(repo: &WorkspaceRepo) -> Self {
        GitRepoSpec {
            url: repo.url.clone(),
            branch: repo.branch.clone(),
            tag: repo.tag.clone(),
            commit: repo.commit.clone(),
            dest_path: repo.dest_path.clone(),
            credential_id: repo.credential_id.clone(),
            depth: repo.depth,
            sparse_paths: repo.sparse_paths.clone(),
            submodules: repo.submodules,
            strategy: Some(match repo.strategy {
                GitCloneStrategy::Clone => "clone".to_string(),
                GitCloneStrategy::Worktree => "worktree".to_string(),
            }),
            worktree_base_path: None,
        }
    }
}

/// A skill to install in the environment (compatible with skills.sh)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceSkill {
    /// Skill identifier (e.g., "vercel-labs/ai-sdk-best-practices" or a URL)
    pub source: String,
    /// Optional specific version/tag
    #[serde(default)]
    pub version: Option<String>,
    /// Override the skill name
    #[serde(default)]
    pub name: Option<String>,
    /// Whether the skill is enabled (allows toggling without removing)
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Additional configuration for the skill
    #[serde(default)]
    pub config: HashMap<String, serde_json::Value>,
}

fn default_true() -> bool {
    true
}

/// A command to run before the agent starts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreCommand {
    /// Human-readable name for this step
    #[serde(default)]
    pub name: Option<String>,
    /// The command to execute
    pub command: String,
    /// Arguments
    #[serde(default)]
    pub args: Vec<String>,
    /// Working directory for the command
    #[serde(default)]
    pub workdir: Option<String>,
    /// Environment variables specific to this command
    #[serde(default)]
    pub env: HashMap<String, String>,
    /// Whether to fail the entire provisioning if this command fails
    #[serde(default = "default_true")]
    pub fail_on_error: bool,
    /// Timeout in seconds for this command
    #[serde(default)]
    pub timeout_secs: Option<u32>,
}

/// A binary to install in the environment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryInstall {
    /// Binary name (e.g., "ripgrep", "fd", "jq")
    pub name: String,
    /// Installation method
    #[serde(default)]
    pub method: BinaryInstallMethod,
    /// Specific version (e.g., "14.1.0")
    #[serde(default)]
    pub version: Option<String>,
    /// Override install command entirely
    #[serde(default)]
    pub install_command: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BinaryInstallMethod {
    /// Use apt-get install (default for Debian-based images)
    #[default]
    Apt,
    /// Use cargo install
    Cargo,
    /// Use npm install -g
    Npm,
    /// Use pip install
    Pip,
    /// Download from URL
    Url { url: String },
    /// Custom install command
    Custom,
}

/// Filesystem configuration for the workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesystemConfig {
    /// Working directory inside the sandbox
    #[serde(default = "default_workdir")]
    pub workdir: String,
    /// Enable copy-on-write isolation (AgentFS-style)
    #[serde(default)]
    pub cow_isolation: bool,
    /// Paths that should be read-only
    #[serde(default)]
    pub readonly_paths: Vec<String>,
    /// Paths that should be writable (when cow_isolation is on, other paths are CoW)
    #[serde(default)]
    pub writable_paths: Vec<String>,
    /// Temporary directory size limit in MB
    #[serde(default)]
    pub tmp_size_mb: Option<u32>,
    /// Whether to persist filesystem changes across restarts
    #[serde(default)]
    pub persist_changes: bool,
    /// File size limit in bytes (prevent agents from creating huge files)
    #[serde(default)]
    pub max_file_size_bytes: Option<u64>,
    /// Excluded paths (glob patterns, e.g., "**/.git/**")
    #[serde(default)]
    pub exclude_patterns: Vec<String>,
    /// AgentFS configuration for CoW filesystem isolation
    #[serde(default)]
    pub agentfs: Option<AgentFsConfig>,
}

fn default_workdir() -> String {
    "/workspace".to_string()
}

impl Default for FilesystemConfig {
    fn default() -> Self {
        Self {
            workdir: default_workdir(),
            cow_isolation: false,
            readonly_paths: Vec::new(),
            writable_paths: Vec::new(),
            tmp_size_mb: None,
            persist_changes: false,
            max_file_size_bytes: None,
            exclude_patterns: Vec::new(),
            agentfs: None,
        }
    }
}

/// Agent configuration at the workspace level (wraps AgentConfig with workspace-specific fields)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceAgentConfig {
    /// Agent provider (e.g., "claude-code", "codex", "gemini", "cursor")
    pub provider: String,
    /// Model to use
    #[serde(default)]
    pub model: Option<String>,
    /// System prompt for the agent
    #[serde(default)]
    pub system_prompt: Option<String>,
    /// Max tokens per response
    #[serde(default)]
    pub max_tokens: Option<u32>,
    /// Temperature
    #[serde(default)]
    pub temperature: Option<f32>,
    /// Whether tools are enabled
    #[serde(default = "default_true")]
    pub tools_enabled: bool,
    /// MCP servers to connect
    #[serde(default)]
    pub mcp_servers: Vec<super::agent::McpServerConfig>,
    /// Allowed tools whitelist (empty = all allowed)
    #[serde(default)]
    pub allowed_tools: Vec<String>,
    /// Denied tools blacklist
    #[serde(default)]
    pub denied_tools: Vec<String>,
    /// Extra provider-specific settings
    #[serde(default)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl From<&WorkspaceAgentConfig> for AgentConfig {
    fn from(wac: &WorkspaceAgentConfig) -> Self {
        AgentConfig {
            provider: wac.provider.clone(),
            model: wac.model.clone(),
            system_prompt: wac.system_prompt.clone(),
            max_tokens: wac.max_tokens,
            temperature: wac.temperature,
            tools_enabled: wac.tools_enabled,
            mcp_servers: wac.mcp_servers.clone(),
            allowed_tools: wac.allowed_tools.clone(),
            denied_tools: wac.denied_tools.clone(),
            extra: wac.extra.clone(),
        }
    }
}

/// Configuration for a subagent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubagentConfig {
    /// Unique name for this subagent within the workspace
    pub name: String,
    /// Agent provider
    pub provider: String,
    /// Model
    #[serde(default)]
    pub model: Option<String>,
    /// System prompt for the subagent
    #[serde(default)]
    pub system_prompt: Option<String>,
    /// When to activate (always, on_demand, on_event)
    #[serde(default)]
    pub activation: SubagentActivation,
    /// Tools available to this subagent
    #[serde(default)]
    pub allowed_tools: Vec<String>,
    /// MCP servers for this subagent
    #[serde(default)]
    pub mcp_servers: Vec<super::agent::McpServerConfig>,
    /// Extra config
    #[serde(default)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubagentActivation {
    /// Always running alongside the main agent
    Always,
    /// Started on demand when the main agent requests it
    #[default]
    OnDemand,
    /// Triggered by specific events
    OnEvent { events: Vec<String> },
}

/// A credential reference in a workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceCredential {
    /// Credential ID (references ciab credential store)
    #[serde(default)]
    pub id: Option<String>,
    /// Credential name (alternative to ID, looked up by name)
    #[serde(default)]
    pub name: Option<String>,
    /// Vault provider (e.g., "local", "aws-secrets-manager", "hashicorp-vault", "1password")
    #[serde(default = "default_vault_provider")]
    pub vault_provider: String,
    /// Vault path/key for external vault providers
    #[serde(default)]
    pub vault_path: Option<String>,
    /// Environment variable name to inject the secret as
    #[serde(default)]
    pub env_var: Option<String>,
    /// File path to write the secret to
    #[serde(default)]
    pub file_path: Option<String>,
}

fn default_vault_provider() -> String {
    "local".to_string()
}

/// TOML-friendly workspace definition for CI/single-run
/// This is what users put in their `ciab-workspace.toml` files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceToml {
    pub workspace: WorkspaceSpec,
}

/// Filters for listing workspaces
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkspaceFilters {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub labels: HashMap<String, String>,
}

// ---------------------------------------------------------------------------
// Template system
// ---------------------------------------------------------------------------

/// Label key used to identify workspaces that are templates
pub const TEMPLATE_KIND_LABEL: &str = "ciab/kind";
pub const TEMPLATE_KIND_VALUE: &str = "template";
/// Label key for the source ID that imported a template
pub const TEMPLATE_SOURCE_ID_LABEL: &str = "ciab/source_id";
/// Label key for the filename within the source repo
pub const TEMPLATE_SOURCE_FILE_LABEL: &str = "ciab/source_file";

/// A Git repository source that contains workspace templates.
/// Templates are stored as TOML files under a configurable path in the repo.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateSource {
    pub id: Uuid,
    pub name: String,
    pub url: String,
    #[serde(default = "default_branch")]
    pub branch: String,
    #[serde(default = "default_templates_path")]
    pub templates_path: String,
    pub last_synced_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub template_count: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

fn default_branch() -> String {
    "main".to_string()
}

fn default_templates_path() -> String {
    ".ciab/templates".to_string()
}
