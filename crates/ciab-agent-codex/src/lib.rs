use async_trait::async_trait;
use chrono::Utc;
use serde_json::json;
use tokio::sync::mpsc;
use tracing::debug;
use uuid::Uuid;

use ciab_core::error::{CiabError, CiabResult};
use ciab_core::traits::agent::AgentProvider;
use ciab_core::types::agent::{
    AgentCommand, AgentConfig, AgentHealth, PromptMode, SlashCommand, SlashCommandArg,
    SlashCommandCategory,
};
use ciab_core::types::llm_provider::{AgentLlmCompatibility, LlmProviderKind};
use ciab_core::types::session::Message;
use ciab_core::types::stream::{StreamEvent, StreamEventType};

pub struct CodexProvider;

#[async_trait]
impl AgentProvider for CodexProvider {
    fn name(&self) -> &str {
        "codex"
    }

    fn base_image(&self) -> &str {
        "ghcr.io/ciab/codex-sandbox:latest"
    }

    fn install_commands(&self) -> Vec<String> {
        vec!["npm install -g @openai/codex".to_string()]
    }

    fn build_start_command(&self, config: &AgentConfig) -> AgentCommand {
        // Codex CLI: `codex --quiet --full-auto "prompt"`
        // The prompt is appended as a positional arg by the session handler (PromptMode::CliArgument).
        let mut args = vec!["--quiet".to_string()];

        if let Some(ref model) = config.model {
            args.push("--model".to_string());
            args.push(model.clone());
        }

        // Approval mode mapping.
        if let Some(mode) = config.extra.get("permission_mode").and_then(|v| v.as_str()) {
            match mode {
                "auto_approve" | "unrestricted" => {
                    args.push("--full-auto".to_string());
                }
                "approve_edits" => {
                    args.push("--auto-edit".to_string());
                }
                // approve_all → suggest mode (default)
                // plan_only → suggest mode (closest equivalent)
                _ => {}
            }
        }

        let mut env: std::collections::HashMap<String, String> = Default::default();

        // LLM provider override
        if let Some(base_url) = config.extra.get("llm_base_url").and_then(|v| v.as_str()) {
            env.insert("OPENAI_BASE_URL".to_string(), base_url.to_string());
        }
        if let Some(api_key) = config.extra.get("llm_api_key").and_then(|v| v.as_str()) {
            env.insert("OPENAI_API_KEY".to_string(), api_key.to_string());
        }

        AgentCommand {
            command: "codex".to_string(),
            args,
            env,
            workdir: None,
        }
    }

    fn prompt_mode(&self) -> PromptMode {
        PromptMode::CliArgument
    }

    fn required_env_vars(&self) -> Vec<String> {
        vec!["OPENAI_API_KEY".to_string()]
    }

