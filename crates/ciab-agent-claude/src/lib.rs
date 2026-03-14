use async_trait::async_trait;
use chrono::Utc;
use serde_json::json;
use tokio::sync::mpsc;
use tracing::debug;
use uuid::Uuid;

use ciab_core::error::{CiabError, CiabResult};
use ciab_core::traits::agent::AgentProvider;
use ciab_core::types::agent::{
    AgentCommand, AgentConfig, AgentHealth, InteractiveProtocol, PromptMode, SlashCommand,
    SlashCommandArg, SlashCommandCategory,
};
use ciab_core::types::session::Message;
use ciab_core::types::stream::{StreamEvent, StreamEventType};

pub struct ClaudeCodeProvider;

#[async_trait]
impl AgentProvider for ClaudeCodeProvider {
    fn name(&self) -> &str {
        "claude-code"
    }

    fn base_image(&self) -> &str {
        "ghcr.io/ciab/claude-sandbox:latest"
    }

    fn install_commands(&self) -> Vec<String> {
        vec!["npm install -g @anthropic-ai/claude-code@latest".to_string()]
    }

    fn build_start_command(&self, config: &AgentConfig) -> AgentCommand {
        let mut args = vec![
            "--print".to_string(),
            "--output-format".to_string(),
            "stream-json".to_string(),
            "--input-format".to_string(),
            "stream-json".to_string(),
            "--verbose".to_string(),
            // Enable token-level streaming — emits content_block_delta events
            // as tokens arrive instead of batching the full response.
            "--include-partial-messages".to_string(),
        ];

        if let Some(ref model) = config.model {
            args.push("--model".to_string());
            args.push(model.clone());
        }

        if let Some(max_tokens) = config.max_tokens {
            args.push("--max-tokens".to_string());
            args.push(max_tokens.to_string());
        }

        if !config.allowed_tools.is_empty() {
            args.push("--allowedTools".to_string());
            args.push(config.allowed_tools.join(" "));
        }

        if !config.denied_tools.is_empty() {
            args.push("--disallowedTools".to_string());
            args.push(config.denied_tools.join(" "));
        }

        // System prompt
        if let Some(ref prompt) = config.system_prompt {
            args.push("--system-prompt".to_string());
            args.push(prompt.clone());
        }

        // Permission mode → CLI flag mapping.
        // Parse PermissionMode from extra config; default to passing through raw value.
        if let Some(mode) = config.extra.get("permission_mode").and_then(|v| v.as_str()) {
            match mode {
                // AutoApprove: CIAB gates post-hoc via control_request/control_response,
                // so tell Claude Code to ask (default mode) — we auto-respond.
                "auto_approve" => {
                    args.push("--permission-mode".to_string());
                    args.push("default".to_string());
                }
                // ApproveEdits: only gate writes — tell Claude Code to use acceptEdits.
                "approve_edits" => {
                    args.push("--permission-mode".to_string());
                    args.push("acceptEdits".to_string());
                }
                // ApproveAll: gate everything — use default mode, CIAB gates all.
                "approve_all" => {
                    args.push("--permission-mode".to_string());
                    args.push("default".to_string());
                }
                // PlanOnly: use Claude Code's native plan mode.
                "plan_only" => {
                    args.push("--permission-mode".to_string());
                    args.push("plan".to_string());
                }
                // Unrestricted: bypass all permission checks entirely.
                "unrestricted" => {
                    args.push("--dangerously-skip-permissions".to_string());
                }
                // Unknown values: pass through as-is for forward compat.
                other => {
                    args.push("--permission-mode".to_string());
                    args.push(other.to_string());
                }
            }
        }

        if let Some(budget) = config.extra.get("max_budget_usd").and_then(|v| v.as_f64()) {
            args.push("--max-budget-usd".to_string());
            args.push(budget.to_string());
        }

        if let Some(effort) = config.extra.get("effort").and_then(|v| v.as_str()) {
            args.push("--effort".to_string());
            args.push(effort.to_string());
        }

        // Legacy: still honour explicit dangerously_skip_permissions if no
        // permission_mode was set (backwards compat).
        if config.extra.get("permission_mode").is_none()
            && config
                .extra
                .get("dangerously_skip_permissions")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
        {
            args.push("--dangerously-skip-permissions".to_string());
        }

        if config
            .extra
            .get("no_session_persistence")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            args.push("--no-session-persistence".to_string());
        }

