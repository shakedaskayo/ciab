use std::collections::HashMap;
use std::convert::Infallible;

use axum::extract::{Path, Query, State};
use axum::response::sse::{Event, Sse};
use axum::response::IntoResponse;
use axum::Json;
use chrono::Utc;
use ciab_core::error::CiabError;
use ciab_core::types::llm_provider::LlmProviderKind;
use ciab_core::types::sandbox::ExecRequest;
use ciab_core::types::session::{Message, MessageContent, MessageRole, Session, SessionState};
use ciab_core::types::stream::{StreamEvent, StreamEventType};
use futures::stream::Stream;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::state::AppState;

// ---------------------------------------------------------------------------
// create_session
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

pub async fn create_session(
    State(state): State<AppState>,
    Path(sandbox_id): Path<Uuid>,
    Json(body): Json<CreateSessionRequest>,
) -> Result<impl IntoResponse, CiabError> {
    // Verify sandbox exists.
    let _sandbox = state
        .db
        .get_sandbox(&sandbox_id)
        .await?
        .ok_or_else(|| CiabError::SandboxNotFound(sandbox_id.to_string()))?;

    let now = Utc::now();
    let session = Session {
        id: Uuid::new_v4(),
        sandbox_id,
        state: SessionState::Active,
        metadata: body.metadata,
        created_at: now,
        updated_at: now,
    };

    state.db.insert_session(&session).await?;
    Ok(Json(session))
}

// ---------------------------------------------------------------------------
// list_sessions
// ---------------------------------------------------------------------------

pub async fn list_sessions(
    State(state): State<AppState>,
    Path(sandbox_id): Path<Uuid>,
) -> Result<impl IntoResponse, CiabError> {
    let sessions = state.db.list_sessions(&sandbox_id).await?;
    Ok(Json(sessions))
}

// ---------------------------------------------------------------------------
// get_session — returns session + messages as SessionWithMessages shape
// ---------------------------------------------------------------------------

pub async fn get_session(
    State(state): State<AppState>,
    Path(sid): Path<Uuid>,
) -> Result<impl IntoResponse, CiabError> {
    let session = state
        .db
        .get_session(&sid)
        .await?
        .ok_or_else(|| CiabError::SessionNotFound(sid.to_string()))?;
    let messages = state.db.get_messages(&sid, None).await?;

    Ok(Json(json!({
        "id": session.id,
        "sandbox_id": session.sandbox_id,
        "state": session.state,
        "metadata": session.metadata,
        "created_at": session.created_at,
        "updated_at": session.updated_at,
        "messages": messages,
    })))
}

// ---------------------------------------------------------------------------
// send_message
// ---------------------------------------------------------------------------

/// Accepts messages in two formats for compatibility:
/// 1. `{ "message": "text" }` — simple string (CLI)
/// 2. `{ "role": "user", "content": [{ "type": "text", "text": "..." }] }` — structured (desktop)
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum SendMessageRequest {
    Structured {
        role: MessageRole,
        content: Vec<MessageContent>,
    },
    Simple {
        message: String,
    },
}

impl SendMessageRequest {
    fn into_content(self) -> (MessageRole, Vec<MessageContent>) {
        match self {
            Self::Structured { role, content } => (role, content),
            Self::Simple { message } => (
                MessageRole::User,
                vec![MessageContent::Text { text: message }],
            ),
        }
    }

