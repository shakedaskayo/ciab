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

pub struct GeminiProvider;

#[async_trait]
impl AgentProvider for GeminiProvider {
    fn name(&self) -> &str {
        "gemini"
    }

    fn base_image(&self) -> &str {
        "ghcr.io/ciab/gemini-sandbox:latest"
    }

    fn install_commands(&self) -> Vec<String> {
        vec!["npm install -g @google/gemini-cli".to_string()]
    }

    fn build_start_command(&self, config: &AgentConfig) -> AgentCommand {
        // Gemini CLI: `gemini --output-format stream-json [flags] "prompt"`
        // The prompt is appended as a positional arg by the session handler (PromptMode::CliArgument).
        let mut args = vec!["--output-format".to_string(), "stream-json".to_string()];

        if let Some(ref model) = config.model {
            args.push("--model".to_string());
            args.push(model.clone());
        }

        // Permission mode mapping.
        if let Some(mode) = config.extra.get("permission_mode").and_then(|v| v.as_str()) {
            match mode {
                "auto_approve" | "unrestricted" => {
                    args.push("--yolo".to_string());
                }
                "approve_edits" => {
                    args.push("--approval-mode".to_string());
                    args.push("auto_edit".to_string());
                }
                // approve_all → default (gemini prompts for approval)
                // plan_only → not directly supported, we pass default
                _ => {}
            }
        }

        // Sandbox mode.
        if config
            .extra
            .get("sandbox")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            args.push("--sandbox".to_string());
        }

