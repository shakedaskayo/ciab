use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Permission mode for agent tool execution gating.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionMode {
    /// Auto-approve all tool calls — no confirmation prompts.
    AutoApprove,
    /// Require approval for edits and commands (Bash, Edit, Write) only.
    ApproveEdits,
    /// Require approval for every tool call.
    ApproveAll,
    /// Read-only mode — agent plans but cannot execute.
    PlanOnly,
    /// Skip ALL permission checks — CIAB gates + agent native.
    Unrestricted,
}

impl Default for PermissionMode {
    fn default() -> Self {
        Self::AutoApprove
    }
}

/// Policy controlling which tool calls require user approval.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionPolicy {
    pub mode: PermissionMode,
    /// Tool names that always require approval regardless of mode.
    #[serde(default)]
    pub always_require_approval: Vec<String>,
    /// Tool names that are always allowed regardless of mode.
    #[serde(default)]
    pub always_allow: Vec<String>,
}

impl Default for PermissionPolicy {
    fn default() -> Self {
        Self {
            mode: PermissionMode::AutoApprove,
            always_require_approval: Vec::new(),
            always_allow: Vec::new(),
        }
    }
}

/// Tools considered "write" operations for ApproveEdits mode.
const WRITE_TOOLS: &[&str] = &["Bash", "Edit", "Write", "NotebookEdit", "MultiEdit"];

impl PermissionPolicy {
    /// Returns true if the given tool name requires user approval under this policy.
    pub fn requires_approval(&self, tool_name: &str) -> bool {
        // Unrestricted mode skips everything
        if self.mode == PermissionMode::Unrestricted {
            return false;
        }

        if self.always_allow.iter().any(|t| t == tool_name) {
            return false;
        }
        if self.always_require_approval.iter().any(|t| t == tool_name) {
            return true;
        }

        match self.mode {
            PermissionMode::AutoApprove => false,
            PermissionMode::ApproveEdits => WRITE_TOOLS.iter().any(|&t| t == tool_name),
            PermissionMode::ApproveAll => true,
            PermissionMode::PlanOnly => false, // handled by CLI flag, not post-hoc gating
            PermissionMode::Unrestricted => false, // handled above but needed for exhaustive match
        }
    }

    /// Returns a risk level string for the given tool.
    pub fn risk_level(tool_name: &str) -> &'static str {
        match tool_name {
            "Bash" => "high",
            "Edit" | "Write" | "NotebookEdit" | "MultiEdit" => "medium",
            _ => "low",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub provider: String,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub system_prompt: Option<String>,
    #[serde(default)]
    pub max_tokens: Option<u32>,
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default = "default_tools_enabled")]
    pub tools_enabled: bool,
    #[serde(default)]
    pub mcp_servers: Vec<McpServerConfig>,
    #[serde(default)]
    pub allowed_tools: Vec<String>,
    #[serde(default)]
    pub denied_tools: Vec<String>,
    #[serde(default)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            provider: String::new(),
            model: None,
            system_prompt: None,
            max_tokens: None,
            temperature: None,
            tools_enabled: true,
            mcp_servers: Vec::new(),
            allowed_tools: Vec::new(),
            denied_tools: Vec::new(),
            extra: HashMap::new(),
        }
    }
}

fn default_tools_enabled() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

/// How prompts are delivered to the agent process.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PromptMode {
    /// Send prompt via stdin as NDJSON: `{"type":"user","message":{"role":"user","content":"..."}}`
    /// Used by Claude Code with `--input-format stream-json`.
    StdinJson,
    /// Append prompt as a positional CLI argument to the command.
    /// Used by Cursor CLI (`cursor agent --print "prompt"`) and Gemini CLI (`gemini "prompt"`).
    CliArgument,
    /// Send prompt as plain text line to stdin.
    StdinPlaintext,
}

impl Default for PromptMode {
    fn default() -> Self {
        Self::StdinJson
    }
}

/// Whether the provider supports interactive stdin control protocol
/// (control_request/control_response for permissions, questions, etc.)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InteractiveProtocol {
    /// Full stdin JSON control protocol (Claude Code).
    ControlRequestResponse,
    /// No interactive stdin protocol — agent runs to completion.
    None,
}

impl Default for InteractiveProtocol {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCommand {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub workdir: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentHealth {
    pub healthy: bool,
    pub status: String,
    #[serde(default)]
    pub uptime_secs: Option<u64>,
}

/// A slash command available in the chat interface.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlashCommand {
    pub name: String,
    pub description: String,
    pub category: SlashCommandCategory,
    #[serde(default)]
    pub args: Vec<SlashCommandArg>,
    pub provider_native: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SlashCommandCategory {
    Session,
    Agent,
    Tools,
    Navigation,
    Help,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlashCommandArg {
    pub name: String,
    pub description: String,
    pub required: bool,
}