        // MCP server configs
        for mcp in &config.mcp_servers {
            let mcp_json = json!({
                "mcpServers": {
                    &mcp.name: {
                        "command": &mcp.command,
                        "args": &mcp.args,
                        "env": &mcp.env,
                    }
                }
            });
            args.push("--mcp-config".to_string());
            args.push(mcp_json.to_string());
        }

        // Setting sources — enables skills when the session has skills active.
        if let Some(sources) = config.extra.get("setting_sources").and_then(|v| v.as_str()) {
            args.push("--setting-sources".to_string());
            args.push(sources.to_string());
        }

        // Resume a previous session
        if let Some(session_id) = config
            .extra
            .get("resume_session_id")
            .and_then(|v| v.as_str())
        {
            args.push("--resume".to_string());
            args.push(session_id.to_string());
        }

        // Continue last conversation
        if config
            .extra
            .get("continue_session")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            args.push("--continue".to_string());
        }

        // CIAB hooks integration — configure Claude Code HTTP hooks pointing to CIAB's hook endpoint.
        // This enables CIAB to auto-approve/deny tool calls via HTTP before they reach the
        // control_request/control_response stdin protocol.
        //
        // Claude Code reads hooks from project-level settings files at
        // `<workdir>/.claude/settings.local.json`. We write this file so the hooks are
        // active for the agent process. The hook URL points to CIAB's HTTP endpoint which
        // checks the session's permission policy and returns allow/deny/ask decisions.
        let mut env: std::collections::HashMap<String, String> = Default::default();
        if let Some(hook_url) = config.extra.get("ciab_hook_url").and_then(|v| v.as_str()) {
            // Store the hook URL as an env var so the session handler can reference it.
            env.insert("CIAB_HOOK_URL".to_string(), hook_url.to_string());

            // Build the hooks settings JSON for Claude Code's project settings.
            let hooks_settings = serde_json::json!({
                "hooks": {
                    "PreToolUse": [{
                        "type": "http",
                        "url": hook_url,
                        "timeout": 30000
                    }],
                    "PostToolUse": [{
                        "type": "http",
                        "url": hook_url,
                        "timeout": 10000
                    }],
                    "Stop": [{
                        "type": "http",
                        "url": hook_url,
                        "timeout": 10000
                    }]
                }
            });
            env.insert(
                "CIAB_HOOKS_SETTINGS".to_string(),
                hooks_settings.to_string(),
            );
        }

        AgentCommand {
            command: "claude".to_string(),
            args,
            env,
            workdir: None,
        }
    }

    fn prompt_mode(&self) -> PromptMode {
        PromptMode::StdinJson
    }

    fn interactive_protocol(&self) -> InteractiveProtocol {
        InteractiveProtocol::ControlRequestResponse
    }

    fn required_env_vars(&self) -> Vec<String> {
        vec!["ANTHROPIC_API_KEY".to_string()]
    }

    /// Parse Claude Code `--output-format stream-json --verbose` NDJSON output.
    ///
    /// Each line is a JSON object with a top-level `"type"` field:
    ///
    /// - `"system"` (subtype: `"init"`) — session init with tools, model, etc.
    /// - `"assistant"` — model response, `message.content[]` has `text` or `tool_use` blocks
    /// - `"user"` — tool results fed back, `message.content[]` has `tool_result` blocks
    /// - `"rate_limit_event"` — rate limit status
    /// - `"result"` — final result with `result` text, usage, cost
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
                                    "tools": obj.get("tools"),
                                    "cwd": obj.get("cwd"),
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
                                data: json!({
                                    "subtype": subtype,
                                    "data": obj,
                                }),
                                timestamp: Utc::now(),
                            });
                        }
                    }
                }

                "assistant" => {
                    // Extract content blocks from message.content[]
                    // Note: With --include-partial-messages enabled, text content
                    // has already been streamed via stream_event/content_block_delta.
                    // We skip emitting TextDelta here to avoid duplicating text on
                    // the frontend. Tool use and other block types are still extracted.
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
                                    // Intentionally skipped — text is streamed via
                                    // stream_event content_block_delta events.
                                }
                                "tool_use" => {
                                    // With --include-partial-messages, tool_use blocks are
                                    // already streamed in real-time via stream_event
                                    // content_block_start + input_json_delta events.
                                    // The assistant message's tool_use blocks are the
                                    // finalized versions — skip re-emitting ToolUseStart
                                    // to avoid duplicates on the frontend.
                                    //
                                    // AskUserQuestion is special — it may also come via
                                    // control_request, but we handle it here as a fallback.
                                    let name =
                                        block.get("name").and_then(|n| n.as_str()).unwrap_or("");
                                    if name == "AskUserQuestion" {
                                        events.push(StreamEvent {
                                            id: Uuid::new_v4().to_string(),
                                            sandbox_id: *sandbox_id,
                                            session_id: None,
                                            event_type: StreamEventType::UserInputRequest,
                                            data: json!({
                                                "tool_use_id": block.get("id"),
                                                "questions": block.get("input").and_then(|i| i.get("questions")),
                                            }),
                                            timestamp: Utc::now(),
                                        });
                                    }
                                    // Other tool_use blocks: already streamed via content_block_start
                                }
                                "thinking" => {
                                    if let Some(text) =
                                        block.get("thinking").and_then(|t| t.as_str())
                                    {
                                        events.push(StreamEvent {
                                            id: Uuid::new_v4().to_string(),
                                            sandbox_id: *sandbox_id,
                                            session_id: None,
                                            event_type: StreamEventType::ThinkingDelta,
                                            data: json!({ "text": text }),
                                            timestamp: Utc::now(),
                                        });
                                    }
                                }
                                "server_tool_use" => {
                                    let name = block
                                        .get("name")
                                        .and_then(|n| n.as_str())
                                        .unwrap_or("subagent");
                                    events.push(StreamEvent {
                                        id: Uuid::new_v4().to_string(),
                                        sandbox_id: *sandbox_id,
                                        session_id: None,
                                        event_type: StreamEventType::SubagentStart,
                                        data: json!({
                                            "id": block.get("id"),
                                            "name": name,
                                            "input": block.get("input"),
                                        }),
                                        timestamp: Utc::now(),
                                    });
                                }
                                _ => {}
                            }
                        }
                    }
                }

                "user" => {
                    // Tool results
                    if let Some(content) = obj
                        .get("message")
                        .and_then(|m| m.get("content"))
                        .and_then(|c| c.as_array())
                    {
                        for block in content {
                            let block_type =
                                block.get("type").and_then(|t| t.as_str()).unwrap_or("");
                            if block_type == "tool_result" {
                                let tool_use_id = block
                                    .get("tool_use_id")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("");
                                let content_str =
                                    block.get("content").and_then(|v| v.as_str()).unwrap_or("");
                                let is_error = block
                                    .get("is_error")
                                    .and_then(|v| v.as_bool())
                                    .unwrap_or(false);

                                // Also capture tool output from sibling data
                                let stdout = obj
                                    .get("tool_use_result")
                                    .and_then(|r| r.get("stdout"))
                                    .and_then(|v| v.as_str())
                                    .unwrap_or(content_str);

                                events.push(StreamEvent {
                                    id: Uuid::new_v4().to_string(),
                                    sandbox_id: *sandbox_id,
                                    session_id: None,
                                    event_type: StreamEventType::ToolResult,
                                    data: json!({
                                        "tool_use_id": tool_use_id,
                                        "content": stdout,
                                        "is_error": is_error,
                                    }),
                                    timestamp: Utc::now(),
                                });
                            }
                        }
                    }
                }

                "tool_progress" => {
                    events.push(StreamEvent {
                        id: Uuid::new_v4().to_string(),
                        sandbox_id: *sandbox_id,
                        session_id: None,
                        event_type: StreamEventType::ToolProgress,
                        data: json!({
                            "tool_use_id": obj.get("tool_use_id"),
                            "progress": obj.get("progress"),
                        }),
                        timestamp: Utc::now(),
                    });
                }

                "result" => {
                    let subtype = obj
                        .get("subtype")
                        .and_then(|s| s.as_str())
                        .unwrap_or("success");
                    match subtype {
                        "error_max_turns" | "error_during_execution" | "error_max_budget_usd" => {
                            events.push(StreamEvent {
                                id: Uuid::new_v4().to_string(),
                                sandbox_id: *sandbox_id,
                                session_id: None,
                                event_type: StreamEventType::ResultError,
                                data: json!({
                                    "error_type": subtype,
                                    "message": obj.get("result"),
                                    "cost_usd": obj.get("total_cost_usd"),
                                    "duration_ms": obj.get("duration_ms"),
                                    "session_id": obj.get("session_id"),
                                }),
                                timestamp: Utc::now(),
                            });
                        }
                        _ => {
                            if let Some(text) = obj.get("result").and_then(|r| r.as_str()) {
                                events.push(StreamEvent {
                                    id: Uuid::new_v4().to_string(),
                                    sandbox_id: *sandbox_id,
                                    session_id: None,
                                    event_type: StreamEventType::TextComplete,
                                    data: json!({
                                        "text": text,
                                        "cost_usd": obj.get("total_cost_usd"),
                                        "duration_ms": obj.get("duration_ms"),
                                        "num_turns": obj.get("num_turns"),
                                        "session_id": obj.get("session_id"),
                                        "stop_reason": obj.get("stop_reason"),
                                        "usage": obj.get("usage"),
                                    }),
                                    timestamp: Utc::now(),
                                });
                            }
                        }
                    }

                    // The "result" message signals the agent is done — emit
                    // SessionCompleted so the send_message loop knows to stop.
                    events.push(StreamEvent {
                        id: Uuid::new_v4().to_string(),
                        sandbox_id: *sandbox_id,
                        session_id: None,
                        event_type: StreamEventType::SessionCompleted,
                        data: json!({
                            "cost_usd": obj.get("total_cost_usd"),
                            "duration_ms": obj.get("duration_ms"),
                            "session_id": obj.get("session_id"),
                        }),
                        timestamp: Utc::now(),
                    });
                }

                "rate_limit_event" => {
                    events.push(StreamEvent {
                        id: Uuid::new_v4().to_string(),
                        sandbox_id: *sandbox_id,
                        session_id: None,
                        event_type: StreamEventType::LogLine,
                        data: json!({
                            "rate_limit": obj.get("rate_limit_info"),
                        }),
                        timestamp: Utc::now(),
                    });
                }

                "stream_event" => {
                    // Token-level streaming events from --include-partial-messages.
                    // These give us real-time content as it's generated.
                    if let Some(event) = obj.get("event") {
                        let event_type = event.get("type").and_then(|t| t.as_str()).unwrap_or("");
                        match event_type {
                            "content_block_start" => {
                                // When a new content block starts, check if it's a tool_use
                                // block and emit ToolUseStart immediately for real-time UI.
                                if let Some(content_block) = event.get("content_block") {
                                    let block_type = content_block
                                        .get("type")
                                        .and_then(|t| t.as_str())
                                        .unwrap_or("");
                                    if block_type == "tool_use" {
                                        let name = content_block
                                            .get("name")
                                            .and_then(|n| n.as_str())
                                            .unwrap_or("");
                                        if name == "AskUserQuestion" {
                                            // Will be handled by the full assistant message
                                        } else {
                                            events.push(StreamEvent {
                                                id: Uuid::new_v4().to_string(),
                                                sandbox_id: *sandbox_id,
                                                session_id: None,
                                                event_type: StreamEventType::ToolUseStart,
                                                data: json!({
                                                    "id": content_block.get("id"),
                                                    "name": name,
                                                    "input": {},
                                                    "streaming": true,
                                                }),
                                                timestamp: Utc::now(),
                                            });
                                        }
                                    }
                                }
                            }
                            "content_block_delta" => {
                                if let Some(delta) = event.get("delta") {
                                    let delta_type =
                                        delta.get("type").and_then(|t| t.as_str()).unwrap_or("");
                                    match delta_type {
                                        "text_delta" => {
                                            if let Some(text) =
                                                delta.get("text").and_then(|t| t.as_str())
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
                                        "thinking_delta" => {
                                            if let Some(text) =
                                                delta.get("thinking").and_then(|t| t.as_str())
                                            {
                                                events.push(StreamEvent {
                                                    id: Uuid::new_v4().to_string(),
                                                    sandbox_id: *sandbox_id,
                                                    session_id: None,
                                                    event_type: StreamEventType::ThinkingDelta,
                                                    data: json!({ "text": text }),
                                                    timestamp: Utc::now(),
                                                });
                                            }
                                        }
                                        "input_json_delta" => {
                                            // Incremental tool input JSON — stream to frontend
                                            // so tool blocks can show partial input as it arrives.
                                            if let Some(json_str) =
                                                delta.get("partial_json").and_then(|t| t.as_str())
                                            {
                                                events.push(StreamEvent {
                                                    id: Uuid::new_v4().to_string(),
                                                    sandbox_id: *sandbox_id,
                                                    session_id: None,
                                                    event_type: StreamEventType::ToolInputDelta,
                                                    data: json!({
                                                        "partial_json": json_str,
                                                        "index": event.get("index"),
                                                    }),
                                                    timestamp: Utc::now(),
                                                });
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            // message_start, content_block_stop, message_delta, message_stop
                            // are structural — we don't need to emit UI events for these.
                            _ => {}
                        }
                    }
                }

                "control_request" => {
                    let request_id = obj.get("request_id").and_then(|v| v.as_str()).unwrap_or("");
                    let request = obj.get("request").cloned().unwrap_or(json!({}));
                    let subtype = request
                        .get("subtype")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");

                    match subtype {
                        "can_use_tool" => {
                            let tool_name = request
                                .get("tool_name")
                                .and_then(|v| v.as_str())
                                .unwrap_or("");

                            if tool_name == "AskUserQuestion" {
                                // Interactive question — extract questions from input.
                                events.push(StreamEvent {
                                    id: Uuid::new_v4().to_string(),
                                    sandbox_id: *sandbox_id,
                                    session_id: None,
                                    event_type: StreamEventType::UserInputRequest,
                                    data: json!({
                                        "request_id": request_id,
                                        "tool_use_id": request_id,
                                        "questions": request.get("input")
                                            .and_then(|i| i.get("questions")),
                                    }),
                                    timestamp: Utc::now(),
                                });
                            } else {
                                // Tool permission request — emit as PermissionRequest for CIAB gate.
                                events.push(StreamEvent {
                                    id: Uuid::new_v4().to_string(),
                                    sandbox_id: *sandbox_id,
                                    session_id: None,
                                    event_type: StreamEventType::PermissionRequest,
                                    data: json!({
                                        "request_id": request_id,
                                        "tool_name": tool_name,
                                        "tool_input": request.get("input"),
                                        "risk_level": ciab_core::types::agent::PermissionPolicy::risk_level(tool_name),
                                    }),
                                    timestamp: Utc::now(),
                                });
                            }
                        }
                        _ => {
                            // Other control requests (hook callbacks, etc.)
                            events.push(StreamEvent {
                                id: Uuid::new_v4().to_string(),
                                sandbox_id: *sandbox_id,
                                session_id: None,
                                event_type: StreamEventType::LogLine,
                                data: json!({
                                    "control_request": subtype,
                                    "request_id": request_id,
                                    "data": request,
                                }),
                                timestamp: Utc::now(),
                            });
                        }
                    }
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
        if config.provider != "claude-code" {
            return Err(CiabError::ConfigValidationError(format!(
                "expected provider 'claude-code', got '{}'",
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
            status: "stub".into(),
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
                name: "compact".into(),
                description: "Compact conversation context".into(),
                category: SlashCommandCategory::Session,
                args: vec![],
                provider_native: true,
            },
            SlashCommand {
                name: "cost".into(),
                description: "Show token/cost usage".into(),
                category: SlashCommandCategory::Session,
                args: vec![],
                provider_native: true,
            },
            SlashCommand {
                name: "status".into(),
                description: "Show status".into(),
                category: SlashCommandCategory::Session,
                args: vec![],
                provider_native: true,
            },
            SlashCommand {
                name: "help".into(),
                description: "Show available commands".into(),
                category: SlashCommandCategory::Help,
                args: vec![],
                provider_native: false,
            },
            SlashCommand {
                name: "bug".into(),
                description: "Report a bug".into(),
                category: SlashCommandCategory::Help,
                args: vec![],
                provider_native: true,
            },
            SlashCommand {
                name: "doctor".into(),
                description: "Check installation health".into(),
                category: SlashCommandCategory::Help,
                args: vec![],
                provider_native: true,
            },
            SlashCommand {
                name: "init".into(),
                description: "Initialize project CLAUDE.md".into(),
                category: SlashCommandCategory::Agent,
                args: vec![],
                provider_native: true,
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
                name: "config".into(),
                description: "View/edit configuration".into(),
                category: SlashCommandCategory::Agent,
                args: vec![],
                provider_native: true,
            },
            SlashCommand {
                name: "skills".into(),
                description: "Browse and attach Agent Skills to this session".into(),
                category: SlashCommandCategory::Tools,
                args: vec![],
                provider_native: false,
            },
            SlashCommand {
                name: "login".into(),
                description: "Switch Anthropic account".into(),
                category: SlashCommandCategory::Agent,
                args: vec![],
                provider_native: true,
            },
            SlashCommand {
                name: "logout".into(),
                description: "Sign out".into(),
                category: SlashCommandCategory::Agent,
                args: vec![],
                provider_native: true,
            },
            SlashCommand {
                name: "memory".into(),
                description: "Edit CLAUDE.md memory".into(),
                category: SlashCommandCategory::Agent,
                args: vec![],
                provider_native: true,
            },
            SlashCommand {
                name: "review".into(),
                description: "Review code changes".into(),
                category: SlashCommandCategory::Agent,
                args: vec![],
                provider_native: true,
            },
            SlashCommand {
                name: "pr-comments".into(),
                description: "View PR comments".into(),
                category: SlashCommandCategory::Agent,
                args: vec![],
                provider_native: true,
            },
            SlashCommand {
                name: "terminal-setup".into(),
                description: "Install Shift+Enter key binding".into(),
                category: SlashCommandCategory::Agent,
                args: vec![],
                provider_native: true,
            },
            SlashCommand {
                name: "permissions".into(),
                description: "View/set permission mode".into(),
                category: SlashCommandCategory::Tools,
                args: vec![],
                provider_native: true,
            },
            SlashCommand {
                name: "allowed-tools".into(),
                description: "Manage allowed tools".into(),
                category: SlashCommandCategory::Tools,
                args: vec![],
                provider_native: true,
            },
            SlashCommand {
                name: "mcp".into(),
                description: "Manage MCP servers".into(),
                category: SlashCommandCategory::Tools,
                args: vec![],
                provider_native: true,
            },
            SlashCommand {
                name: "vim".into(),
                description: "Toggle vim mode".into(),
                category: SlashCommandCategory::Navigation,
                args: vec![],
                provider_native: true,
            },
        ]
    }
}
