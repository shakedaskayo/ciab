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

pub struct CursorProvider;

#[async_trait]
impl AgentProvider for CursorProvider {
    fn name(&self) -> &str {
        "cursor"
    }

    fn base_image(&self) -> &str {
        "ghcr.io/ciab/cursor-sandbox:latest"
    }

    fn install_commands(&self) -> Vec<String> {
        vec!["curl -fsSL https://cursor.sh/install.sh | bash".to_string()]
    }

    fn build_start_command(&self, config: &AgentConfig) -> AgentCommand {
        // Cursor agent CLI: `cursor agent --print --output-format stream-json [flags] "prompt"`
        // The prompt is appended as a positional arg by the session handler (PromptMode::CliArgument).
        let mut args = vec![
            "agent".to_string(),
            "--print".to_string(),
            "--output-format".to_string(),
            "stream-json".to_string(),
            // Trust workspace without prompting — required for non-interactive use.
            "--trust".to_string(),
        ];

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
                "plan_only" => {
                    args.push("--mode".to_string());
                    args.push("plan".to_string());
                }
                // approve_edits, approve_all — Cursor doesn't have fine-grained control,
                // so we use default (interactive prompting is handled by CIAB).
                _ => {}
            }
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

        // Continue last conversation.
        if config
            .extra
            .get("continue_session")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            args.push("--continue".to_string());
        }

        // MCP server auto-approval.
        if !config.mcp_servers.is_empty() {
            args.push("--approve-mcps".to_string());
        }

        AgentCommand {
            command: "cursor".to_string(),
            args,
            env: Default::default(),
            workdir: None,
        }
    }

    fn prompt_mode(&self) -> PromptMode {
        PromptMode::CliArgument
    }

    fn required_env_vars(&self) -> Vec<String> {
        vec!["CURSOR_API_KEY".to_string()]
    }

    /// Parse Cursor CLI `--output-format stream-json` NDJSON output.
    ///
    /// Cursor emits NDJSON with these event types:
    /// - `system` (subtype `init`) — session metadata (session_id, model, cwd, permissionMode)
    /// - `user` — user message echo
    /// - `assistant` — complete assistant response segment with content blocks
    /// - `tool_call` (subtype `started`/`completed`) — tool invocations
    /// - `result` (subtype `success`) — final result with duration, full text, session_id
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
                "system" => {
                    let subtype = obj
                        .get("subtype")
                        .and_then(|s| s.as_str())
                        .unwrap_or("init");
                    match subtype {
                        "init" => {
                            events.push(StreamEvent {
                                id: Uuid::new_v4().to_string(),
                                sandbox_id: *sandbox_id,
                                session_id: None,
                                event_type: StreamEventType::Connected,
                                data: json!({
                                    "session_id": obj.get("session_id"),
                                    "model": obj.get("model"),
                                    "cwd": obj.get("cwd"),
                                    "permission_mode": obj.get("permissionMode"),
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
                                data: obj.clone(),
                                timestamp: Utc::now(),
                            });
                        }
                    }
                }

                "assistant" => {
                    // Cursor's assistant messages contain content blocks similar to Claude.
                    // Extract text and tool_use blocks.
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
                                    let name =
                                        block.get("name").and_then(|n| n.as_str()).unwrap_or("");
                                    events.push(StreamEvent {
                                        id: Uuid::new_v4().to_string(),
                                        sandbox_id: *sandbox_id,
                                        session_id: None,
                                        event_type: StreamEventType::ToolUseStart,
                                        data: json!({
                                            "id": block.get("id"),
                                            "name": name,
                                            "input": block.get("input").cloned().unwrap_or(json!({})),
                                        }),
                                        timestamp: Utc::now(),
                                    });
                                }
                                _ => {}
                            }
                        }
                    }
                    // Fallback: if message is a plain text string.
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

                "tool_call" => {
                    let subtype = obj.get("subtype").and_then(|s| s.as_str()).unwrap_or("");
                    match subtype {
                        "started" => {
                            let name = obj
                                .get("tool_name")
                                .or_else(|| obj.get("name"))
                                .and_then(|n| n.as_str())
                                .unwrap_or("unknown");
                            events.push(StreamEvent {
                                id: Uuid::new_v4().to_string(),
                                sandbox_id: *sandbox_id,
                                session_id: None,
                                event_type: StreamEventType::ToolUseStart,
                                data: json!({
                                    "id": obj.get("tool_call_id").or_else(|| obj.get("id")),
                                    "name": name,
                                    "input": obj.get("args").or_else(|| obj.get("input")).cloned().unwrap_or(json!({})),
                                }),
                                timestamp: Utc::now(),
                            });
                        }
                        "completed" => {
                            events.push(StreamEvent {
                                id: Uuid::new_v4().to_string(),
                                sandbox_id: *sandbox_id,
                                session_id: None,
                                event_type: StreamEventType::ToolResult,
                                data: json!({
                                    "tool_use_id": obj.get("tool_call_id").or_else(|| obj.get("id")),
                                    "content": obj.get("result").or_else(|| obj.get("output")),
                                    "is_error": obj.get("is_error").and_then(|v| v.as_bool()).unwrap_or(false),
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
                                data: obj.clone(),
                                timestamp: Utc::now(),
                            });
                        }
                    }
                }

                "user" => {
                    // User message echo — skip (we already have the user message).
                }

                "result" => {
                    let subtype = obj
                        .get("subtype")
                        .and_then(|s| s.as_str())
                        .unwrap_or("success");

                    if obj
                        .get("is_error")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false)
                    {
                        events.push(StreamEvent {
                            id: Uuid::new_v4().to_string(),
                            sandbox_id: *sandbox_id,
                            session_id: None,
                            event_type: StreamEventType::ResultError,
                            data: json!({
                                "error_type": subtype,
                                "message": obj.get("result"),
                                "duration_ms": obj.get("duration_ms"),
                                "session_id": obj.get("session_id"),
                            }),
                            timestamp: Utc::now(),
                        });
                    } else {
                        if let Some(text) = obj.get("result").and_then(|r| r.as_str()) {
                            events.push(StreamEvent {
                                id: Uuid::new_v4().to_string(),
                                sandbox_id: *sandbox_id,
                                session_id: None,
                                event_type: StreamEventType::TextComplete,
                                data: json!({
                                    "text": text,
                                    "duration_ms": obj.get("duration_ms"),
                                    "duration_api_ms": obj.get("duration_api_ms"),
                                    "session_id": obj.get("session_id"),
                                    "request_id": obj.get("request_id"),
                                }),
                                timestamp: Utc::now(),
                            });
                        }
                    }

                    // Signal session completion.
                    events.push(StreamEvent {
                        id: Uuid::new_v4().to_string(),
                        sandbox_id: *sandbox_id,
                        session_id: None,
                        event_type: StreamEventType::SessionCompleted,
                        data: json!({
                            "duration_ms": obj.get("duration_ms"),
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
        if config.provider != "cursor" {
            return Err(CiabError::ConfigValidationError(format!(
                "expected provider 'cursor', got '{}'",
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
                name: "mode".into(),
                description: "Switch mode (agent, plan, ask)".into(),
                category: SlashCommandCategory::Agent,
                args: vec![SlashCommandArg {
                    name: "mode".into(),
                    description: "Mode: agent, plan, ask".into(),
                    required: false,
                }],
                provider_native: true,
            },
        ]
    }

    fn supported_llm_providers(&self) -> Vec<AgentLlmCompatibility> {
        vec![AgentLlmCompatibility {
            agent_provider: "cursor".to_string(),
            llm_provider_kind: LlmProviderKind::OpenAi,
            env_var_mapping: [("CURSOR_API_KEY".to_string(), "{api_key}".to_string())]
                .into_iter()
                .collect(),
            supports_model_override: true,
            notes: Some("Uses OpenAI-compatible API".to_string()),
        }]
    }
}