    /// Extract plain text from the message content.
    fn text(&self) -> String {
        match self {
            Self::Structured { content, .. } => content
                .iter()
                .filter_map(|c| match c {
                    MessageContent::Text { text } => Some(text.as_str()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("\n"),
            Self::Simple { message } => message.clone(),
        }
    }
}

pub async fn send_message(
    State(state): State<AppState>,
    Path(sid): Path<Uuid>,
    Json(body): Json<SendMessageRequest>,
) -> Result<Json<Message>, CiabError> {
    let session = state
        .db
        .get_session(&sid)
        .await?
        .ok_or_else(|| CiabError::SessionNotFound(sid.to_string()))?;

    let sandbox = state
        .db
        .get_sandbox(&session.sandbox_id)
        .await?
        .ok_or_else(|| CiabError::SandboxNotFound(session.sandbox_id.to_string()))?;

    let agent = state
        .agents
        .get(&sandbox.agent_provider)
        .ok_or_else(|| CiabError::AgentProviderNotFound(sandbox.agent_provider.clone()))?
        .clone();

    let prompt_text = body.text();

    // Intercept slash commands before sending to agent.
    if prompt_text.starts_with('/') {
        let cmd_name = prompt_text
            .split_whitespace()
            .next()
            .unwrap_or("")
            .trim_start_matches('/');
        let commands = agent.slash_commands();

        if let Some(cmd) = commands.iter().find(|c| c.name == cmd_name) {
            if !cmd.provider_native {
                return handle_local_command(&state, &sid, cmd_name, &commands).await;
            }
        }
    }

    let (role, content) = body.into_content();

    // Store user message in DB immediately.
    let user_msg = Message {
        id: Uuid::new_v4(),
        session_id: sid,
        role: role.clone(),
        content: content.clone(),
        timestamp: Utc::now(),
    };
    state.db.insert_message(&user_msg).await?;

    // Enqueue the message for processing. If no agent is currently running
    // for this session, start processing immediately.
    let should_start = {
        let mut queues = state.session_queues.write().await;
        let queue = queues.entry(sid).or_default();
        queue.messages.push_back(crate::state::QueuedMessage {
            id: user_msg.id,
            session_id: sid,
            role,
            content,
            prompt_text: prompt_text.clone(),
            queued_at: Utc::now(),
        });

        // Emit queue_updated event so frontend can show queued messages.
        let queue_positions: Vec<_> = queue
            .messages
            .iter()
            .map(|m| {
                json!({
                    "id": m.id,
                    "prompt_text": m.prompt_text,
                    "queued_at": m.queued_at,
                })
            })
            .collect();
        let _ = state
            .stream_handler
            .publish(StreamEvent {
                id: Uuid::new_v4().to_string(),
                sandbox_id: sandbox.id,
                session_id: Some(sid),
                event_type: StreamEventType::QueueUpdated,
                data: json!({
                    "queue": queue_positions,
                    "queue_length": queue.messages.len(),
                }),
                timestamp: Utc::now(),
            })
            .await;

        if !queue.processing {
            queue.processing = true;
            true
        } else {
            false
        }
    };

    if should_start {
        let bg_state = state.clone();
        tokio::spawn(async move {
            process_session_queue(bg_state, sid).await;
        });
    }

    Ok(Json(user_msg))
}

/// Process queued messages for a session, one at a time (FIFO).
/// Keeps running until the queue is empty, then marks processing as false.
async fn process_session_queue(state: AppState, sid: Uuid) {
    loop {
        // Dequeue the next message.
        let queued_msg = {
            let mut queues = state.session_queues.write().await;
            let queue = match queues.get_mut(&sid) {
                Some(q) => q,
                None => return,
            };
            match queue.messages.pop_front() {
                Some(msg) => msg,
                None => {
                    // Queue is empty — stop processing.
                    queue.processing = false;

                    // Emit queue_updated with empty queue.
                    let sandbox_id = state
                        .db
                        .get_session(&sid)
                        .await
                        .ok()
                        .flatten()
                        .map(|s| s.sandbox_id)
                        .unwrap_or(Uuid::nil());
                    let _ = state
                        .stream_handler
                        .publish(StreamEvent {
                            id: Uuid::new_v4().to_string(),
                            sandbox_id,
                            session_id: Some(sid),
                            event_type: StreamEventType::QueueUpdated,
                            data: json!({ "queue": [], "queue_length": 0 }),
                            timestamp: Utc::now(),
                        })
                        .await;

                    return;
                }
            }
        };

        // Process this message through the agent.
        if let Err(e) = process_queued_message(&state, sid, &queued_msg).await {
            tracing::error!("Error processing queued message {}: {}", queued_msg.id, e);
        }
    }
}

/// Process a single queued message: build agent config, spawn agent, wait for completion.
async fn process_queued_message(
    state: &AppState,
    sid: Uuid,
    queued_msg: &crate::state::QueuedMessage,
) -> Result<(), CiabError> {
    let session = state
        .db
        .get_session(&sid)
        .await?
        .ok_or_else(|| CiabError::SessionNotFound(sid.to_string()))?;
    let sandbox = state
        .db
        .get_sandbox(&session.sandbox_id)
        .await?
        .ok_or_else(|| CiabError::SandboxNotFound(session.sandbox_id.to_string()))?;
    let agent = state
        .agents
        .get(&sandbox.agent_provider)
        .ok_or_else(|| CiabError::AgentProviderNotFound(sandbox.agent_provider.clone()))?
        .clone();

    // Update session state to processing.
    state
        .db
        .update_session_state(&sid, &SessionState::Processing)
        .await?;

    // Build agent config (fresh — picks up latest session state).
    let mut config =
        sandbox
            .spec
            .agent_config
            .clone()
            .unwrap_or_else(|| ciab_core::types::agent::AgentConfig {
                provider: sandbox.agent_provider.clone(),
                ..Default::default()
            });

    // Per-session model/provider override stored in metadata at creation time.
    if let Some(model) = session
        .metadata
        .get("model_override")
        .and_then(|v| v.as_str())
    {
        if !model.is_empty() {
            config.model = Some(model.to_string());
        }
    }
    // Resolve LLM provider override: look up the provider from the DB and inject
    // llm_base_url + llm_api_key into config.extra so the agent provider can set
    // the correct env vars (e.g. ANTHROPIC_BASE_URL for Ollama).
    let effective_provider_id = session
        .metadata
        .get("llm_provider_id_override")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .or_else(|| {
            config
                .extra
                .get("llm_provider_id")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
        })
        .and_then(|s| Uuid::parse_str(s).ok());

    if let Some(provider_uuid) = effective_provider_id {
        match state.db.get_llm_provider(&provider_uuid).await {
            Ok(Some(provider)) => {
                let kind = &provider.kind;
                let base_url = provider
                    .base_url
                    .clone()
                    .unwrap_or_else(|| "http://localhost:11434".to_string());

                // Emit a log event so the sandbox log shows which LLM provider is active.
                let log_event = StreamEvent {
                    id: format!("llm-provider-{}", Uuid::new_v4()),
                    sandbox_id: sandbox.id,
                    session_id: Some(sid),
                    event_type: StreamEventType::LogLine,
                    data: json!({
                        "line": format!(
                            "[ciab] LLM provider override: {} ({:?}) @ {}",
                            provider.name, kind, base_url
                        ),
                        "stream": "stderr"
                    }),
                    timestamp: Utc::now(),
                };
                let _ = state.stream_handler.publish(log_event).await;

                match kind {
                    LlmProviderKind::Ollama => {
                        // Ollama uses its Anthropic-compatible API at the root URL (no /v1).
                        // ANTHROPIC_AUTH_TOKEN must be set; ANTHROPIC_API_KEY is cleared.
                        config.extra.insert(
                            "llm_base_url".to_string(),
                            serde_json::Value::String(base_url.clone()),
                        );
                        config.extra.insert(
                            "llm_auth_token".to_string(),
                            serde_json::Value::String("ollama".to_string()),
                        );
                        // Signal that ANTHROPIC_API_KEY validation should be skipped.
                        config.extra.insert(
                            "llm_skip_api_key".to_string(),
                            serde_json::Value::Bool(true),
                        );

                        // Validate Ollama connectivity and emit a log event.
                        let ollama_version_url =
                            format!("{}/api/version", base_url.trim_end_matches('/'));
                        let http = reqwest::Client::builder()
                            .timeout(std::time::Duration::from_secs(5))
                            .build()
                            .unwrap_or_default();
                        let connectivity_line = match http.get(&ollama_version_url).send().await {
                            Ok(resp) if resp.status().is_success() => {
                                let version = resp
                                    .json::<serde_json::Value>()
                                    .await
                                    .ok()
                                    .and_then(|v| {
                                        v.get("version")
                                            .and_then(|v| v.as_str())
                                            .map(|s| s.to_string())
                                    })
                                    .unwrap_or_else(|| "unknown".to_string());
                                format!("[ciab] Ollama connected — version {version} @ {base_url}")
                            }
                            Ok(resp) => {
                                format!(
                                    "[ciab] Ollama reachable but returned HTTP {} @ {base_url}",
                                    resp.status()
                                )
                            }
                            Err(e) => {
                                format!("[ciab] WARNING: Ollama not reachable @ {base_url} — {e}. Agent will likely fail.")
                            }
                        };
                        let conn_event = StreamEvent {
                            id: format!("ollama-conn-{}", Uuid::new_v4()),
                            sandbox_id: sandbox.id,
                            session_id: Some(sid),
                            event_type: StreamEventType::LogLine,
                            data: json!({ "line": connectivity_line, "stream": "stderr" }),
                            timestamp: Utc::now(),
                        };
                        let _ = state.stream_handler.publish(conn_event).await;
                    }
                    _ => {
                        // For non-Ollama providers, inject base_url and api_key generically.
                        if let Some(url) = &provider.base_url {
                            config.extra.insert(
                                "llm_base_url".to_string(),
                                serde_json::Value::String(url.clone()),
                            );
                        }
                        // API key will be resolved from credentials by the agent.
                    }
                }

                config.extra.insert(
                    "llm_provider_id".to_string(),
                    serde_json::Value::String(provider_uuid.to_string()),
                );
            }
            Ok(None) => {
                tracing::warn!(provider_id = %provider_uuid, "LLM provider not found in DB, using default");
            }
            Err(e) => {
                tracing::warn!(error = %e, "Failed to look up LLM provider, using default");
            }
        }
    }

    if let Some(claude_sid) = session
        .metadata
        .get("claude_session_id")
        .and_then(|v| v.as_str())
    {
        config.extra.insert(
            "resume_session_id".to_string(),
            serde_json::Value::String(claude_sid.to_string()),
        );
    }

    {
        let perms = state.session_permissions.read().await;
        let policy = perms.get(&sid).cloned().unwrap_or_default();
        let mode_str = serde_json::to_value(&policy.mode)
            .ok()
            .and_then(|v| v.as_str().map(|s| s.to_string()));
        if let Some(mode_str) = mode_str {
            config.extra.insert(
                "permission_mode".to_string(),
                serde_json::Value::String(mode_str),
            );
        }
        for tool in &policy.always_allow {
            if !config.allowed_tools.contains(tool) {
                config.allowed_tools.push(tool.clone());
            }
        }
    }

    if let Some(active_skills) = session.metadata.get("active_skills") {
        if let Some(arr) = active_skills.as_array() {
            if !arr.is_empty() {
                config.extra.insert(
                    "setting_sources".to_string(),
                    serde_json::Value::String("user,project".to_string()),
                );
                let skill_tool = "Skill".to_string();
                if !config.allowed_tools.contains(&skill_tool) {
                    config.allowed_tools.push(skill_tool);
                }
            }
        }
    }

    // For the claude-code provider in local mode: try to inherit the host's Claude
    // subscription OAuth token so users don't need a separate API key configured.
    // Only do this when no explicit LLM provider override is active.
    if sandbox.agent_provider == "claude-code"
        && !config.extra.contains_key("llm_base_url")
        && !config.extra.contains_key("llm_api_key")
        && !config.extra.contains_key("claude_oauth_token")
    {
        match read_host_claude_oauth_token() {
            HostClaudeAuth::ValidToken {
                token,
                subscription_type,
                expires_in_secs,
            } => {
                config.extra.insert(
                    "claude_oauth_token".to_string(),
                    serde_json::Value::String(token),
                );
                let sub = subscription_type.as_deref().unwrap_or("unknown");
                let log_line = if expires_in_secs < 600 {
                    format!("[ciab] Inherited Claude {sub} subscription token (expires in {}m — consider re-logging in Claude Code)", expires_in_secs / 60)
                } else {
                    format!(
                        "[ciab] Inherited Claude {sub} subscription token (expires in {}h{}m)",
                        expires_in_secs / 3600,
                        (expires_in_secs % 3600) / 60
                    )
                };
                let log_event = StreamEvent {
                    id: format!("claude-auth-{}", Uuid::new_v4()),
                    sandbox_id: sandbox.id,
                    session_id: Some(sid),
                    event_type: StreamEventType::LogLine,
                    data: json!({ "line": log_line, "stream": "stderr" }),
                    timestamp: Utc::now(),
                };
                let _ = state.stream_handler.publish(log_event).await;
            }
            HostClaudeAuth::Expired { subscription_type } => {
                let sub = subscription_type.as_deref().unwrap_or("unknown");
                let log_event = StreamEvent {
                    id: format!("claude-auth-{}", Uuid::new_v4()),
                    sandbox_id: sandbox.id,
                    session_id: Some(sid),
                    event_type: StreamEventType::LogLine,
                    data: json!({
                        "line": format!("[ciab] WARNING: Claude {sub} subscription token has expired. Run `claude` in a terminal and log in again, or add an ANTHROPIC_API_KEY provider in Settings."),
                        "stream": "stderr"
                    }),
                    timestamp: Utc::now(),
                };
                let _ = state.stream_handler.publish(log_event).await;
            }
            HostClaudeAuth::NotFound => {
                // Check if ANTHROPIC_API_KEY is set in the environment as a fallback.
                let has_env_key = std::env::var("ANTHROPIC_API_KEY")
                    .map(|k| !k.is_empty())
                    .unwrap_or(false);
                if !has_env_key {
                    let log_event = StreamEvent {
                        id: format!("claude-auth-{}", Uuid::new_v4()),
                        sandbox_id: sandbox.id,
                        session_id: Some(sid),
                        event_type: StreamEventType::LogLine,
                        data: json!({
                            "line": "[ciab] No Claude auth found. Log in via `claude` in a terminal (Claude subscription), or add an ANTHROPIC_API_KEY in Settings → LLM Providers.",
                            "stream": "stderr"
                        }),
                        timestamp: Utc::now(),
                    };
                    let _ = state.stream_handler.publish(log_event).await;
                }
            }
        }
    }

    // Inject hook configuration for Claude Code provider.
    // This tells the agent provider to configure HTTP hooks pointing to CIAB's hook endpoint.
    let server_port = state.config.server.port;
    let hook_url = format!(
        "http://127.0.0.1:{}/api/v1/hooks/claude/{}",
        server_port, sid
    );
    config.extra.insert(
        "ciab_hook_url".to_string(),
        serde_json::Value::String(hook_url),
    );
    config.extra.insert(
        "ciab_session_id".to_string(),
        serde_json::Value::String(sid.to_string()),
    );

    let prompt_mode = agent.prompt_mode();
    let interactive_protocol = agent.interactive_protocol();

    let agent_cmd = agent.build_start_command(&config);
    let mut cmd = vec![agent_cmd.command.clone()];
    cmd.extend(agent_cmd.args.clone());

    // For CliArgument prompt mode, append the prompt text as a positional arg.
    // This is how Cursor CLI, Gemini CLI, and Codex deliver prompts.
    if prompt_mode == ciab_core::types::agent::PromptMode::CliArgument {
        cmd.push(queued_msg.prompt_text.clone());
    }

    let mut exec_env = agent_cmd.env.clone();
    exec_env.remove("CLAUDECODE");

    // Write Claude Code hooks settings file to the sandbox workdir.
    // Claude Code reads `.claude/settings.local.json` from the project root (cwd).
    // The local runtime uses `<base_workdir>/<sandbox_id>` as the cwd when no
    // explicit workdir is set.
    if let Some(hooks_settings) = exec_env.remove("CIAB_HOOKS_SETTINGS") {
        let workdir = agent_cmd.workdir.clone().unwrap_or_else(|| {
            let base = state
                .config
                .runtime
                .local_workdir
                .as_deref()
                .unwrap_or("/tmp/ciab-sandboxes");
            format!("{}/{}", base, sandbox.id)
        });
        let claude_dir = std::path::Path::new(&workdir).join(".claude");
        let _ = std::fs::create_dir_all(&claude_dir);
        let settings_path = claude_dir.join("settings.local.json");
        if let Err(e) = std::fs::write(&settings_path, &hooks_settings) {
            tracing::warn!(
                "Failed to write Claude hooks settings to {}: {}",
                settings_path.display(),
                e
            );
        } else {
            tracing::debug!("Wrote Claude hooks settings to {}", settings_path.display());
        }
    }

    let exec_req = ExecRequest {
        command: cmd,
        workdir: agent_cmd.workdir.clone(),
        env: exec_env,
        stdin: None,
        timeout_secs: Some(300),
        tty: false,
    };

    let policy = {
        let perms = state.session_permissions.read().await;
        perms.get(&sid).cloned().unwrap_or_default()
    };

    run_agent_background(
        state.clone(),
        state.runtime.clone(),
        state.stream_handler.clone(),
        state.db.clone(),
        agent,
        sandbox.id,
        sid,
        queued_msg.prompt_text.clone(),
        exec_req,
        policy,
        session.metadata.clone(),
        prompt_mode,
        interactive_protocol,
    )
    .await;

    Ok(())
}

/// Background agent execution — processes agent output, handles permissions,
/// and persists the assistant message when done.
#[allow(clippy::too_many_arguments)]
async fn run_agent_background(
    state: AppState,
    runtime: std::sync::Arc<dyn ciab_core::traits::runtime::SandboxRuntime>,
    stream_handler: std::sync::Arc<dyn ciab_core::traits::stream::StreamHandler>,
    db: std::sync::Arc<ciab_db::Database>,
    agent: std::sync::Arc<dyn ciab_core::traits::agent::AgentProvider>,
    sandbox_id: Uuid,
    sid: Uuid,
    prompt_text: String,
    exec_req: ExecRequest,
    policy: ciab_core::types::agent::PermissionPolicy,
    session_metadata: HashMap<String, serde_json::Value>,
    prompt_mode: ciab_core::types::agent::PromptMode,
    _interactive_protocol: ciab_core::types::agent::InteractiveProtocol,
) {
    let mut assistant_content = Vec::new();
    let mut text_parts = Vec::new();
    let mut assistant_text;
    let mut interrupted_by_denial = false;
    let mut captured_claude_session_id: Option<String> = None;
    let mut deferred_completion_event: Option<StreamEvent> = None;
    let mut current_subagent_name: Option<String> = None;

    match runtime
        .exec_streaming_interactive(&sandbox_id, &exec_req)
        .await
    {
        Ok((mut line_rx, stdin_tx, join_handle)) => {
            // Wrap stdin_tx in Option so we can drop it when the agent signals completion.
            let mut stdin_tx = Some(stdin_tx);

            // Send the user prompt via stdin based on the provider's prompt mode.
            // - StdinJson: Claude Code's stream-json protocol (JSON object on stdin).
            // - StdinPlaintext: Plain text line on stdin.
            // - CliArgument: Already appended to command args — no stdin needed.
            match prompt_mode {
                ciab_core::types::agent::PromptMode::StdinJson => {
                    let user_input = json!({
                        "type": "user",
                        "message": {
                            "role": "user",
                            "content": prompt_text,
                        }
                    });
                    if let Some(ref tx) = stdin_tx {
                        let _ = tx.send(user_input.to_string()).await;
                    }
                }
                ciab_core::types::agent::PromptMode::StdinPlaintext => {
                    if let Some(ref tx) = stdin_tx {
                        let _ = tx.send(prompt_text.clone()).await;
                    }
                }
                ciab_core::types::agent::PromptMode::CliArgument => {
                    // Prompt already in command args — nothing to send.
                    // Close stdin immediately so agent doesn't hang waiting for input.
                    stdin_tx.take();
                }
            }

            while let Some(line) = line_rx.recv().await {
                if interrupted_by_denial {
                    break;
                }

                // Emit a LogLine event for every raw output line so the
                // Logs tab can display real-time process output.
                let log_event = StreamEvent {
                    id: Uuid::new_v4().to_string(),
                    sandbox_id,
                    session_id: Some(sid),
                    event_type: StreamEventType::LogLine,
                    data: json!({
                        "line": &line,
                        "stream": "stdout",
                    }),
                    timestamp: Utc::now(),
                };
                let _ = stream_handler.publish(log_event).await;

                let events = agent.parse_output(&sandbox_id, &line);
                for mut event in events {
                    if interrupted_by_denial {
                        break;
                    }

                    event.session_id = Some(sid);

                    match &event.event_type {
                        StreamEventType::Connected => {
                            if let Some(claude_sid) =
                                event.data.get("session_id").and_then(|v| v.as_str())
                            {
                                captured_claude_session_id = Some(claude_sid.to_string());
                            }
                        }
                        StreamEventType::TextDelta => {
                            if let Some(text) = event.data.get("text").and_then(|v| v.as_str()) {
                                text_parts.push(text.to_string());
                            }
                        }
                        StreamEventType::TextComplete => {
                            if let Some(text) = event.data.get("text").and_then(|v| v.as_str()) {
                                text_parts.clear();
                                text_parts.push(text.to_string());
                            }
                            if let Some(claude_sid) =
                                event.data.get("session_id").and_then(|v| v.as_str())
                            {
                                captured_claude_session_id = Some(claude_sid.to_string());
                            }
                        }
                        StreamEventType::ToolUseStart => {
                            // Accumulate tool use content.
                            // With streaming, input starts empty and builds up via ToolInputDelta.
                            if let (Some(id), Some(name)) = (
                                event.data.get("id").and_then(|v| v.as_str()),
                                event.data.get("name").and_then(|v| v.as_str()),
                            ) {
                                let input = event.data.get("input").cloned().unwrap_or(json!({}));
                                assistant_content.push(MessageContent::ToolUse {
                                    id: id.to_string(),
                                    name: name.to_string(),
                                    input,
                                    agent_name: current_subagent_name.clone(),
                                });
                            }
                        }
                        StreamEventType::ToolInputDelta => {
                            // Accumulate partial JSON for the last tool_use block's input.
                            if let Some(partial) =
                                event.data.get("partial_json").and_then(|v| v.as_str())
                            {
                                // Find the last ToolUse in assistant_content and try to update its input
                                // once we have a complete JSON string.
                                if let Some(MessageContent::ToolUse { ref mut input, .. }) =
                                    assistant_content
                                        .iter_mut()
                                        .rev()
                                        .find(|c| matches!(c, MessageContent::ToolUse { .. }))
                                {
                                    // Append to a running string buffer
                                    let current = if let Some(buf) =
                                        input.get("__partial_buf").and_then(|v| v.as_str())
                                    {
                                        format!("{}{}", buf, partial)
                                    } else {
                                        partial.to_string()
                                    };
                                    // Try to parse the accumulated JSON
                                    if let Ok(parsed) =
                                        serde_json::from_str::<serde_json::Value>(&current)
                                    {
                                        *input = parsed;
                                    } else {
                                        // Store partial buffer for next delta
                                        *input = json!({"__partial_buf": current});
                                    }
                                }
                            }
                        }
                        StreamEventType::PermissionRequest => {
                            let request_id = event
                                .data
                                .get("request_id")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            let tool_name = event
                                .data
                                .get("tool_name")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();

                            if !policy.requires_approval(&tool_name) {
                                // Auto-approve: send control_response immediately via stdin.
                                let response = json!({
                                    "type": "control_response",
                                    "request_id": request_id,
                                    "response": {
                                        "subtype": "success",
                                        "response": {"behavior": "allow"}
                                    }
                                });
                                if let Some(ref tx) = stdin_tx {
                                    let _ = tx.send(response.to_string()).await;
                                }
                                // Don't publish permission_request to frontend — it was auto-approved.
                                continue;
                            }

                            // Needs user approval — publish event and wait.
                            let _ = stream_handler.publish(event.clone()).await;

                            // Update session state to WaitingForInput.
                            let _ = db
                                .update_session_state(&sid, &SessionState::WaitingForInput)
                                .await;

                            // Create oneshot channel and store in pending_permissions.
                            let (perm_tx, perm_rx) = tokio::sync::oneshot::channel::<bool>();
                            {
                                let mut pending = state.pending_permissions.write().await;
                                pending.insert(
                                    request_id.clone(),
                                    crate::state::PendingPermission { tx: perm_tx },
                                );
                            }

                            // Await user response with 5-minute timeout.
                            let approved =
                                tokio::time::timeout(std::time::Duration::from_secs(300), perm_rx)
                                    .await
                                    .unwrap_or(Ok(false))
                                    .unwrap_or(false);

                            // Restore session state to Processing.
                            let _ = db
                                .update_session_state(&sid, &SessionState::Processing)
                                .await;

                            // Send control_response via stdin.
                            let response = if approved {
                                json!({
                                    "type": "control_response",
                                    "request_id": request_id,
                                    "response": {
                                        "subtype": "success",
                                        "response": {"behavior": "allow"}
                                    }
                                })
                            } else {
                                json!({
                                    "type": "control_response",
                                    "request_id": request_id,
                                    "response": {
                                        "subtype": "success",
                                        "response": {"behavior": "deny", "message": "User denied this tool"}
                                    }
                                })
                            };
                            if let Some(ref tx) = stdin_tx {
                                let _ = tx.send(response.to_string()).await;
                            }

                            // Emit permission_response event.
                            let resp_event = StreamEvent {
                                id: Uuid::new_v4().to_string(),
                                sandbox_id,
                                session_id: Some(sid),
                                event_type: StreamEventType::PermissionResponse,
                                data: json!({
                                    "request_id": request_id,
                                    "tool_name": tool_name,
                                    "approved": approved,
                                }),
                                timestamp: Utc::now(),
                            };
                            let _ = stream_handler.publish(resp_event).await;

                            if !approved {
                                interrupted_by_denial = true;
                            }
                            continue; // Don't re-publish the original PermissionRequest event.
                        }
                        StreamEventType::UserInputRequest => {
                            // Publish the question to the frontend.
                            let _ = stream_handler.publish(event.clone()).await;

                            let request_id = event
                                .data
                                .get("request_id")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();

                            // Update session state to WaitingForInput.
                            let _ = db
                                .update_session_state(&sid, &SessionState::WaitingForInput)
                                .await;

                            // Store the stdin_tx so the user_input response handler can use it.
                            let (answer_tx, answer_rx) = tokio::sync::oneshot::channel::<String>();
                            {
                                let mut pending = state.pending_user_inputs.write().await;
                                pending.insert(
                                    request_id.clone(),
                                    crate::state::PendingUserInput { tx: answer_tx },
                                );
                            }

                            // Wait for user's answer (10-minute timeout for questions).
                            let answer = tokio::time::timeout(
                                std::time::Duration::from_secs(600),
                                answer_rx,
                            )
                            .await
                            .unwrap_or(Ok("(no answer)".to_string()))
                            .unwrap_or_else(|_| "(no answer)".to_string());

                            // Restore session state to Processing.
                            let _ = db
                                .update_session_state(&sid, &SessionState::Processing)
                                .await;

                            // Send control_response with the answer via stdin.
                            let response = json!({
                                "type": "control_response",
                                "request_id": request_id,
                                "response": {
                                    "subtype": "success",
                                    "response": {"behavior": "allow", "updatedInput": {"answer": answer}}
                                }
                            });
                            if let Some(ref tx) = stdin_tx {
                                let _ = tx.send(response.to_string()).await;
                            }

                            continue; // Don't re-publish.
                        }
                        StreamEventType::SessionCompleted | StreamEventType::ResultError => {
                            // Agent is done — defer publishing session_completed until
                            // after we persist the assistant message to DB. This prevents
                            // the frontend from refetching before the message exists.
                            deferred_completion_event = Some(event);
                            stdin_tx.take(); // drop sender → closes stdin → process exits
                            interrupted_by_denial = true; // reuse flag to break outer loop
                            continue;
                        }
                        StreamEventType::ToolResult => {
                            if let Some(tool_use_id) =
                                event.data.get("tool_use_id").and_then(|v| v.as_str())
                            {
                                let content = event
                                    .data
                                    .get("content")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string();
                                let is_error = event
                                    .data
                                    .get("is_error")
                                    .and_then(|v| v.as_bool())
                                    .unwrap_or(false);
                                assistant_content.push(MessageContent::ToolResult {
                                    tool_use_id: tool_use_id.to_string(),
                                    content,
                                    is_error,
                                });
                            }
                        }
                        StreamEventType::SubagentStart => {
                            current_subagent_name = event
                                .data
                                .get("name")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string());
                        }
                        StreamEventType::SubagentEnd => {
                            current_subagent_name = None;
                        }
                        _ => {}
                    }

                    let _ = stream_handler.publish(event).await;
                }
            }

            // Ensure stdin is closed so the agent process gets EOF and exits.
            drop(stdin_tx);

            // Wait for the process to finish.
            match join_handle.await {
                Ok(Ok(result)) => {
                    assistant_text = text_parts.join("");
                    if result.exit_code != 0 && assistant_text.is_empty() {
                        assistant_content.push(MessageContent::Text {
                            text: format!(
                                "Agent exited with code {}.\n{}",
                                result.exit_code,
                                result.stderr.trim()
                            ),
                        });
                    }
                }
                Ok(Err(e)) => {
                    assistant_text = text_parts.join("");
                    if assistant_text.is_empty() {
                        assistant_text = format!("Error: {}", e);
                    }
                }
                Err(e) => {
                    assistant_text = format!("Error: task panicked: {}", e);
                }
            }
        }
        Err(e) => {
            let error_event = StreamEvent {
                id: Uuid::new_v4().to_string(),
                sandbox_id,
                session_id: Some(sid),
                event_type: StreamEventType::Error,
                data: json!({ "message": e.to_string() }),
                timestamp: Utc::now(),
            };
            let _ = stream_handler.publish(error_event).await;
            assistant_text = format!("Error: {}", e);
        }
    }

    // Build the final assistant content.
    if !assistant_text.is_empty() {
        assistant_content.insert(
            0,
            MessageContent::Text {
                text: assistant_text,
            },
        );
    }

    if assistant_content.is_empty() {
        assistant_content.push(MessageContent::Text {
            text: "(No response from agent)".to_string(),
        });
    }

    // Store assistant message.
    let assistant_msg = Message {
        id: Uuid::new_v4(),
        session_id: sid,
        role: MessageRole::Assistant,
        content: assistant_content,
        timestamp: Utc::now(),
    };
    let _ = db.insert_message(&assistant_msg).await;

    // Persist captured Claude session ID for future --resume.
    if let Some(claude_sid) = captured_claude_session_id {
        let mut metadata = session_metadata;
        metadata.insert(
            "claude_session_id".to_string(),
            serde_json::Value::String(claude_sid),
        );
        let _ = db.update_session_metadata(&sid, &metadata).await;
    }

    // Update session state back to active.
    let _ = db.update_session_state(&sid, &SessionState::Active).await;

    // Now that the assistant message is persisted, publish the deferred
    // session_completed event. The frontend can safely refetch and find
    // the assistant message in the DB.
    if let Some(event) = deferred_completion_event {
        let _ = stream_handler.publish(event).await;
    }
}

// ---------------------------------------------------------------------------
// interrupt_session
// ---------------------------------------------------------------------------

pub async fn interrupt_session(
    State(state): State<AppState>,
    Path(sid): Path<Uuid>,
) -> Result<impl IntoResponse, CiabError> {
    let session = state
        .db
        .get_session(&sid)
        .await?
        .ok_or_else(|| CiabError::SessionNotFound(sid.to_string()))?;

    let sandbox = state
        .db
        .get_sandbox(&session.sandbox_id)
        .await?
        .ok_or_else(|| CiabError::SandboxNotFound(session.sandbox_id.to_string()))?;

    let agent = state
        .agents
        .get(&sandbox.agent_provider)
        .ok_or_else(|| CiabError::AgentProviderNotFound(sandbox.agent_provider.clone()))?;

    agent.interrupt(&sandbox.id).await?;
    let _ = state.runtime.kill_exec(&sandbox.id).await;

    state
        .db
        .update_session_state(&sid, &SessionState::Active)
        .await?;

    Ok(Json(json!({"status": "interrupted"})))
}

// ---------------------------------------------------------------------------
// session_stream — SSE stream filtered by session_id
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Default)]
pub struct SessionStreamQuery {
    #[serde(default)]
    pub last_event_id: Option<String>,
}