    /// Parse Codex CLI output.
    ///
    /// Codex outputs a mix of plain text and JSON. In `--quiet` mode it outputs
    /// the agent's response. We parse each line looking for JSON events, falling
    /// back to plain text lines as TextDelta.
    fn parse_output(&self, sandbox_id: &Uuid, raw: &str) -> Vec<StreamEvent> {
        let mut events = Vec::new();

        for line in raw.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let obj: serde_json::Value = match serde_json::from_str(line) {
                Ok(v) => v,
                Err(_) => {
                    // Plain text output from Codex — treat as text delta.
                    events.push(StreamEvent {
                        id: Uuid::new_v4().to_string(),
                        sandbox_id: *sandbox_id,
                        session_id: None,
                        event_type: StreamEventType::TextDelta,
                        data: json!({ "text": format!("{}\n", line) }),
                        timestamp: Utc::now(),
                    });
                    continue;
                }
            };

            let event_type = obj.get("type").and_then(|t| t.as_str()).unwrap_or("");

            match event_type {
                "system" | "init" => {
                    events.push(StreamEvent {
                        id: Uuid::new_v4().to_string(),
                        sandbox_id: *sandbox_id,
                        session_id: None,
                        event_type: StreamEventType::Connected,
                        data: json!({
                            "session_id": obj.get("session_id"),
                            "model": obj.get("model"),
                        }),
                        timestamp: Utc::now(),
                    });
                }

                "message" | "assistant" | "text" => {
                    let text = obj
                        .get("text")
                        .or_else(|| obj.get("content"))
                        .or_else(|| obj.get("message"))
                        .and_then(|t| t.as_str())
                        .unwrap_or("");
                    if !text.is_empty() {
                        events.push(StreamEvent {
                            id: Uuid::new_v4().to_string(),
                            sandbox_id: *sandbox_id,
                            session_id: None,
                            event_type: StreamEventType::TextDelta,
                            data: json!({ "text": text }),
                            timestamp: Utc::now(),
                        });
                    }
                }

                "tool_use" | "tool_call" => {
                    let name = obj
                        .get("name")
                        .or_else(|| obj.get("tool_name"))
                        .and_then(|n| n.as_str())
                        .unwrap_or("unknown");
                    events.push(StreamEvent {
                        id: Uuid::new_v4().to_string(),
                        sandbox_id: *sandbox_id,
                        session_id: None,
                        event_type: StreamEventType::ToolUseStart,
                        data: json!({
                            "id": obj.get("id").or_else(|| obj.get("tool_use_id")),
                            "name": name,
                            "input": obj.get("input").or_else(|| obj.get("args")).cloned().unwrap_or(json!({})),
                        }),
                        timestamp: Utc::now(),
                    });
                }

                "tool_result" => {
                    events.push(StreamEvent {
                        id: Uuid::new_v4().to_string(),
                        sandbox_id: *sandbox_id,
                        session_id: None,
                        event_type: StreamEventType::ToolResult,
                        data: json!({
                            "tool_use_id": obj.get("tool_use_id").or_else(|| obj.get("id")),
                            "content": obj.get("content").or_else(|| obj.get("output")),
                            "is_error": obj.get("is_error").and_then(|v| v.as_bool()).unwrap_or(false),
                        }),
                        timestamp: Utc::now(),
                    });
                }

                "result" => {
                    if let Some(text) = obj
                        .get("result")
                        .or_else(|| obj.get("response"))
                        .and_then(|r| r.as_str())
                    {
                        events.push(StreamEvent {
                            id: Uuid::new_v4().to_string(),
                            sandbox_id: *sandbox_id,
                            session_id: None,
                            event_type: StreamEventType::TextComplete,
                            data: json!({
                                "text": text,
                                "session_id": obj.get("session_id"),
                            }),
                            timestamp: Utc::now(),
                        });
                    }

                    events.push(StreamEvent {
                        id: Uuid::new_v4().to_string(),
                        sandbox_id: *sandbox_id,
                        session_id: None,
                        event_type: StreamEventType::SessionCompleted,
                        data: json!({
                            "session_id": obj.get("session_id"),
                        }),
                        timestamp: Utc::now(),
                    });
                }

                _ => {
                    events.push(StreamEvent {
                        id: Uuid::new_v4().to_string(),
                        sandbox_id: *sandbox_id,
                        session_id: None,
                        event_type: StreamEventType::LogLine,
                        data: obj,
                        timestamp: Utc::now(),
                    });
                }
            }
        }