        // Debug mode.
        if config
            .extra
            .get("debug")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            args.push("--debug".to_string());
        }

        // Resume previous session.
        if let Some(session_id) = config
            .extra
            .get("resume_session_id")
            .and_then(|v| v.as_str())
        {
            args.push("--resume".to_string());
            args.push(session_id.to_string());
        }

        // Allowed tools.
        if !config.allowed_tools.is_empty() {
            args.push("--allowed-tools".to_string());
            args.push(config.allowed_tools.join(","));
        }

        // Extensions.
        if let Some(extensions) = config.extra.get("extensions").and_then(|v| v.as_str()) {
            args.push("--extensions".to_string());
            args.push(extensions.to_string());
        }

        AgentCommand {
            command: "gemini".to_string(),
            args,
            env: Default::default(),
            workdir: None,
        }
    }

    fn prompt_mode(&self) -> PromptMode {
        PromptMode::CliArgument
    }

    fn required_env_vars(&self) -> Vec<String> {
        vec!["GOOGLE_API_KEY".to_string()]
    }

    /// Parse Gemini CLI `--output-format stream-json` NDJSON output.
    ///
    /// Gemini emits NDJSON with these event types:
    /// - `init` — session initialization with model, cwd
    /// - `message` — assistant text content (streamed)
    /// - `tool_use` — tool invocation started
    /// - `tool_result` — tool invocation completed with result
    /// - `error` — error event
    /// - `result` — final result with response text, stats, session_id
    ///
    /// Also handles the alternate format where events use `type: "system"` etc.
    /// (Gemini CLI format has evolved across versions).
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
                    events.push(StreamEvent {
                        id: Uuid::new_v4().to_string(),
                        sandbox_id: *sandbox_id,
                        session_id: None,
                        event_type: StreamEventType::LogLine,
                        data: json!({ "line": line }),
                        timestamp: Utc::now(),
                    });
                    continue;
                }
            };

            let event_type = obj.get("type").and_then(|t| t.as_str()).unwrap_or("");

            match event_type {
                "init" | "system" => {
                    events.push(StreamEvent {
                        id: Uuid::new_v4().to_string(),
                        sandbox_id: *sandbox_id,
                        session_id: None,
                        event_type: StreamEventType::Connected,
                        data: json!({
                            "session_id": obj.get("session_id"),
                            "model": obj.get("model"),
                            "cwd": obj.get("cwd"),
                            "tools": obj.get("tools"),
                        }),
                        timestamp: Utc::now(),
                    });
                }

                "message" | "assistant" => {
                    // Gemini streams assistant text via "message" events.
                    // Can be a simple text field or content blocks.
                    if let Some(text) = obj.get("text").and_then(|t| t.as_str()) {
                        events.push(StreamEvent {
                            id: Uuid::new_v4().to_string(),
                            sandbox_id: *sandbox_id,
                            session_id: None,
                            event_type: StreamEventType::TextDelta,
                            data: json!({ "text": text }),
                            timestamp: Utc::now(),
                        });
                    }
                    // Handle content block array format.
                    if let Some(content) = obj
                        .get("message")
                        .and_then(|m| m.get("content"))
                        .and_then(|c| c.as_array())
                    {
                        for block in content {
                            let block_type =
                                block.get("type").and_then(|t| t.as_str()).unwrap_or("");
                            match block_type {
                                "text" => {
                                    if let Some(text) = block.get("text").and_then(|t| t.as_str()) {
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
                                "tool_use" => {
                                    events.push(StreamEvent {
                                        id: Uuid::new_v4().to_string(),
                                        sandbox_id: *sandbox_id,
                                        session_id: None,
                                        event_type: StreamEventType::ToolUseStart,
                                        data: json!({
                                            "id": block.get("id"),
                                            "name": block.get("name"),
                                            "input": block.get("input").cloned().unwrap_or(json!({})),
                                        }),
                                        timestamp: Utc::now(),
                                    });
                                }
                                _ => {}
                            }
                        }
                    }
                    // Handle plain content string.
                    if let Some(text) = obj
                        .get("message")
                        .and_then(|m| m.get("content"))
                        .and_then(|c| c.as_str())
                    {
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

                "tool_use" => {
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
                            "content": obj.get("content").or_else(|| obj.get("output")).or_else(|| obj.get("result")),
                            "is_error": obj.get("is_error").and_then(|v| v.as_bool()).unwrap_or(false),
                        }),
                        timestamp: Utc::now(),
                    });
                }

                "error" => {
                    events.push(StreamEvent {
                        id: Uuid::new_v4().to_string(),
                        sandbox_id: *sandbox_id,
                        session_id: None,
                        event_type: StreamEventType::ResultError,
                        data: json!({
                            "error_type": "error",
                            "message": obj.get("message").or_else(|| obj.get("error")),
                        }),
                        timestamp: Utc::now(),
                    });

                    events.push(StreamEvent {
                        id: Uuid::new_v4().to_string(),
                        sandbox_id: *sandbox_id,
                        session_id: None,
                        event_type: StreamEventType::SessionCompleted,
                        data: json!({
                            "session_id": obj.get("session_id"),
                            "error": true,
                        }),
                        timestamp: Utc::now(),
                    });
                }

                "result" => {
                    // Extract response text.
                    let response_text = obj
                        .get("response")
                        .and_then(|r| r.as_str())
                        .or_else(|| obj.get("result").and_then(|r| r.as_str()));

                    if let Some(text) = response_text {
                        events.push(StreamEvent {
                            id: Uuid::new_v4().to_string(),
                            sandbox_id: *sandbox_id,
                            session_id: None,
                            event_type: StreamEventType::TextComplete,
                            data: json!({
                                "text": text,
                                "session_id": obj.get("session_id"),
                                "stats": obj.get("stats"),
                            }),
                            timestamp: Utc::now(),
                        });
                    }

                    // Signal session completion.
                    events.push(StreamEvent {
                        id: Uuid::new_v4().to_string(),
                        sandbox_id: *sandbox_id,
                        session_id: None,
                        event_type: StreamEventType::SessionCompleted,
                        data: json!({
                            "session_id": obj.get("session_id"),
                            "stats": obj.get("stats"),
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
        if config.provider != "gemini" {
            return Err(CiabError::ConfigValidationError(format!(
                "expected provider 'gemini', got '{}'",
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
                name: "stats".into(),
                description: "Show usage statistics".into(),
                category: SlashCommandCategory::Session,
                args: vec![],
                provider_native: true,
            },
        ]
    }

    fn supported_llm_providers(&self) -> Vec<AgentLlmCompatibility> {
        vec![AgentLlmCompatibility {
            agent_provider: "gemini".to_string(),
            llm_provider_kind: LlmProviderKind::Google,
            env_var_mapping: [("GOOGLE_API_KEY".to_string(), "{api_key}".to_string())]
                .into_iter()
                .collect(),
            supports_model_override: true,
            notes: Some("Native provider".to_string()),
        }]
    }
}