pub async fn session_stream(
    State(state): State<AppState>,
    Path(sid): Path<Uuid>,
    Query(params): Query<SessionStreamQuery>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, CiabError> {
    let session = state
        .db
        .get_session(&sid)
        .await?
        .ok_or_else(|| CiabError::SessionNotFound(sid.to_string()))?;

    // Use subscribe_with_replay to get buffered events + live receiver.
    let (replay_events, mut rx) = state
        .stream_handler
        .subscribe_with_replay(&session.sandbox_id, params.last_event_id.as_deref())
        .await?;
    let keepalive_secs = state.config.streaming.keepalive_interval_secs;

    let stream = async_stream::stream! {
        // 1. Replay buffered events (filtered to this session)
        for event in &replay_events {
            let matches = match event.session_id {
                Some(event_sid) => event_sid == sid,
                None => true,
            };
            if matches {
                let data = serde_json::to_string(event).unwrap_or_default();
                let event_type = serde_json::to_string(&event.event_type)
                    .unwrap_or_default()
                    .trim_matches('"')
                    .to_string();
                yield Ok::<_, Infallible>(
                    Event::default()
                        .id(event.id.clone())
                        .event(event_type)
                        .data(data),
                );
            }
        }

        // 2. Stream live events with keepalive
        let mut keepalive = tokio::time::interval(
            std::time::Duration::from_secs(keepalive_secs),
        );
        keepalive.tick().await;

        loop {
            tokio::select! {
                result = rx.recv() => {
                    match result {
                        Ok(event) => {
                            let matches = match event.session_id {
                                Some(event_sid) => event_sid == sid,
                                None => true,
                            };
                            if matches {
                                let data = serde_json::to_string(&event).unwrap_or_default();
                                let event_type = serde_json::to_string(&event.event_type)
                                    .unwrap_or_default()
                                    .trim_matches('"')
                                    .to_string();
                                yield Ok::<_, Infallible>(
                                    Event::default()
                                        .id(event.id)
                                        .event(event_type)
                                        .data(data),
                                );
                            }
                        }
                        Err(_) => continue,
                    }
                }
                _ = keepalive.tick() => {
                    yield Ok(Event::default().comment("keepalive"));
                }
            }
        }
    };

    Ok(Sse::new(stream))
}

// ---------------------------------------------------------------------------
// update_session_skills — update active skills for a session
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct UpdateSessionSkillsRequest {
    pub active_skills: Vec<serde_json::Value>,
}

pub async fn update_session_skills(
    State(state): State<AppState>,
    Path(sid): Path<Uuid>,
    Json(body): Json<UpdateSessionSkillsRequest>,
) -> Result<impl IntoResponse, CiabError> {
    let session = state
        .db
        .get_session(&sid)
        .await?
        .ok_or_else(|| CiabError::SessionNotFound(sid.to_string()))?;

    let mut metadata = session.metadata.clone();
    metadata.insert(
        "active_skills".to_string(),
        serde_json::Value::Array(body.active_skills.clone()),
    );
    state.db.update_session_metadata(&sid, &metadata).await?;

    Ok(Json(json!({
        "status": "ok",
        "active_skills": body.active_skills,
    })))
}

// ---------------------------------------------------------------------------
// get_queue — list queued messages for a session
// ---------------------------------------------------------------------------

pub async fn get_queue(
    State(state): State<AppState>,
    Path(sid): Path<Uuid>,
) -> Result<impl IntoResponse, CiabError> {
    let queues = state.session_queues.read().await;
    let queue = queues.get(&sid);
    let messages: Vec<_> = queue
        .map(|q| q.messages.iter().cloned().collect())
        .unwrap_or_default();
    let processing = queue.map(|q| q.processing).unwrap_or(false);
    Ok(Json(json!({
        "messages": messages,
        "processing": processing,
        "queue_length": messages.len(),
    })))
}

// ---------------------------------------------------------------------------
// cancel_queued_message — remove a message from the queue
// ---------------------------------------------------------------------------

pub async fn cancel_queued_message(
    State(state): State<AppState>,
    Path((sid, msg_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, CiabError> {
    let mut queues = state.session_queues.write().await;
    let queue = queues
        .get_mut(&sid)
        .ok_or_else(|| CiabError::SessionNotFound(sid.to_string()))?;

    let before_len = queue.messages.len();
    queue.messages.retain(|m| m.id != msg_id);
    let removed = queue.messages.len() < before_len;

    if removed {
        // Also delete the user message from DB since it won't be processed.
        let _ = state.db.delete_message(&msg_id).await;
    }

    // Emit queue_updated event.
    let sandbox_id = state
        .db
        .get_session(&sid)
        .await
        .ok()
        .flatten()
        .map(|s| s.sandbox_id)
        .unwrap_or(Uuid::nil());
    let queue_positions: Vec<_> = queue
        .messages
        .iter()
        .map(|m| {
            json!({
                "id": m.id,
                "prompt_text": m.prompt_text,
                "queued_at": m.queued_at,
            })
        })
        .collect();
    let _ = state
        .stream_handler
        .publish(StreamEvent {
            id: Uuid::new_v4().to_string(),
            sandbox_id,
            session_id: Some(sid),
            event_type: StreamEventType::QueueUpdated,
            data: json!({
                "queue": queue_positions,
                "queue_length": queue.messages.len(),
            }),
            timestamp: Utc::now(),
        })
        .await;

    Ok(Json(json!({
        "status": if removed { "cancelled" } else { "not_found" },
        "queue_length": queue.messages.len(),
    })))
}

// ---------------------------------------------------------------------------
// handle_local_command — handles non-native slash commands locally
// ---------------------------------------------------------------------------

async fn handle_local_command(
    state: &AppState,
    session_id: &Uuid,
    cmd_name: &str,
    commands: &[ciab_core::types::agent::SlashCommand],
) -> Result<Json<Message>, CiabError> {
    match cmd_name {
        "clear" => {
            state.db.delete_session_messages(session_id).await?;
            let msg = Message {
                id: Uuid::new_v4(),
                session_id: *session_id,
                role: MessageRole::Assistant,
                content: vec![MessageContent::Text {
                    text: "Conversation cleared.".to_string(),
                }],
                timestamp: Utc::now(),
            };
            Ok(Json(msg))
        }
        "help" => {
            let mut help = String::from("## Available Commands\n\n");
            let mut by_category: std::collections::BTreeMap<
                String,
                Vec<&ciab_core::types::agent::SlashCommand>,
            > = std::collections::BTreeMap::new();
            for cmd in commands {
                let cat = serde_json::to_value(&cmd.category)
                    .ok()
                    .and_then(|v| v.as_str().map(|s| s.to_string()))
                    .unwrap_or_else(|| "other".to_string());
                by_category.entry(cat).or_default().push(cmd);
            }
            for (category, cmds) in &by_category {
                help.push_str(&format!("### {}\n", category));
                for cmd in cmds {
                    help.push_str(&format!(
                        "- **/{name}** — {desc}\n",
                        name = cmd.name,
                        desc = cmd.description
                    ));
                }
                help.push('\n');
            }
            let msg = Message {
                id: Uuid::new_v4(),
                session_id: *session_id,
                role: MessageRole::Assistant,
                content: vec![MessageContent::Text { text: help }],
                timestamp: Utc::now(),
            };
            state.db.insert_message(&msg).await?;
            Ok(Json(msg))
        }
        "skills" => {
            // Return the session's active skills as a message.
            // The frontend intercepts this to open the skill picker dialog.
            let session = state
                .db
                .get_session(session_id)
                .await?
                .ok_or_else(|| CiabError::SessionNotFound(session_id.to_string()))?;

            let active_skills = session
                .metadata
                .get("active_skills")
                .cloned()
                .unwrap_or_else(|| json!([]));

            let msg = Message {
                id: Uuid::new_v4(),
                session_id: *session_id,
                role: MessageRole::Assistant,
                content: vec![MessageContent::Text {
                    text: json!({
                        "__ciab_command": "skills",
                        "active_skills": active_skills,
                    })
                    .to_string(),
                }],
                timestamp: Utc::now(),
            };
            Ok(Json(msg))
        }
        _ => Err(CiabError::ConfigValidationError(format!(
            "Unknown local command: /{}",
            cmd_name
        ))),
    }
}

// ---------------------------------------------------------------------------
// Host Claude OAuth token inheritance
// ---------------------------------------------------------------------------

enum HostClaudeAuth {
    /// Valid token read from host keychain or credentials file.
    ValidToken {
        token: String,
        subscription_type: Option<String>,
        /// Seconds until the token expires.
        expires_in_secs: i64,
    },
    /// Token found but already expired.
    Expired { subscription_type: Option<String> },
    /// No credentials found on the host.
    NotFound,
}

/// Try to read the Claude Code OAuth token from the local machine.
///
/// Claude Code stores credentials in two places (tried in order):
/// 1. macOS Keychain — `security find-generic-password -s "Claude Code-credentials"`
/// 2. ~/.claude/.credentials.json — plaintext fallback
///
/// The stored JSON is: `{ "claudeAiOauth": { "accessToken": "...", "expiresAt": <ms>, ... } }`
///
/// Claude Code accepts the token via `CLAUDE_CODE_OAUTH_TOKEN` env var.
fn read_host_claude_oauth_token() -> HostClaudeAuth {
    // 1. Try macOS Keychain
    #[cfg(target_os = "macos")]
    {
        if let Some(creds) = read_keychain_credentials() {
            return parse_claude_credentials(&creds);
        }
    }

    // 2. Try ~/.claude/.credentials.json (plaintext fallback, all platforms)
    let credentials_path = {
        let home = std::env::var("HOME").unwrap_or_else(|_| "~".to_string());
        let config_dir =
            std::env::var("CLAUDE_CONFIG_DIR").unwrap_or_else(|_| format!("{}/.claude", home));
        std::path::PathBuf::from(config_dir).join(".credentials.json")
    };

    if let Ok(raw) = std::fs::read_to_string(&credentials_path) {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&raw) {
            return parse_claude_credentials(&value);
        }
    }

    HostClaudeAuth::NotFound
}

#[cfg(target_os = "macos")]
fn read_keychain_credentials() -> Option<serde_json::Value> {
    // Claude Code uses: security find-generic-password -s "Claude Code-credentials" -w
    // The account is the current user. No suffix when using default CLAUDE_CONFIG_DIR.
    let service = "Claude Code-credentials";
    let output = std::process::Command::new("security")
        .args(["find-generic-password", "-s", service, "-w"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let raw = String::from_utf8(output.stdout).ok()?;
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }

    serde_json::from_str(raw).ok()
}

fn parse_claude_credentials(value: &serde_json::Value) -> HostClaudeAuth {
    let oauth = match value.get("claudeAiOauth") {
        Some(v) => v,
        None => return HostClaudeAuth::NotFound,
    };

    let access_token = match oauth.get("accessToken").and_then(|v| v.as_str()) {
        Some(t) if !t.is_empty() => t.to_string(),
        _ => return HostClaudeAuth::NotFound,
    };

    let subscription_type = oauth
        .get("subscriptionType")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let expires_at_ms = oauth.get("expiresAt").and_then(|v| v.as_i64()).unwrap_or(0);

    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);

    let expires_in_secs = (expires_at_ms - now_ms) / 1000;

    if expires_at_ms > 0 && expires_in_secs <= 0 {
        return HostClaudeAuth::Expired { subscription_type };
    }

    HostClaudeAuth::ValidToken {
        token: access_token,
        subscription_type,
        expires_in_secs,
    }
}
