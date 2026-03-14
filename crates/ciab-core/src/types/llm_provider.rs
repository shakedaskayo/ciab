use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// The kind of LLM inference backend.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LlmProviderKind {
    Anthropic,
    OpenAi,
    Google,
    Ollama,
    OpenRouter,
    Custom,
}

impl std::fmt::Display for LlmProviderKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Anthropic => write!(f, "anthropic"),
            Self::OpenAi => write!(f, "openai"),
            Self::Google => write!(f, "google"),
            Self::Ollama => write!(f, "ollama"),
            Self::OpenRouter => write!(f, "openrouter"),
            Self::Custom => write!(f, "custom"),
        }
    }
}

impl std::str::FromStr for LlmProviderKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "anthropic" => Ok(Self::Anthropic),
            "openai" | "open_ai" => Ok(Self::OpenAi),
            "google" => Ok(Self::Google),
            "ollama" => Ok(Self::Ollama),
            "openrouter" | "open_router" => Ok(Self::OpenRouter),
            "custom" => Ok(Self::Custom),
            _ => Err(format!("unknown LLM provider kind: {}", s)),
        }
    }
}

/// An LLM provider — a source of model inference with credentials and base URL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmProvider {
    pub id: Uuid,
    pub name: String,
    pub kind: LlmProviderKind,
    pub enabled: bool,
    pub base_url: Option<String>,
    /// Links to the credentials table for API key storage.
    pub api_key_credential_id: Option<Uuid>,
    pub default_model: Option<String>,
    pub is_local: bool,
    pub auto_detected: bool,
    pub extra: HashMap<String, Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A model available from an LLM provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmModel {
    pub id: String,
    pub name: String,
    pub provider_id: Uuid,
    pub context_window: Option<u64>,
    pub supports_tools: bool,
    pub supports_vision: bool,
    pub is_local: bool,
    pub size_bytes: Option<u64>,
    pub family: Option<String>,
}

/// Describes how an agent provider can use an LLM provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentLlmCompatibility {
    pub agent_provider: String,
    pub llm_provider_kind: LlmProviderKind,
    /// Template for env var injection, e.g. {"ANTHROPIC_BASE_URL": "{base_url}"}
    pub env_var_mapping: HashMap<String, String>,
    pub supports_model_override: bool,
    pub notes: Option<String>,
}