        events
    }

    fn validate_config(&self, config: &AgentConfig) -> CiabResult<()> {
        if config.provider != "codex" {
            return Err(CiabError::ConfigValidationError(format!(
                "expected provider 'codex', got '{}'",
                config.provider
            )));
        }
        Ok(())
    }

    async fn send_message(
        &self,
        sandbox_id: &Uuid,
        session_id: &Uuid,
        message: &Message,
        tx: &mpsc::Sender<StreamEvent>,
    ) -> CiabResult<()> {
        debug!(
            sandbox_id = %sandbox_id,
            session_id = %session_id,
            "stub: message would be sent via execd"
        );

        let event = StreamEvent {
            id: Uuid::new_v4().to_string(),
            sandbox_id: *sandbox_id,
            session_id: Some(*session_id),
            event_type: StreamEventType::TextDelta,
            data: json!({
                "text": format!(
                    "stub: message with {} content part(s) would be sent via execd",
                    message.content.len()
                )
            }),
            timestamp: Utc::now(),
        };

        tx.send(event).await.map_err(|e| {
            CiabError::AgentCommunicationError(format!("failed to send event: {}", e))
        })?;

        Ok(())
    }

    async fn interrupt(&self, _sandbox_id: &Uuid) -> CiabResult<()> {
        Ok(())
    }

    async fn health_check(&self, _sandbox_id: &Uuid) -> CiabResult<AgentHealth> {
        Ok(AgentHealth {
            healthy: true,
            status: "ok".into(),
            uptime_secs: None,
        })
    }

    fn slash_commands(&self) -> Vec<SlashCommand> {
        vec![
            SlashCommand {
                name: "clear".into(),
                description: "Clear conversation history".into(),
                category: SlashCommandCategory::Session,
                args: vec![],
                provider_native: false,
            },
            SlashCommand {
                name: "help".into(),
                description: "Show available commands".into(),
                category: SlashCommandCategory::Help,
                args: vec![],
                provider_native: false,
            },
            SlashCommand {
                name: "model".into(),
                description: "Switch model".into(),
                category: SlashCommandCategory::Agent,
                args: vec![SlashCommandArg {
                    name: "model".into(),
                    description: "Model name to switch to".into(),
                    required: false,
                }],
                provider_native: true,
            },
            SlashCommand {
                name: "approval-mode".into(),
                description: "Set approval mode (suggest, auto-edit, full-auto)".into(),
                category: SlashCommandCategory::Agent,
                args: vec![SlashCommandArg {
                    name: "mode".into(),
                    description: "Approval mode".into(),
                    required: false,
                }],
                provider_native: true,
            },
        ]
    }

    fn supported_llm_providers(&self) -> Vec<AgentLlmCompatibility> {
        vec![
            AgentLlmCompatibility {
                agent_provider: "codex".to_string(),
                llm_provider_kind: LlmProviderKind::OpenAi,
                env_var_mapping: [("OPENAI_API_KEY".to_string(), "{api_key}".to_string())]
                    .into_iter()
                    .collect(),
                supports_model_override: true,
                notes: Some("Native provider".to_string()),
            },
            AgentLlmCompatibility {
                agent_provider: "codex".to_string(),
                llm_provider_kind: LlmProviderKind::OpenRouter,
                env_var_mapping: [
                    (
                        "OPENAI_BASE_URL".to_string(),
                        "https://openrouter.ai/api/v1".to_string(),
                    ),
                    ("OPENAI_API_KEY".to_string(), "{api_key}".to_string()),
                ]
                .into_iter()
                .collect(),
                supports_model_override: true,
                notes: Some("Via OPENAI_BASE_URL override".to_string()),
            },
            // Ollama: Codex uses the OpenAI-compatible endpoint via OPENAI_BASE_URL.
            AgentLlmCompatibility {
                agent_provider: "codex".to_string(),
                llm_provider_kind: LlmProviderKind::Ollama,
                env_var_mapping: [
                    (
                        "OPENAI_BASE_URL".to_string(),
                        "{base_url}/v1".to_string(),
                    ),
                    ("OPENAI_API_KEY".to_string(), "ollama".to_string()),
                ]
                .into_iter()
                .collect(),
                supports_model_override: true,
                notes: Some("Via OPENAI_BASE_URL → Ollama OpenAI-compatible endpoint".to_string()),
            },
        ]
    }
}
