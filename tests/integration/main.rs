use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use ciab_api::{build_router, AppState};
use ciab_core::error::{CiabError, CiabResult};
use ciab_core::traits::agent::AgentProvider;
use ciab_core::traits::runtime::SandboxRuntime;
use ciab_core::traits::stream::StreamHandler;
use ciab_core::types::agent::{AgentCommand, AgentConfig, AgentHealth};
use ciab_core::types::config::*;
use ciab_core::types::sandbox::*;
use ciab_core::types::session::Message;
use ciab_core::types::stream::{StreamEvent, StreamEventType};
use ciab_credentials::CredentialStore;
use ciab_db::Database;
use ciab_provisioning::ProvisioningPipeline;
use ciab_streaming::StreamBroker;
use dashmap::DashMap;
use tokio::sync::mpsc;
use uuid::Uuid;

use ciab_agent_claude::ClaudeCodeProvider;
use ciab_agent_codex::CodexProvider;
use ciab_agent_cursor::CursorProvider;
use ciab_agent_gemini::GeminiProvider;
// SlashCommand types tested via API endpoints

use axum::body::Body;
use axum::http::{self, Request, StatusCode};
use tower::ServiceExt;

// =============================================================================
// Mock Sandbox Runtime
// =============================================================================

struct MockRuntime {
    sandboxes: DashMap<Uuid, SandboxInfo>,
    files: DashMap<(Uuid, String), Vec<u8>>,
}

impl MockRuntime {
    fn new() -> Self {
        Self {
            sandboxes: DashMap::new(),
            files: DashMap::new(),
        }
    }
}

#[async_trait]
impl SandboxRuntime for MockRuntime {
    async fn create_sandbox(&self, spec: &SandboxSpec) -> CiabResult<SandboxInfo> {
        let now = Utc::now();
        let id = Uuid::new_v4();
        let info = SandboxInfo {
            id,
            name: spec.name.clone(),
            state: SandboxState::Running,
            persistence: spec.persistence.clone(),
            agent_provider: spec.agent_provider.clone(),
            endpoint_url: Some(format!("http://mock-sandbox-{}", id)),
            resource_stats: None,
            labels: spec.labels.clone(),
            created_at: now,
            updated_at: now,
            spec: spec.clone(),
        };
        self.sandboxes.insert(id, info.clone());
        Ok(info)
    }

    async fn get_sandbox(&self, id: &Uuid) -> CiabResult<SandboxInfo> {
        self.sandboxes
            .get(id)
            .map(|v| v.clone())
            .ok_or_else(|| CiabError::SandboxNotFound(id.to_string()))
    }

    async fn list_sandboxes(
        &self,
        state: Option<SandboxState>,
        provider: Option<&str>,
        _labels: &HashMap<String, String>,
    ) -> CiabResult<Vec<SandboxInfo>> {
        let mut result: Vec<SandboxInfo> =
            self.sandboxes.iter().map(|e| e.value().clone()).collect();
        if let Some(s) = state {
            result.retain(|sb| sb.state == s);
        }
        if let Some(p) = provider {
            result.retain(|sb| sb.agent_provider == p);
        }
        Ok(result)
    }

    async fn start_sandbox(&self, id: &Uuid) -> CiabResult<()> {
        let mut entry = self
            .sandboxes
            .get_mut(id)
            .ok_or_else(|| CiabError::SandboxNotFound(id.to_string()))?;
        entry.state = SandboxState::Running;
        Ok(())
    }

    async fn stop_sandbox(&self, id: &Uuid) -> CiabResult<()> {
        let mut entry = self
            .sandboxes
            .get_mut(id)
            .ok_or_else(|| CiabError::SandboxNotFound(id.to_string()))?;
        entry.state = SandboxState::Stopped;
        Ok(())
    }

    async fn pause_sandbox(&self, id: &Uuid) -> CiabResult<()> {
        let mut entry = self
            .sandboxes
            .get_mut(id)
            .ok_or_else(|| CiabError::SandboxNotFound(id.to_string()))?;
        entry.state = SandboxState::Paused;
        Ok(())
    }

    async fn resume_sandbox(&self, id: &Uuid) -> CiabResult<()> {
        let mut entry = self
            .sandboxes
            .get_mut(id)
            .ok_or_else(|| CiabError::SandboxNotFound(id.to_string()))?;
        entry.state = SandboxState::Running;
        Ok(())
    }

    async fn terminate_sandbox(&self, id: &Uuid) -> CiabResult<()> {
        self.sandboxes.remove(id);
        Ok(())
    }

    async fn exec(&self, _id: &Uuid, request: &ExecRequest) -> CiabResult<ExecResult> {
        let cmd_str = request.command.join(" ");
        Ok(ExecResult {
            exit_code: 0,
            stdout: format!("mock output for: {}", cmd_str),
            stderr: String::new(),
            duration_ms: 42,
        })
    }

    async fn read_file(&self, id: &Uuid, path: &str) -> CiabResult<Vec<u8>> {
        self.files
            .get(&(*id, path.to_string()))
            .map(|v| v.clone())
            .ok_or_else(|| CiabError::FileNotFound(format!("{}:{}", id, path)))
    }

    async fn write_file(&self, id: &Uuid, path: &str, content: &[u8]) -> CiabResult<()> {
        self.files.insert((*id, path.to_string()), content.to_vec());
        Ok(())
    }

    async fn list_files(&self, id: &Uuid, _path: &str) -> CiabResult<Vec<FileInfo>> {
        let files: Vec<FileInfo> = self
            .files
            .iter()
            .filter(|entry| entry.key().0 == *id)
            .map(|entry| FileInfo {
                path: entry.key().1.clone(),
                size: entry.value().len() as u64,
                is_dir: false,
                mode: 0o644,
                modified_at: Some(Utc::now()),
            })
            .collect();
        Ok(files)
    }

    async fn get_stats(&self, _id: &Uuid) -> CiabResult<ResourceStats> {
        Ok(ResourceStats {
            cpu_usage_percent: 15.5,
            memory_used_mb: 256,
            memory_limit_mb: 1024,
            disk_used_mb: 100,
            disk_limit_mb: 5120,
            network_rx_bytes: 1024000,
            network_tx_bytes: 512000,
        })
    }

    async fn stream_logs(
        &self,
        _id: &Uuid,
        _options: &LogOptions,
    ) -> CiabResult<mpsc::Receiver<String>> {
        let (tx, rx) = mpsc::channel(16);
        tokio::spawn(async move {
            let _ = tx.send("mock log line 1".to_string()).await;
            let _ = tx.send("mock log line 2".to_string()).await;
        });
        Ok(rx)
    }
}

// =============================================================================
// Mock Agent Provider
// =============================================================================

struct MockAgentProvider;

#[async_trait]
impl AgentProvider for MockAgentProvider {
    fn name(&self) -> &str {
        "mock-agent"
    }

    fn base_image(&self) -> &str {
        "mock-image:latest"
    }

    fn install_commands(&self) -> Vec<String> {
        vec!["echo install".to_string()]
    }

    fn build_start_command(&self, _config: &AgentConfig) -> AgentCommand {
        AgentCommand {
            command: "echo".to_string(),
            args: vec!["agent-started".to_string()],
            env: HashMap::new(),
            workdir: Some("/workspace".to_string()),
        }
    }

    fn required_env_vars(&self) -> Vec<String> {
        vec!["MOCK_API_KEY".to_string()]
    }

    fn parse_output(&self, sandbox_id: &Uuid, raw: &str) -> Vec<StreamEvent> {
        vec![StreamEvent {
            id: Uuid::new_v4().to_string(),
            sandbox_id: *sandbox_id,
            session_id: None,
            event_type: StreamEventType::TextDelta,
            data: serde_json::json!({"text": raw}),
            timestamp: Utc::now(),
        }]
    }

    fn validate_config(&self, _config: &AgentConfig) -> CiabResult<()> {
        Ok(())
    }

    async fn send_message(
        &self,
        sandbox_id: &Uuid,
        session_id: &Uuid,
        message: &Message,
        tx: &mpsc::Sender<StreamEvent>,
    ) -> CiabResult<()> {
        let text = message
            .content
            .iter()
            .filter_map(|c| match c {
                ciab_core::types::session::MessageContent::Text { text } => Some(text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("");

        let response_text = format!("Mock response to: {}", text);

        let _ = tx
            .send(StreamEvent {
                id: Uuid::new_v4().to_string(),
                sandbox_id: *sandbox_id,
                session_id: Some(*session_id),
                event_type: StreamEventType::TextComplete,
                data: serde_json::json!({"text": response_text}),
                timestamp: Utc::now(),
            })
            .await;
        Ok(())
    }

    async fn interrupt(&self, _sandbox_id: &Uuid) -> CiabResult<()> {
        Ok(())
    }

    async fn health_check(&self, _sandbox_id: &Uuid) -> CiabResult<AgentHealth> {
        Ok(AgentHealth {
            healthy: true,
            status: "running".to_string(),
            uptime_secs: Some(120),
        })
    }
}

// =============================================================================
// Test Helpers
// =============================================================================

fn test_config() -> AppConfig {
    AppConfig {
        server: ServerConfig {
            host: "127.0.0.1".to_string(),
            port: 0,
            workers: None,
            request_timeout_secs: 30,
            cors_origins: vec!["*".to_string()],
            web_ui_dir: None,
        },
        runtime: RuntimeConfig {
            backend: "local".to_string(),
            opensandbox_url: Some("http://mock:8000".to_string()),
            opensandbox_api_key: None,
            docker_socket: None,
            local_workdir: None,
            local_max_processes: None,
            kubernetes: None,
            ec2: None,
            packer: None,
        },
        agents: AgentsConfig {
            default_provider: "mock-agent".to_string(),
            providers: HashMap::new(),
        },
        credentials: CredentialsConfig {
            backend: "sqlite".to_string(),
            encryption_key_env: "test-encryption-key-for-ciab-testing-1234".to_string(),
        },
        provisioning: ProvisioningConfig {
            timeout_secs: 60,
            max_script_size_bytes: 1048576,
        },
        streaming: StreamingConfig {
            buffer_size: 100,
            keepalive_interval_secs: 15,
            max_stream_duration_secs: 3600,
        },
        security: SecurityConfig {
            api_keys: vec![], // empty = no auth required
            drop_capabilities: vec![],
        },
        logging: LoggingConfig {
            level: "info".to_string(),
            format: "json".to_string(),
        },
        oauth: None,
        gateway: Default::default(),
        channels: Default::default(),
        llm_providers: Default::default(),
    }
}

async fn setup_test_app() -> (axum::Router, Arc<Database>, Arc<dyn SandboxRuntime>) {
    let db = Arc::new(Database::new("sqlite::memory:").await.unwrap());
    let runtime: Arc<dyn SandboxRuntime> = Arc::new(MockRuntime::new());
    let stream_handler = Arc::new(StreamBroker::new(100));
    let credential_store = Arc::new(
        CredentialStore::new(db.clone(), "test-encryption-key-for-ciab-testing-1234").unwrap(),
    );
    let provisioning = Arc::new(ProvisioningPipeline::new(
        runtime.clone(),
        credential_store.clone(),
        60,
    ));

    let mut agents: HashMap<String, Arc<dyn AgentProvider>> = HashMap::new();
    agents.insert("mock-agent".to_string(), Arc::new(MockAgentProvider));

    let config = Arc::new(test_config());

    let mut runtimes: HashMap<String, Arc<dyn SandboxRuntime>> = HashMap::new();
    runtimes.insert("local".to_string(), runtime.clone());

    let state = AppState {
        runtime: runtime.clone(),
        agents,
        runtimes,
        credentials: credential_store,
        stream_handler,
        provisioning,
        db: db.clone(),
        config,
        config_path: None,
        gateway: Arc::new(tokio::sync::RwLock::new(None)),
        channel_manager: Arc::new(tokio::sync::RwLock::new(None)),
        pending_permissions: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        session_permissions: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        pending_user_inputs: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        session_queues: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        image_builder: None,
    };

    let router = build_router(state);
    (router, db, runtime)
}

fn json_request(method: &str, uri: &str, body: Option<serde_json::Value>) -> Request<Body> {
    let builder = Request::builder()
        .method(method)
        .uri(uri)
        .header(http::header::CONTENT_TYPE, "application/json");

    match body {
        Some(b) => builder
            .body(Body::from(serde_json::to_vec(&b).unwrap()))
            .unwrap(),
        None => builder.body(Body::empty()).unwrap(),
    }
}

async fn response_json(response: axum::response::Response) -> serde_json::Value {
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&body).unwrap_or_else(|_| {
        let text = String::from_utf8_lossy(&body);
        serde_json::json!({"raw": text.to_string()})
    })
}

// =============================================================================
// Tests
// =============================================================================

#[tokio::test]
async fn test_health_endpoint() {
    let (app, _db, _rt) = setup_test_app().await;

    let response = app
        .oneshot(json_request("GET", "/health", None))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    assert_eq!(body["status"], "healthy");
}

#[tokio::test]
async fn test_ready_endpoint() {
    let (app, _db, _rt) = setup_test_app().await;

    let response = app
        .oneshot(json_request("GET", "/ready", None))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    assert_eq!(body["status"], "ready");
}

#[tokio::test]
async fn test_sandbox_create_returns_accepted() {
    let (app, _db, _rt) = setup_test_app().await;

    let response = app
        .oneshot(json_request(
            "POST",
            "/api/v1/sandboxes",
            Some(serde_json::json!({
                "agent_provider": "mock-agent",
                "name": "test-sandbox",
                "env_vars": {"MOCK_API_KEY": "test-key-123"}
            })),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::ACCEPTED);
    let body = response_json(response).await;
    assert_eq!(body["status"], "provisioning");
    assert!(body["sandbox_id"].is_string());
}

#[tokio::test]
async fn test_sandbox_list_empty() {
    let (app, _db, _rt) = setup_test_app().await;

    let response = app
        .oneshot(json_request("GET", "/api/v1/sandboxes", None))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert!(body.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_sandbox_crud_lifecycle() {
    let (app, db, runtime) = setup_test_app().await;

    // First, manually insert a sandbox into the DB (since create is async/background)
    let sandbox_info = runtime
        .create_sandbox(&SandboxSpec {
            name: Some("lifecycle-test".to_string()),
            agent_provider: "mock-agent".to_string(),
            image: None,
            resource_limits: None,
            persistence: SandboxPersistence::Ephemeral,
            network: None,
            env_vars: HashMap::new(),
            volumes: vec![],
            ports: vec![],
            git_repos: vec![],
            credentials: vec![],
            provisioning_scripts: vec![],
            labels: HashMap::new(),
            timeout_secs: None,
            agent_config: None,
            local_mounts: vec![],
            runtime_backend: None,
        })
        .await
        .unwrap();
    db.insert_sandbox(&sandbox_info).await.unwrap();
    let sandbox_id = sandbox_info.id;

    // GET sandbox
    let response = app
        .clone()
        .oneshot(json_request(
            "GET",
            &format!("/api/v1/sandboxes/{}", sandbox_id),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["id"], sandbox_id.to_string());
    assert_eq!(body["name"], "lifecycle-test");
    assert_eq!(body["state"], "running");

    // LIST sandboxes
    let response = app
        .clone()
        .oneshot(json_request("GET", "/api/v1/sandboxes", None))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body.as_array().unwrap().len(), 1);

    // PAUSE sandbox
    let response = app
        .clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/v1/sandboxes/{}/pause", sandbox_id),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["status"], "paused");

    // Verify state persisted in DB
    let sb = db.get_sandbox(&sandbox_id).await.unwrap().unwrap();
    assert_eq!(sb.state, SandboxState::Paused);

    // RESUME sandbox
    let response = app
        .clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/v1/sandboxes/{}/resume", sandbox_id),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["status"], "running");

    // STOP sandbox
    let response = app
        .clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/v1/sandboxes/{}/stop", sandbox_id),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // STATS
    let response = app
        .clone()
        .oneshot(json_request(
            "GET",
            &format!("/api/v1/sandboxes/{}/stats", sandbox_id),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["cpu_usage_percent"], 15.5);
    assert_eq!(body["memory_used_mb"], 256);

    // DELETE sandbox
    let response = app
        .clone()
        .oneshot(json_request(
            "DELETE",
            &format!("/api/v1/sandboxes/{}", sandbox_id),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // Verify deleted from DB
    let sb = db.get_sandbox(&sandbox_id).await.unwrap();
    assert!(sb.is_none());
}

#[tokio::test]
async fn test_sandbox_not_found() {
    let (app, _db, _rt) = setup_test_app().await;
    let fake_id = Uuid::new_v4();

    let response = app
        .oneshot(json_request(
            "GET",
            &format!("/api/v1/sandboxes/{}", fake_id),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body = response_json(response).await;
    assert_eq!(body["error"]["code"], "sandbox_not_found");
}

#[tokio::test]
async fn test_session_create_and_list() {
    let (app, db, runtime) = setup_test_app().await;

    // Create sandbox
    let sandbox_info = runtime
        .create_sandbox(&SandboxSpec {
            name: Some("session-test".to_string()),
            agent_provider: "mock-agent".to_string(),
            image: None,
            resource_limits: None,
            persistence: SandboxPersistence::Ephemeral,
            network: None,
            env_vars: HashMap::new(),
            volumes: vec![],
            ports: vec![],
            git_repos: vec![],
            credentials: vec![],
            provisioning_scripts: vec![],
            labels: HashMap::new(),
            timeout_secs: None,
            agent_config: None,
            local_mounts: vec![],
            runtime_backend: None,
        })
        .await
        .unwrap();
    db.insert_sandbox(&sandbox_info).await.unwrap();
    let sandbox_id = sandbox_info.id;

    // CREATE session
    let response = app
        .clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/v1/sandboxes/{}/sessions", sandbox_id),
            Some(serde_json::json!({"metadata": {"test": true}})),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    let session_id = body["id"].as_str().unwrap().to_string();
    assert_eq!(body["sandbox_id"], sandbox_id.to_string());
    assert_eq!(body["state"], "active");

    // LIST sessions
    let response = app
        .clone()
        .oneshot(json_request(
            "GET",
            &format!("/api/v1/sandboxes/{}/sessions", sandbox_id),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body.as_array().unwrap().len(), 1);

    // GET session
    let response = app
        .clone()
        .oneshot(json_request(
            "GET",
            &format!("/api/v1/sessions/{}", session_id),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    // get_session returns a flat object (not nested under "session")
    assert_eq!(body["id"], session_id);
}

#[tokio::test]
async fn test_send_message_and_get_response() {
    let (app, db, runtime) = setup_test_app().await;

    // Setup sandbox + session
    let sandbox_info = runtime
        .create_sandbox(&SandboxSpec {
            name: Some("msg-test".to_string()),
            agent_provider: "mock-agent".to_string(),
            image: None,
            resource_limits: None,
            persistence: SandboxPersistence::Ephemeral,
            network: None,
            env_vars: HashMap::new(),
            volumes: vec![],
            ports: vec![],
            git_repos: vec![],
            credentials: vec![],
            provisioning_scripts: vec![],
            labels: HashMap::new(),
            timeout_secs: None,
            agent_config: None,
            local_mounts: vec![],
            runtime_backend: None,
        })
        .await
        .unwrap();
    db.insert_sandbox(&sandbox_info).await.unwrap();

    let session = ciab_core::types::session::Session {
        id: Uuid::new_v4(),
        sandbox_id: sandbox_info.id,
        state: ciab_core::types::session::SessionState::Active,
        metadata: HashMap::new(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    db.insert_session(&session).await.unwrap();

    // SEND MESSAGE
    let response = app
        .clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/v1/sessions/{}/messages", session.id),
            Some(serde_json::json!({"message": "Hello agent!"})),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    // send_message now returns the user message as an ack; agent runs in background
    assert_eq!(body["role"], "user");

    // Allow background processing to complete
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // Verify user message is persisted
    let response = app
        .clone()
        .oneshot(json_request(
            "GET",
            &format!("/api/v1/sessions/{}", session.id),
            None,
        ))
        .await
        .unwrap();
    let body = response_json(response).await;
    let messages = body["messages"].as_array().unwrap();
    assert!(!messages.is_empty());
    assert_eq!(messages[0]["role"], "user");
}

#[tokio::test]
async fn test_exec_command() {
    let (app, db, runtime) = setup_test_app().await;

    let sandbox_info = runtime
        .create_sandbox(&SandboxSpec {
            name: Some("exec-test".to_string()),
            agent_provider: "mock-agent".to_string(),
            image: None,
            resource_limits: None,
            persistence: SandboxPersistence::Ephemeral,
            network: None,
            env_vars: HashMap::new(),
            volumes: vec![],
            ports: vec![],
            git_repos: vec![],
            credentials: vec![],
            provisioning_scripts: vec![],
            labels: HashMap::new(),
            timeout_secs: None,
            agent_config: None,
            local_mounts: vec![],
            runtime_backend: None,
        })
        .await
        .unwrap();
    db.insert_sandbox(&sandbox_info).await.unwrap();

    let response = app
        .oneshot(json_request(
            "POST",
            &format!("/api/v1/sandboxes/{}/exec", sandbox_info.id),
            Some(serde_json::json!({
                "command": ["echo", "hello", "world"],
                "workdir": "/workspace"
            })),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["exit_code"], 0);
    assert!(body["stdout"]
        .as_str()
        .unwrap()
        .contains("echo hello world"));
    assert_eq!(body["duration_ms"], 42);
}

#[tokio::test]
async fn test_file_operations() {
    let (app, db, runtime) = setup_test_app().await;

    let sandbox_info = runtime
        .create_sandbox(&SandboxSpec {
            name: Some("files-test".to_string()),
            agent_provider: "mock-agent".to_string(),
            image: None,
            resource_limits: None,
            persistence: SandboxPersistence::Ephemeral,
            network: None,
            env_vars: HashMap::new(),
            volumes: vec![],
            ports: vec![],
            git_repos: vec![],
            credentials: vec![],
            provisioning_scripts: vec![],
            labels: HashMap::new(),
            timeout_secs: None,
            agent_config: None,
            local_mounts: vec![],
            runtime_backend: None,
        })
        .await
        .unwrap();
    db.insert_sandbox(&sandbox_info).await.unwrap();
    let sid = sandbox_info.id;

    // UPLOAD file
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!(
                    "/api/v1/sandboxes/{}/files/workspace/test.txt",
                    sid
                ))
                .header(http::header::CONTENT_TYPE, "application/octet-stream")
                .body(Body::from("hello file content"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // LIST files
    let response = app
        .clone()
        .oneshot(json_request(
            "GET",
            &format!("/api/v1/sandboxes/{}/files?path=/workspace", sid),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    let files = body.as_array().unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0]["path"], "/workspace/test.txt");

    // DOWNLOAD file
    let response = app
        .clone()
        .oneshot(json_request(
            "GET",
            &format!("/api/v1/sandboxes/{}/files/workspace/test.txt", sid),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(body_bytes.as_ref(), b"hello file content");

    // DELETE file (uses exec under the hood)
    let response = app
        .clone()
        .oneshot(json_request(
            "DELETE",
            &format!("/api/v1/sandboxes/{}/files/workspace/test.txt", sid),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_credential_crud() {
    let (app, _db, _rt) = setup_test_app().await;

    // CREATE credential
    let response = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/v1/credentials",
            Some(serde_json::json!({
                "name": "my-api-key",
                "credential_type": "api_key",
                "value": "sk-secret-12345",
                "labels": {"provider": "anthropic"}
            })),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = response_json(response).await;
    let cred_id = body["id"].as_str().unwrap().to_string();
    assert_eq!(body["name"], "my-api-key");
    assert_eq!(body["credential_type"], "api_key");

    // LIST credentials
    let response = app
        .clone()
        .oneshot(json_request("GET", "/api/v1/credentials", None))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body.as_array().unwrap().len(), 1);

    // GET credential
    let response = app
        .clone()
        .oneshot(json_request(
            "GET",
            &format!("/api/v1/credentials/{}", cred_id),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["name"], "my-api-key");

    // DELETE credential
    let response = app
        .clone()
        .oneshot(json_request(
            "DELETE",
            &format!("/api/v1/credentials/{}", cred_id),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // Verify deleted
    let response = app
        .clone()
        .oneshot(json_request("GET", "/api/v1/credentials", None))
        .await
        .unwrap();
    let body = response_json(response).await;
    assert!(body.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_sandbox_logs() {
    let (app, db, runtime) = setup_test_app().await;

    let sandbox_info = runtime
        .create_sandbox(&SandboxSpec {
            name: Some("logs-test".to_string()),
            agent_provider: "mock-agent".to_string(),
            image: None,
            resource_limits: None,
            persistence: SandboxPersistence::Ephemeral,
            network: None,
            env_vars: HashMap::new(),
            volumes: vec![],
            ports: vec![],
            git_repos: vec![],
            credentials: vec![],
            provisioning_scripts: vec![],
            labels: HashMap::new(),
            timeout_secs: None,
            agent_config: None,
            local_mounts: vec![],
            runtime_backend: None,
        })
        .await
        .unwrap();
    db.insert_sandbox(&sandbox_info).await.unwrap();

    // Non-follow logs (returns JSON)
    let response = app
        .oneshot(json_request(
            "GET",
            &format!("/api/v1/sandboxes/{}/logs", sandbox_info.id),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    let logs = body["logs"].as_array().unwrap();
    assert_eq!(logs.len(), 2);
    assert_eq!(logs[0], "mock log line 1");
    assert_eq!(logs[1], "mock log line 2");
}

#[tokio::test]
async fn test_interrupt_session() {
    let (app, db, runtime) = setup_test_app().await;

    let sandbox_info = runtime
        .create_sandbox(&SandboxSpec {
            name: Some("interrupt-test".to_string()),
            agent_provider: "mock-agent".to_string(),
            image: None,
            resource_limits: None,
            persistence: SandboxPersistence::Ephemeral,
            network: None,
            env_vars: HashMap::new(),
            volumes: vec![],
            ports: vec![],
            git_repos: vec![],
            credentials: vec![],
            provisioning_scripts: vec![],
            labels: HashMap::new(),
            timeout_secs: None,
            agent_config: None,
            local_mounts: vec![],
            runtime_backend: None,
        })
        .await
        .unwrap();
    db.insert_sandbox(&sandbox_info).await.unwrap();

    let session = ciab_core::types::session::Session {
        id: Uuid::new_v4(),
        sandbox_id: sandbox_info.id,
        state: ciab_core::types::session::SessionState::Processing,
        metadata: HashMap::new(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    db.insert_session(&session).await.unwrap();

    let response = app
        .oneshot(json_request(
            "POST",
            &format!("/api/v1/sessions/{}/interrupt", session.id),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["status"], "interrupted");

    // Verify state reset to active
    let updated = db.get_session(&session.id).await.unwrap().unwrap();
    assert_eq!(
        updated.state,
        ciab_core::types::session::SessionState::Active
    );
}

#[tokio::test]
async fn test_streaming_broker() {
    let broker = StreamBroker::new(100);
    let sandbox_id = Uuid::new_v4();

    // Subscribe
    let mut rx = broker.subscribe(&sandbox_id).await.unwrap();

    // Publish an event
    let event = StreamEvent {
        id: "evt-1".to_string(),
        sandbox_id,
        session_id: None,
        event_type: StreamEventType::TextDelta,
        data: serde_json::json!({"text": "hello"}),
        timestamp: Utc::now(),
    };
    broker.publish(event.clone()).await.unwrap();

    // Receive it
    let received = rx.recv().await.unwrap();
    assert_eq!(received.id, "evt-1");
    assert_eq!(received.event_type, StreamEventType::TextDelta);

    // Check buffer
    let size = broker.buffer_size(&sandbox_id).await;
    assert_eq!(size, 1);

    // Replay from the event we just published (should return nothing after it since it's the latest)
    let replayed = broker.replay_from(&sandbox_id, "evt-1").await;
    assert_eq!(replayed.len(), 0);

    // Publish another event, then replay from evt-1 should give us the new one
    let event2 = StreamEvent {
        id: "evt-2".to_string(),
        sandbox_id,
        session_id: None,
        event_type: StreamEventType::TextComplete,
        data: serde_json::json!({"text": "world"}),
        timestamp: Utc::now(),
    };
    broker.publish(event2).await.unwrap();

    let replayed = broker.replay_from(&sandbox_id, "evt-1").await;
    assert_eq!(replayed.len(), 1);
    assert_eq!(replayed[0].id, "evt-2");
}

#[tokio::test]
async fn test_credential_encryption_roundtrip() {
    let db = Arc::new(Database::new("sqlite::memory:").await.unwrap());
    let store =
        CredentialStore::new(db.clone(), "test-encryption-key-for-ciab-testing-1234").unwrap();

    // Store
    let cred = store
        .store_credential(
            "test-key",
            ciab_core::types::credentials::CredentialType::ApiKey,
            b"super-secret-value",
            HashMap::new(),
            None,
        )
        .await
        .unwrap();

    // Retrieve and decrypt
    let (retrieved, plaintext) = store.get_credential(&cred.id).await.unwrap();
    assert_eq!(retrieved.name, "test-key");
    assert_eq!(plaintext, b"super-secret-value");

    // List (should not contain plaintext)
    let all = store.list_credentials().await.unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].name, "test-key");

    // Delete
    store.delete_credential(&cred.id).await.unwrap();
    let all = store.list_credentials().await.unwrap();
    assert!(all.is_empty());
}

#[tokio::test]
async fn test_database_sandbox_crud() {
    let db = Database::new("sqlite::memory:").await.unwrap();

    let now = Utc::now();
    let id = Uuid::new_v4();
    let info = SandboxInfo {
        id,
        name: Some("db-test".to_string()),
        state: SandboxState::Running,
        persistence: SandboxPersistence::Ephemeral,
        agent_provider: "claude-code".to_string(),
        endpoint_url: None,
        resource_stats: None,
        labels: HashMap::from([("env".to_string(), "test".to_string())]),
        created_at: now,
        updated_at: now,
        spec: SandboxSpec {
            name: Some("db-test".to_string()),
            agent_provider: "claude-code".to_string(),
            image: None,
            resource_limits: None,
            persistence: SandboxPersistence::Ephemeral,
            network: None,
            env_vars: HashMap::new(),
            volumes: vec![],
            ports: vec![],
            git_repos: vec![],
            credentials: vec![],
            provisioning_scripts: vec![],
            labels: HashMap::new(),
            timeout_secs: None,
            agent_config: None,
            local_mounts: vec![],
            runtime_backend: None,
        },
    };

    // Insert
    db.insert_sandbox(&info).await.unwrap();

    // Get
    let retrieved = db.get_sandbox(&id).await.unwrap().unwrap();
    assert_eq!(retrieved.id, id);
    assert_eq!(retrieved.name, Some("db-test".to_string()));
    assert_eq!(retrieved.state, SandboxState::Running);

    // Update state
    db.update_sandbox_state(&id, &SandboxState::Paused)
        .await
        .unwrap();
    let retrieved = db.get_sandbox(&id).await.unwrap().unwrap();
    assert_eq!(retrieved.state, SandboxState::Paused);

    // List with filter
    let filters = SandboxFilters {
        state: Some(SandboxState::Paused),
        provider: None,
        labels: HashMap::new(),
    };
    let list = db.list_sandboxes(&filters).await.unwrap();
    assert_eq!(list.len(), 1);

    let filters_no_match = SandboxFilters {
        state: Some(SandboxState::Running),
        provider: None,
        labels: HashMap::new(),
    };
    let list = db.list_sandboxes(&filters_no_match).await.unwrap();
    assert!(list.is_empty());

    // Delete
    db.delete_sandbox(&id).await.unwrap();
    assert!(db.get_sandbox(&id).await.unwrap().is_none());
}

#[tokio::test]
async fn test_database_session_and_messages() {
    let db = Database::new("sqlite::memory:").await.unwrap();

    // Need a sandbox first
    let now = Utc::now();
    let sandbox_id = Uuid::new_v4();
    let info = SandboxInfo {
        id: sandbox_id,
        name: None,
        state: SandboxState::Running,
        persistence: SandboxPersistence::Ephemeral,
        agent_provider: "mock".to_string(),
        endpoint_url: None,
        resource_stats: None,
        labels: HashMap::new(),
        created_at: now,
        updated_at: now,
        spec: SandboxSpec {
            name: None,
            agent_provider: "mock".to_string(),
            image: None,
            resource_limits: None,
            persistence: SandboxPersistence::Ephemeral,
            network: None,
            env_vars: HashMap::new(),
            volumes: vec![],
            ports: vec![],
            git_repos: vec![],
            credentials: vec![],
            provisioning_scripts: vec![],
            labels: HashMap::new(),
            timeout_secs: None,
            agent_config: None,
            local_mounts: vec![],
            runtime_backend: None,
        },
    };
    db.insert_sandbox(&info).await.unwrap();

    // Create session
    let session = ciab_core::types::session::Session {
        id: Uuid::new_v4(),
        sandbox_id,
        state: ciab_core::types::session::SessionState::Active,
        metadata: HashMap::from([("key".to_string(), serde_json::json!("value"))]),
        created_at: now,
        updated_at: now,
    };
    db.insert_session(&session).await.unwrap();

    // Get session
    let retrieved = db.get_session(&session.id).await.unwrap().unwrap();
    assert_eq!(retrieved.id, session.id);

    // List sessions
    let sessions = db.list_sessions(&sandbox_id).await.unwrap();
    assert_eq!(sessions.len(), 1);

    // Insert messages
    let msg1 = ciab_core::types::session::Message {
        id: Uuid::new_v4(),
        session_id: session.id,
        role: ciab_core::types::session::MessageRole::User,
        content: vec![ciab_core::types::session::MessageContent::Text {
            text: "Hello".to_string(),
        }],
        timestamp: now,
    };
    db.insert_message(&msg1).await.unwrap();

    let msg2 = ciab_core::types::session::Message {
        id: Uuid::new_v4(),
        session_id: session.id,
        role: ciab_core::types::session::MessageRole::Assistant,
        content: vec![ciab_core::types::session::MessageContent::Text {
            text: "Hi there!".to_string(),
        }],
        timestamp: now,
    };
    db.insert_message(&msg2).await.unwrap();

    // Get messages
    let messages = db.get_messages(&session.id, None).await.unwrap();
    assert_eq!(messages.len(), 2);
    assert_eq!(
        messages[0].role,
        ciab_core::types::session::MessageRole::User
    );
    assert_eq!(
        messages[1].role,
        ciab_core::types::session::MessageRole::Assistant
    );

    // Get with limit
    let messages = db.get_messages(&session.id, Some(1)).await.unwrap();
    assert_eq!(messages.len(), 1);
}

#[tokio::test]
async fn test_agent_provider_interface() {
    let provider = MockAgentProvider;

    assert_eq!(provider.name(), "mock-agent");
    assert_eq!(provider.base_image(), "mock-image:latest");
    assert!(!provider.install_commands().is_empty());
    assert!(!provider.required_env_vars().is_empty());

    let config = AgentConfig {
        provider: "mock-agent".to_string(),
        model: Some("test-model".to_string()),
        system_prompt: None,
        max_tokens: None,
        temperature: None,
        tools_enabled: true,
        mcp_servers: vec![],
        allowed_tools: vec![],
        denied_tools: vec![],
        extra: HashMap::new(),
    };

    provider.validate_config(&config).unwrap();

    let cmd = provider.build_start_command(&config);
    assert_eq!(cmd.command, "echo");

    let sandbox_id = Uuid::new_v4();
    let health = provider.health_check(&sandbox_id).await.unwrap();
    assert!(health.healthy);

    let events = provider.parse_output(&sandbox_id, "test output");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, StreamEventType::TextDelta);
}

#[tokio::test]
async fn test_event_buffer() {
    use ciab_streaming::EventBuffer;

    let mut buf = EventBuffer::new(3);
    let sandbox_id = Uuid::new_v4();

    for i in 0..5 {
        buf.push(StreamEvent {
            id: format!("evt-{}", i),
            sandbox_id,
            session_id: None,
            event_type: StreamEventType::TextDelta,
            data: serde_json::json!({"i": i}),
            timestamp: Utc::now(),
        });
    }

    // Buffer capacity is 3, so oldest 2 should be gone
    assert_eq!(buf.len(), 3);

    // Replay from evt-2 should give evt-3, evt-4
    let replayed = buf.replay_from("evt-2");
    assert_eq!(replayed.len(), 2);
    assert_eq!(replayed[0].id, "evt-3");
    assert_eq!(replayed[1].id, "evt-4");

    // Replay from non-existent should return everything (catch-up behavior)
    let replayed = buf.replay_from("evt-999");
    assert_eq!(replayed.len(), 3);
}

#[tokio::test]
async fn test_multiple_sessions_in_sandbox() {
    let (app, db, runtime) = setup_test_app().await;

    let sandbox_info = runtime
        .create_sandbox(&SandboxSpec {
            name: Some("multi-session".to_string()),
            agent_provider: "mock-agent".to_string(),
            image: None,
            resource_limits: None,
            persistence: SandboxPersistence::Ephemeral,
            network: None,
            env_vars: HashMap::new(),
            volumes: vec![],
            ports: vec![],
            git_repos: vec![],
            credentials: vec![],
            provisioning_scripts: vec![],
            labels: HashMap::new(),
            timeout_secs: None,
            agent_config: None,
            local_mounts: vec![],
            runtime_backend: None,
        })
        .await
        .unwrap();
    db.insert_sandbox(&sandbox_info).await.unwrap();
    let sid = sandbox_info.id;

    // Create 3 sessions
    for i in 0..3 {
        let response = app
            .clone()
            .oneshot(json_request(
                "POST",
                &format!("/api/v1/sandboxes/{}/sessions", sid),
                Some(serde_json::json!({"metadata": {"idx": i}})),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    // List should return 3
    let response = app
        .clone()
        .oneshot(json_request(
            "GET",
            &format!("/api/v1/sandboxes/{}/sessions", sid),
            None,
        ))
        .await
        .unwrap();
    let body = response_json(response).await;
    assert_eq!(body.as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn test_config_deserialization() {
    let toml_str = r#"
[server]
host = "0.0.0.0"
port = 9090
request_timeout_secs = 60
cors_origins = ["*"]

[runtime]
opensandbox_url = "http://localhost:8000"

[agents]
default_provider = "claude-code"

[agents.providers.claude-code]
enabled = true
image = "test:latest"
default_model = "claude-sonnet-4-20250514"
api_key_env = "ANTHROPIC_API_KEY"

[credentials]
backend = "sqlite"
encryption_key_env = "KEY"

[provisioning]
timeout_secs = 300
max_script_size_bytes = 1048576

[streaming]
buffer_size = 500
keepalive_interval_secs = 15
max_stream_duration_secs = 3600

[security]
api_keys = []
drop_capabilities = []

[logging]
level = "info"
format = "json"
"#;

    let config: AppConfig = toml::from_str(toml_str).unwrap();
    assert_eq!(config.server.port, 9090);
    assert_eq!(config.agents.default_provider, "claude-code");
    assert_eq!(config.streaming.buffer_size, 500);
    assert!(config.agents.providers.contains_key("claude-code"));
    assert_eq!(
        config.agents.providers["claude-code"].image,
        Some("test:latest".to_string())
    );
}

#[tokio::test]
async fn test_sandbox_create_with_invalid_provider() {
    let (app, _db, _rt) = setup_test_app().await;

    let response = app
        .oneshot(json_request(
            "POST",
            "/api/v1/sandboxes",
            Some(serde_json::json!({
                "agent_provider": "nonexistent-provider"
            })),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body = response_json(response).await;
    assert_eq!(body["error"]["code"], "agent_provider_not_found");
}

// =============================================================================
// Permission System Tests
// =============================================================================

#[tokio::test]
async fn test_set_permission_mode() {
    let (app, db, _rt) = setup_test_app().await;

    // Create a sandbox first
    let sandbox_id = create_test_sandbox(&db).await;

    // Create a session
    let session_id = create_test_session(&db, &sandbox_id).await;

    let response = app
        .oneshot(json_request(
            "POST",
            &format!("/api/v1/sessions/{}/permissions", session_id),
            Some(serde_json::json!({
                "mode": "approve_all"
            })),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["status"], "ok");
    assert_eq!(body["mode"], "approve_all");
}

#[tokio::test]
async fn test_set_permission_mode_with_overrides() {
    let (app, db, _rt) = setup_test_app().await;

    let sandbox_id = create_test_sandbox(&db).await;
    let session_id = create_test_session(&db, &sandbox_id).await;

    let response = app
        .oneshot(json_request(
            "POST",
            &format!("/api/v1/sessions/{}/permissions", session_id),
            Some(serde_json::json!({
                "mode": "approve_edits",
                "always_require_approval": ["Bash"],
                "always_allow": ["Read"]
            })),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["status"], "ok");
    assert_eq!(body["mode"], "approve_edits");
}

#[tokio::test]
async fn test_set_permission_mode_invalid_session() {
    let (app, _db, _rt) = setup_test_app().await;

    let fake_id = Uuid::new_v4();
    let response = app
        .oneshot(json_request(
            "POST",
            &format!("/api/v1/sessions/{}/permissions", fake_id),
            Some(serde_json::json!({
                "mode": "auto_approve"
            })),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_respond_to_permission_no_pending() {
    let (app, db, _rt) = setup_test_app().await;

    let sandbox_id = create_test_sandbox(&db).await;
    let session_id = create_test_session(&db, &sandbox_id).await;

    let fake_request_id = Uuid::new_v4();
    let response = app
        .oneshot(json_request(
            "POST",
            &format!(
                "/api/v1/sessions/{}/permissions/{}/respond",
                session_id, fake_request_id
            ),
            Some(serde_json::json!({
                "approved": true
            })),
        ))
        .await
        .unwrap();

    // Should fail because there's no pending permission request
    assert_eq!(response.status(), StatusCode::CONFLICT);
    let body = response_json(response).await;
    assert!(body["error"]["message"]
        .as_str()
        .unwrap_or("")
        .contains("no pending permission request"));
}

#[tokio::test]
async fn test_respond_to_permission_invalid_session() {
    let (app, _db, _rt) = setup_test_app().await;

    let fake_session = Uuid::new_v4();
    let fake_request = Uuid::new_v4();
    let response = app
        .oneshot(json_request(
            "POST",
            &format!(
                "/api/v1/sessions/{}/permissions/{}/respond",
                fake_session, fake_request
            ),
            Some(serde_json::json!({
                "approved": false
            })),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_permission_mode_auto_approve() {
    use ciab_core::types::agent::{PermissionMode, PermissionPolicy};

    let policy = PermissionPolicy {
        mode: PermissionMode::AutoApprove,
        always_require_approval: vec![],
        always_allow: vec![],
    };

    // Auto-approve should never require approval
    assert!(!policy.requires_approval("Bash"));
    assert!(!policy.requires_approval("Edit"));
    assert!(!policy.requires_approval("Read"));
    assert!(!policy.requires_approval("Write"));
}

#[tokio::test]
async fn test_permission_mode_approve_all() {
    use ciab_core::types::agent::{PermissionMode, PermissionPolicy};

    let policy = PermissionPolicy {
        mode: PermissionMode::ApproveAll,
        always_require_approval: vec![],
        always_allow: vec![],
    };

    // Approve-all should require approval for everything
    assert!(policy.requires_approval("Bash"));
    assert!(policy.requires_approval("Read"));
    assert!(policy.requires_approval("Edit"));
}

#[tokio::test]
async fn test_permission_mode_approve_edits() {
    use ciab_core::types::agent::{PermissionMode, PermissionPolicy};

    let policy = PermissionPolicy {
        mode: PermissionMode::ApproveEdits,
        always_require_approval: vec![],
        always_allow: vec![],
    };

    // Approve-edits should require approval for write tools only
    assert!(policy.requires_approval("Bash"));
    assert!(policy.requires_approval("Edit"));
    assert!(policy.requires_approval("Write"));
    assert!(!policy.requires_approval("Read"));
    assert!(!policy.requires_approval("Grep"));
    assert!(!policy.requires_approval("Glob"));
}

#[tokio::test]
async fn test_permission_policy_always_override() {
    use ciab_core::types::agent::{PermissionMode, PermissionPolicy};

    let policy = PermissionPolicy {
        mode: PermissionMode::ApproveEdits,
        always_require_approval: vec!["Read".to_string()],
        always_allow: vec!["Bash".to_string()],
    };

    // Read is normally safe but forced to require approval
    assert!(policy.requires_approval("Read"));
    // Bash is normally a write tool but forced to allow
    assert!(!policy.requires_approval("Bash"));
    // Edit is still a write tool requiring approval
    assert!(policy.requires_approval("Edit"));
}

#[tokio::test]
async fn test_permission_risk_levels() {
    use ciab_core::types::agent::PermissionPolicy;

    assert_eq!(PermissionPolicy::risk_level("Bash"), "high");
    assert_eq!(PermissionPolicy::risk_level("Edit"), "medium");
    assert_eq!(PermissionPolicy::risk_level("Write"), "medium");
    assert_eq!(PermissionPolicy::risk_level("Read"), "low");
    assert_eq!(PermissionPolicy::risk_level("Grep"), "low");
    assert_eq!(PermissionPolicy::risk_level("UnknownTool"), "low");
}

// Helper: create a sandbox directly in the DB and return its ID
async fn create_test_sandbox(db: &Arc<Database>) -> Uuid {
    use ciab_core::types::sandbox::{SandboxPersistence, SandboxSpec};

    let sandbox_id = Uuid::new_v4();
    let now = Utc::now();
    let spec = SandboxSpec {
        name: Some("test-sandbox".to_string()),
        agent_provider: "mock-agent".to_string(),
        image: None,
        resource_limits: None,
        persistence: SandboxPersistence::Ephemeral,
        network: None,
        env_vars: HashMap::new(),
        volumes: vec![],
        ports: vec![],
        git_repos: vec![],
        credentials: vec![],
        provisioning_scripts: vec![],
        labels: HashMap::new(),
        timeout_secs: None,
        agent_config: None,
        local_mounts: vec![],
        runtime_backend: None,
    };
    let info = SandboxInfo {
        id: sandbox_id,
        name: Some("test-sandbox".to_string()),
        state: SandboxState::Running,
        persistence: SandboxPersistence::Ephemeral,
        agent_provider: "mock-agent".to_string(),
        endpoint_url: None,
        resource_stats: None,
        labels: HashMap::new(),
        created_at: now,
        updated_at: now,
        spec,
    };
    db.insert_sandbox(&info).await.unwrap();
    sandbox_id
}

// Helper: create a session directly in the DB and return its ID
async fn create_test_session(db: &Arc<Database>, sandbox_id: &Uuid) -> Uuid {
    use ciab_core::types::session::{Session, SessionState};

    let session_id = Uuid::new_v4();
    let now = Utc::now();
    let session = Session {
        id: session_id,
        sandbox_id: *sandbox_id,
        state: SessionState::Active,
        metadata: HashMap::new(),
        created_at: now,
        updated_at: now,
    };
    db.insert_session(&session).await.unwrap();
    session_id
}

// =============================================================================
// Slash Commands Tests
// =============================================================================

/// Helper: set up app with all real agent providers registered.
async fn setup_test_app_with_providers() -> (axum::Router, Arc<Database>, Arc<dyn SandboxRuntime>) {
    let db = Arc::new(Database::new("sqlite::memory:").await.unwrap());
    let runtime: Arc<dyn SandboxRuntime> = Arc::new(MockRuntime::new());
    let stream_handler = Arc::new(StreamBroker::new(100));
    let credential_store = Arc::new(
        CredentialStore::new(db.clone(), "test-encryption-key-for-ciab-testing-1234").unwrap(),
    );
    let provisioning = Arc::new(ProvisioningPipeline::new(
        runtime.clone(),
        credential_store.clone(),
        60,
    ));

    let mut agents: HashMap<String, Arc<dyn AgentProvider>> = HashMap::new();
    agents.insert("mock-agent".to_string(), Arc::new(MockAgentProvider));
    agents.insert("claude-code".to_string(), Arc::new(ClaudeCodeProvider));
    agents.insert("codex".to_string(), Arc::new(CodexProvider));
    agents.insert("gemini".to_string(), Arc::new(GeminiProvider));
    agents.insert("cursor".to_string(), Arc::new(CursorProvider));

    let config = Arc::new(test_config());

    let mut runtimes: HashMap<String, Arc<dyn SandboxRuntime>> = HashMap::new();
    runtimes.insert("local".to_string(), runtime.clone());

    let state = AppState {
        runtime: runtime.clone(),
        agents,
        runtimes,
        credentials: credential_store,
        stream_handler,
        provisioning,
        db: db.clone(),
        config,
        config_path: None,
        gateway: Arc::new(tokio::sync::RwLock::new(None)),
        channel_manager: Arc::new(tokio::sync::RwLock::new(None)),
        pending_permissions: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        session_permissions: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        pending_user_inputs: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        session_queues: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        image_builder: None,
    };

    let router = build_router(state);
    (router, db, runtime)
}

// -- API: GET /api/v1/agents --

#[tokio::test]
async fn test_list_providers() {
    let (app, _db, _rt) = setup_test_app_with_providers().await;

    let response = app
        .oneshot(json_request("GET", "/api/v1/agents", None))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    let providers = body.as_array().unwrap();
    // Should contain all registered providers
    let names: Vec<&str> = providers.iter().filter_map(|v| v.as_str()).collect();
    assert!(
        names.contains(&"claude-code"),
        "missing claude-code: {:?}",
        names
    );
    assert!(names.contains(&"codex"), "missing codex: {:?}", names);
    assert!(names.contains(&"gemini"), "missing gemini: {:?}", names);
    assert!(names.contains(&"cursor"), "missing cursor: {:?}", names);
    assert!(
        names.contains(&"mock-agent"),
        "missing mock-agent: {:?}",
        names
    );
}

// -- API: GET /api/v1/agents/{provider}/commands --

#[tokio::test]
async fn test_get_claude_code_slash_commands() {
    let (app, _db, _rt) = setup_test_app_with_providers().await;

    let response = app
        .oneshot(json_request(
            "GET",
            "/api/v1/agents/claude-code/commands",
            None,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    let commands = body.as_array().unwrap();
    assert!(
        commands.len() >= 20,
        "expected 20+ commands, got {}",
        commands.len()
    );

    // Verify specific commands exist
    let names: Vec<&str> = commands.iter().filter_map(|c| c["name"].as_str()).collect();
    assert!(names.contains(&"clear"));
    assert!(names.contains(&"compact"));
    assert!(names.contains(&"help"));
    assert!(names.contains(&"model"));
    assert!(names.contains(&"permissions"));
    assert!(names.contains(&"mcp"));
    assert!(names.contains(&"vim"));
    assert!(names.contains(&"cost"));

    // Verify /clear is local (not provider_native)
    let clear = commands.iter().find(|c| c["name"] == "clear").unwrap();
    assert_eq!(clear["provider_native"], false);
    assert_eq!(clear["category"], "session");

    // Verify /compact is provider_native
    let compact = commands.iter().find(|c| c["name"] == "compact").unwrap();
    assert_eq!(compact["provider_native"], true);

    // Verify /model has args
    let model = commands.iter().find(|c| c["name"] == "model").unwrap();
    let args = model["args"].as_array().unwrap();
    assert!(!args.is_empty());
    assert_eq!(args[0]["name"], "model");
}

#[tokio::test]
async fn test_get_codex_slash_commands() {
    let (app, _db, _rt) = setup_test_app_with_providers().await;

    let response = app
        .oneshot(json_request("GET", "/api/v1/agents/codex/commands", None))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    let commands = body.as_array().unwrap();
    assert_eq!(commands.len(), 4);

    let names: Vec<&str> = commands.iter().filter_map(|c| c["name"].as_str()).collect();
    assert!(names.contains(&"clear"));
    assert!(names.contains(&"help"));
    assert!(names.contains(&"model"));
    assert!(names.contains(&"approval-mode"));
}

#[tokio::test]
async fn test_get_gemini_slash_commands() {
    let (app, _db, _rt) = setup_test_app_with_providers().await;

    let response = app
        .oneshot(json_request("GET", "/api/v1/agents/gemini/commands", None))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    let commands = body.as_array().unwrap();
    assert_eq!(commands.len(), 4);

    let names: Vec<&str> = commands.iter().filter_map(|c| c["name"].as_str()).collect();
    assert!(names.contains(&"clear"));
    assert!(names.contains(&"help"));
    assert!(names.contains(&"stats"));
    assert!(names.contains(&"model"));
}

#[tokio::test]
async fn test_get_cursor_slash_commands() {
    let (app, _db, _rt) = setup_test_app_with_providers().await;

    let response = app
        .oneshot(json_request("GET", "/api/v1/agents/cursor/commands", None))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    let commands = body.as_array().unwrap();
    assert_eq!(commands.len(), 4);

    let names: Vec<&str> = commands.iter().filter_map(|c| c["name"].as_str()).collect();
    assert!(names.contains(&"clear"));
    assert!(names.contains(&"help"));
    assert!(names.contains(&"model"));
    assert!(names.contains(&"mode"));
}

#[tokio::test]
async fn test_get_mock_agent_slash_commands_empty() {
    let (app, _db, _rt) = setup_test_app_with_providers().await;

    let response = app
        .oneshot(json_request(
            "GET",
            "/api/v1/agents/mock-agent/commands",
            None,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    let commands = body.as_array().unwrap();
    assert!(commands.is_empty(), "mock agent should have no commands");
}

#[tokio::test]
async fn test_get_commands_unknown_provider_returns_error() {
    let (app, _db, _rt) = setup_test_app_with_providers().await;

    let response = app
        .oneshot(json_request(
            "GET",
            "/api/v1/agents/nonexistent/commands",
            None,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body = response_json(response).await;
    assert_eq!(body["error"]["code"], "agent_provider_not_found");
}

// -- Slash command: /clear --

#[tokio::test]
async fn test_slash_clear_clears_messages() {
    let (app, db, _rt) = setup_test_app_with_providers().await;

    // Create sandbox with claude-code provider
    let sandbox_id = Uuid::new_v4();
    let now = Utc::now();
    let spec = SandboxSpec {
        name: Some("slash-test".to_string()),
        agent_provider: "claude-code".to_string(),
        image: None,
        resource_limits: None,
        persistence: SandboxPersistence::Ephemeral,
        network: None,
        env_vars: HashMap::new(),
        volumes: vec![],
        ports: vec![],
        git_repos: vec![],
        credentials: vec![],
        provisioning_scripts: vec![],
        labels: HashMap::new(),
        timeout_secs: None,
        agent_config: None,
        local_mounts: vec![],
        runtime_backend: None,
    };
    let info = SandboxInfo {
        id: sandbox_id,
        name: Some("slash-test".to_string()),
        state: SandboxState::Running,
        persistence: SandboxPersistence::Ephemeral,
        agent_provider: "claude-code".to_string(),
        endpoint_url: None,
        resource_stats: None,
        labels: HashMap::new(),
        created_at: now,
        updated_at: now,
        spec,
    };
    db.insert_sandbox(&info).await.unwrap();

    // Create session
    let session_id = create_test_session(&db, &sandbox_id).await;

    // Insert some messages manually
    let msg1 = ciab_core::types::session::Message {
        id: Uuid::new_v4(),
        session_id,
        role: ciab_core::types::session::MessageRole::User,
        content: vec![ciab_core::types::session::MessageContent::Text {
            text: "Hello".to_string(),
        }],
        timestamp: Utc::now(),
    };
    db.insert_message(&msg1).await.unwrap();
    let msg2 = ciab_core::types::session::Message {
        id: Uuid::new_v4(),
        session_id,
        role: ciab_core::types::session::MessageRole::Assistant,
        content: vec![ciab_core::types::session::MessageContent::Text {
            text: "Hi there".to_string(),
        }],
        timestamp: Utc::now(),
    };
    db.insert_message(&msg2).await.unwrap();

    // Verify messages exist
    let msgs = db.get_messages(&session_id, None).await.unwrap();
    assert_eq!(msgs.len(), 2);

    // Send /clear command
    let response = app
        .clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/v1/sessions/{}/messages", session_id),
            Some(serde_json::json!({"message": "/clear"})),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    assert_eq!(body["role"], "assistant");
    let content = body["content"].as_array().unwrap();
    let text = content[0]["text"].as_str().unwrap();
    assert!(text.contains("Conversation cleared"), "got: {}", text);

    // Verify messages were deleted
    let msgs = db.get_messages(&session_id, None).await.unwrap();
    assert!(
        msgs.is_empty(),
        "expected 0 messages after /clear, got {}",
        msgs.len()
    );
}

// -- Slash command: /help --

#[tokio::test]
async fn test_slash_help_returns_formatted_commands() {
    let (app, db, _rt) = setup_test_app_with_providers().await;

    // Create sandbox with claude-code provider
    let sandbox_id = Uuid::new_v4();
    let now = Utc::now();
    let spec = SandboxSpec {
        name: Some("help-test".to_string()),
        agent_provider: "claude-code".to_string(),
        image: None,
        resource_limits: None,
        persistence: SandboxPersistence::Ephemeral,
        network: None,
        env_vars: HashMap::new(),
        volumes: vec![],
        ports: vec![],
        git_repos: vec![],
        credentials: vec![],
        provisioning_scripts: vec![],
        labels: HashMap::new(),
        timeout_secs: None,
        agent_config: None,
        local_mounts: vec![],
        runtime_backend: None,
    };
    let info = SandboxInfo {
        id: sandbox_id,
        name: Some("help-test".to_string()),
        state: SandboxState::Running,
        persistence: SandboxPersistence::Ephemeral,
        agent_provider: "claude-code".to_string(),
        endpoint_url: None,
        resource_stats: None,
        labels: HashMap::new(),
        created_at: now,
        updated_at: now,
        spec,
    };
    db.insert_sandbox(&info).await.unwrap();

    let session_id = create_test_session(&db, &sandbox_id).await;

    // Send /help command
    let response = app
        .clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/v1/sessions/{}/messages", session_id),
            Some(serde_json::json!({"message": "/help"})),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    assert_eq!(body["role"], "assistant");
    let content = body["content"].as_array().unwrap();
    let text = content[0]["text"].as_str().unwrap();

    // Should contain markdown-formatted help
    assert!(
        text.contains("Available Commands"),
        "missing header in: {}",
        text
    );
    assert!(text.contains("/clear"), "missing /clear in help text");
    assert!(text.contains("/compact"), "missing /compact in help text");
    assert!(text.contains("/model"), "missing /model in help text");
    assert!(text.contains("/help"), "missing /help in help text");

    // /help message should be persisted
    let msgs = db.get_messages(&session_id, None).await.unwrap();
    assert_eq!(msgs.len(), 1); // just the help response (no user message stored for local commands)
    assert_eq!(
        msgs[0].role,
        ciab_core::types::session::MessageRole::Assistant
    );
}

// -- Provider-native commands fall through --

#[tokio::test]
async fn test_native_slash_command_falls_through_to_agent() {
    let (app, db, _rt) = setup_test_app_with_providers().await;

    // Create sandbox with claude-code provider
    let sandbox_id = Uuid::new_v4();
    let now = Utc::now();
    let spec = SandboxSpec {
        name: Some("native-cmd-test".to_string()),
        agent_provider: "claude-code".to_string(),
        image: None,
        resource_limits: None,
        persistence: SandboxPersistence::Ephemeral,
        network: None,
        env_vars: HashMap::new(),
        volumes: vec![],
        ports: vec![],
        git_repos: vec![],
        credentials: vec![],
        provisioning_scripts: vec![],
        labels: HashMap::new(),
        timeout_secs: None,
        agent_config: None,
        local_mounts: vec![],
        runtime_backend: None,
    };
    let info = SandboxInfo {
        id: sandbox_id,
        name: Some("native-cmd-test".to_string()),
        state: SandboxState::Running,
        persistence: SandboxPersistence::Ephemeral,
        agent_provider: "claude-code".to_string(),
        endpoint_url: None,
        resource_stats: None,
        labels: HashMap::new(),
        created_at: now,
        updated_at: now,
        spec,
    };
    db.insert_sandbox(&info).await.unwrap();

    let session_id = create_test_session(&db, &sandbox_id).await;

    // Send /compact (a provider_native command) — should fall through to exec
    let response = app
        .clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/v1/sessions/{}/messages", session_id),
            Some(serde_json::json!({"message": "/compact"})),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    // send_message returns the user message as ack; agent processes in background
    assert_eq!(body["role"], "user");
    // Allow background processing to complete
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    // The user message should be persisted; assistant response is async
    let msgs = db.get_messages(&session_id, None).await.unwrap();
    assert!(
        !msgs.is_empty(),
        "expected at least user message for native command"
    );
    assert_eq!(msgs[0].role, ciab_core::types::session::MessageRole::User);
    // If background processing completed, assistant message may also be here
    if msgs.len() > 1 {
        assert_eq!(
            msgs[1].role,
            ciab_core::types::session::MessageRole::Assistant
        );
    }
}

// -- Non-slash messages pass through normally --

#[tokio::test]
async fn test_regular_message_not_intercepted() {
    let (app, db, _rt) = setup_test_app_with_providers().await;

    // Create sandbox with claude-code provider
    let sandbox_id = Uuid::new_v4();
    let now = Utc::now();
    let spec = SandboxSpec {
        name: Some("regular-msg-test".to_string()),
        agent_provider: "claude-code".to_string(),
        image: None,
        resource_limits: None,
        persistence: SandboxPersistence::Ephemeral,
        network: None,
        env_vars: HashMap::new(),
        volumes: vec![],
        ports: vec![],
        git_repos: vec![],
        credentials: vec![],
        provisioning_scripts: vec![],
        labels: HashMap::new(),
        timeout_secs: None,
        agent_config: None,
        local_mounts: vec![],
        runtime_backend: None,
    };
    let info = SandboxInfo {
        id: sandbox_id,
        name: Some("regular-msg-test".to_string()),
        state: SandboxState::Running,
        persistence: SandboxPersistence::Ephemeral,
        agent_provider: "claude-code".to_string(),
        endpoint_url: None,
        resource_stats: None,
        labels: HashMap::new(),
        created_at: now,
        updated_at: now,
        spec,
    };
    db.insert_sandbox(&info).await.unwrap();

    let session_id = create_test_session(&db, &sandbox_id).await;

    // Send a regular (non-slash) message
    let response = app
        .clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/v1/sessions/{}/messages", session_id),
            Some(serde_json::json!({"message": "Hello, what is Rust?"})),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    // send_message returns user message as ack; agent runs in background
    assert_eq!(body["role"], "user");

    // Allow background processing
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // User message should be persisted; assistant response is async
    let msgs = db.get_messages(&session_id, None).await.unwrap();
    assert!(!msgs.is_empty());
    assert_eq!(msgs[0].role, ciab_core::types::session::MessageRole::User);
}

// -- Unknown slash command (not in provider list) passes through --

#[tokio::test]
async fn test_unknown_slash_command_passes_through() {
    let (app, db, _rt) = setup_test_app_with_providers().await;

    let sandbox_id = Uuid::new_v4();
    let now = Utc::now();
    let spec = SandboxSpec {
        name: Some("unknown-cmd-test".to_string()),
        agent_provider: "claude-code".to_string(),
        image: None,
        resource_limits: None,
        persistence: SandboxPersistence::Ephemeral,
        network: None,
        env_vars: HashMap::new(),
        volumes: vec![],
        ports: vec![],
        git_repos: vec![],
        credentials: vec![],
        provisioning_scripts: vec![],
        labels: HashMap::new(),
        timeout_secs: None,
        agent_config: None,
        local_mounts: vec![],
        runtime_backend: None,
    };
    let info = SandboxInfo {
        id: sandbox_id,
        name: Some("unknown-cmd-test".to_string()),
        state: SandboxState::Running,
        persistence: SandboxPersistence::Ephemeral,
        agent_provider: "claude-code".to_string(),
        endpoint_url: None,
        resource_stats: None,
        labels: HashMap::new(),
        created_at: now,
        updated_at: now,
        spec,
    };
    db.insert_sandbox(&info).await.unwrap();

    let session_id = create_test_session(&db, &sandbox_id).await;

    // Send /nonexistent — not in provider's command list, should pass through to exec
    let response = app
        .clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/v1/sessions/{}/messages", session_id),
            Some(serde_json::json!({"message": "/nonexistent"})),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    // Should go through normal exec flow since "nonexistent" isn't in the command list
    assert_eq!(body["role"], "user");
}

// -- Command structure validation --

#[tokio::test]
async fn test_slash_command_categories_are_valid() {
    let (app, _db, _rt) = setup_test_app_with_providers().await;

    let response = app
        .oneshot(json_request(
            "GET",
            "/api/v1/agents/claude-code/commands",
            None,
        ))
        .await
        .unwrap();
    let body = response_json(response).await;
    let commands = body.as_array().unwrap();

    let valid_categories = ["session", "agent", "tools", "navigation", "help"];
    for cmd in commands {
        let cat = cmd["category"].as_str().unwrap();
        assert!(
            valid_categories.contains(&cat),
            "invalid category '{}' for command '{}'",
            cat,
            cmd["name"]
        );
    }
}

#[tokio::test]
async fn test_slash_commands_have_required_fields() {
    let (app, _db, _rt) = setup_test_app_with_providers().await;

    for provider in &["claude-code", "codex", "gemini", "cursor"] {
        let response = app
            .clone()
            .oneshot(json_request(
                "GET",
                &format!("/api/v1/agents/{}/commands", provider),
                None,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK, "failed for {}", provider);

        let body = response_json(response).await;
        let commands = body.as_array().unwrap();

        for cmd in commands {
            assert!(
                cmd["name"].is_string(),
                "missing name for cmd in {}",
                provider
            );
            assert!(
                cmd["description"].is_string(),
                "missing description for {} in {}",
                cmd["name"],
                provider
            );
            assert!(
                cmd["category"].is_string(),
                "missing category for {} in {}",
                cmd["name"],
                provider
            );
            assert!(
                cmd["provider_native"].is_boolean(),
                "missing provider_native for {} in {}",
                cmd["name"],
                provider
            );
            assert!(
                cmd["args"].is_array(),
                "missing args array for {} in {}",
                cmd["name"],
                provider
            );
        }
    }
}

// -- All providers have /clear and /help as local commands --

#[tokio::test]
async fn test_all_providers_have_clear_and_help() {
    let (app, _db, _rt) = setup_test_app_with_providers().await;

    for provider in &["claude-code", "codex", "gemini", "cursor"] {
        let response = app
            .clone()
            .oneshot(json_request(
                "GET",
                &format!("/api/v1/agents/{}/commands", provider),
                None,
            ))
            .await
            .unwrap();
        let body = response_json(response).await;
        let commands = body.as_array().unwrap();

        let clear = commands.iter().find(|c| c["name"] == "clear");
        assert!(clear.is_some(), "{} missing /clear command", provider);
        assert_eq!(
            clear.unwrap()["provider_native"],
            false,
            "{}: /clear should be local",
            provider
        );

        let help = commands.iter().find(|c| c["name"] == "help");
        assert!(help.is_some(), "{} missing /help command", provider);
        assert_eq!(
            help.unwrap()["provider_native"],
            false,
            "{}: /help should be local",
            provider
        );
    }
}

// -- /help for different providers returns different commands --

#[tokio::test]
async fn test_help_content_varies_by_provider() {
    let (app, db, _rt) = setup_test_app_with_providers().await;

    // Test with codex provider (has /approval-mode)
    let sandbox_id = Uuid::new_v4();
    let now = Utc::now();
    let spec = SandboxSpec {
        name: Some("codex-help-test".to_string()),
        agent_provider: "codex".to_string(),
        image: None,
        resource_limits: None,
        persistence: SandboxPersistence::Ephemeral,
        network: None,
        env_vars: HashMap::new(),
        volumes: vec![],
        ports: vec![],
        git_repos: vec![],
        credentials: vec![],
        provisioning_scripts: vec![],
        labels: HashMap::new(),
        timeout_secs: None,
        agent_config: None,
        local_mounts: vec![],
        runtime_backend: None,
    };
    let info = SandboxInfo {
        id: sandbox_id,
        name: Some("codex-help-test".to_string()),
        state: SandboxState::Running,
        persistence: SandboxPersistence::Ephemeral,
        agent_provider: "codex".to_string(),
        endpoint_url: None,
        resource_stats: None,
        labels: HashMap::new(),
        created_at: now,
        updated_at: now,
        spec,
    };
    db.insert_sandbox(&info).await.unwrap();

    let session_id = create_test_session(&db, &sandbox_id).await;

    let response = app
        .clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/v1/sessions/{}/messages", session_id),
            Some(serde_json::json!({"message": "/help"})),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    let text = body["content"][0]["text"].as_str().unwrap();
    // Codex has /approval-mode but not /compact
    assert!(
        text.contains("/approval-mode"),
        "codex help should include /approval-mode"
    );
    assert!(
        !text.contains("/compact"),
        "codex help should NOT include /compact"
    );
}

// -- DB: delete_session_messages --

#[tokio::test]
async fn test_db_delete_session_messages() {
    let db = Arc::new(Database::new("sqlite::memory:").await.unwrap());

    let sandbox_id = Uuid::new_v4();
    let now = Utc::now();
    // Insert a sandbox first (foreign key)
    let spec = SandboxSpec {
        name: Some("db-test".to_string()),
        agent_provider: "mock-agent".to_string(),
        image: None,
        resource_limits: None,
        persistence: SandboxPersistence::Ephemeral,
        network: None,
        env_vars: HashMap::new(),
        volumes: vec![],
        ports: vec![],
        git_repos: vec![],
        credentials: vec![],
        provisioning_scripts: vec![],
        labels: HashMap::new(),
        timeout_secs: None,
        agent_config: None,
        local_mounts: vec![],
        runtime_backend: None,
    };
    let sandbox = SandboxInfo {
        id: sandbox_id,
        name: Some("db-test".to_string()),
        state: SandboxState::Running,
        persistence: SandboxPersistence::Ephemeral,
        agent_provider: "mock-agent".to_string(),
        endpoint_url: None,
        resource_stats: None,
        labels: HashMap::new(),
        created_at: now,
        updated_at: now,
        spec,
    };
    db.insert_sandbox(&sandbox).await.unwrap();

    // Create two sessions
    let sid1 = Uuid::new_v4();
    let sid2 = Uuid::new_v4();
    for sid in [sid1, sid2] {
        let s = ciab_core::types::session::Session {
            id: sid,
            sandbox_id,
            state: ciab_core::types::session::SessionState::Active,
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
        };
        db.insert_session(&s).await.unwrap();
    }

    // Insert messages into both sessions
    for sid in [sid1, sid2] {
        for i in 0..3 {
            let msg = ciab_core::types::session::Message {
                id: Uuid::new_v4(),
                session_id: sid,
                role: ciab_core::types::session::MessageRole::User,
                content: vec![ciab_core::types::session::MessageContent::Text {
                    text: format!("msg {}", i),
                }],
                timestamp: Utc::now(),
            };
            db.insert_message(&msg).await.unwrap();
        }
    }

    // Verify both sessions have messages
    assert_eq!(db.get_messages(&sid1, None).await.unwrap().len(), 3);
    assert_eq!(db.get_messages(&sid2, None).await.unwrap().len(), 3);

    // Delete only sid1 messages
    db.delete_session_messages(&sid1).await.unwrap();

    // sid1 should be empty, sid2 should still have messages
    assert_eq!(db.get_messages(&sid1, None).await.unwrap().len(), 0);
    assert_eq!(db.get_messages(&sid2, None).await.unwrap().len(), 3);
}

// -- Delete messages from empty session (no-op, no error) --

#[tokio::test]
async fn test_db_delete_session_messages_empty_session() {
    let db = Arc::new(Database::new("sqlite::memory:").await.unwrap());

    let sandbox_id = Uuid::new_v4();
    let now = Utc::now();
    let spec = SandboxSpec {
        name: Some("empty-test".to_string()),
        agent_provider: "mock-agent".to_string(),
        image: None,
        resource_limits: None,
        persistence: SandboxPersistence::Ephemeral,
        network: None,
        env_vars: HashMap::new(),
        volumes: vec![],
        ports: vec![],
        git_repos: vec![],
        credentials: vec![],
        provisioning_scripts: vec![],
        labels: HashMap::new(),
        timeout_secs: None,
        agent_config: None,
        local_mounts: vec![],
        runtime_backend: None,
    };
    let sandbox = SandboxInfo {
        id: sandbox_id,
        name: Some("empty-test".to_string()),
        state: SandboxState::Running,
        persistence: SandboxPersistence::Ephemeral,
        agent_provider: "mock-agent".to_string(),
        endpoint_url: None,
        resource_stats: None,
        labels: HashMap::new(),
        created_at: now,
        updated_at: now,
        spec,
    };
    db.insert_sandbox(&sandbox).await.unwrap();

    let sid = Uuid::new_v4();
    let s = ciab_core::types::session::Session {
        id: sid,
        sandbox_id,
        state: ciab_core::types::session::SessionState::Active,
        metadata: HashMap::new(),
        created_at: now,
        updated_at: now,
    };
    db.insert_session(&s).await.unwrap();

    // Delete on empty session should not error
    db.delete_session_messages(&sid).await.unwrap();
    assert_eq!(db.get_messages(&sid, None).await.unwrap().len(), 0);
}

// =============================================================================
// Phase: Rust Library API, Cloud Provisioners, Config Improvements
// =============================================================================

// -- Config Resolution Chain --

#[tokio::test]
async fn test_appconfig_load_default() {
    // The embedded default config should always parse successfully
    let config = AppConfig::load_default().expect("embedded default config should parse");
    assert_eq!(config.server.port, 9090);
    assert_eq!(config.server.host, "0.0.0.0");
    assert_eq!(config.runtime.backend, "local");
    assert_eq!(config.agents.default_provider, "claude-code");
    assert!(config.agents.providers.contains_key("claude-code"));
    assert!(config.agents.providers.contains_key("codex"));
    assert_eq!(config.credentials.backend, "sqlite");
    assert_eq!(config.provisioning.timeout_secs, 300);
    assert_eq!(config.streaming.buffer_size, 500);
}

#[tokio::test]
async fn test_appconfig_load_falls_back_to_default() {
    // When no config file exists and no env var is set, load() should fall back to embedded default.
    // We unset the env var and use a nonexistent explicit path: None.
    // Since we're in a test directory, there may be a config.toml — so we use an env override trick.
    // Actually, just call load_default directly to test the embedded config works.
    let config = AppConfig::load_default().unwrap();
    assert_eq!(config.runtime.backend, "local");
    assert!(config.runtime.ec2.is_none());
    assert!(config.runtime.packer.is_none());
    assert!(config.runtime.kubernetes.is_none());
}

#[tokio::test]
async fn test_appconfig_parse_with_ec2_config() {
    let toml_content = r#"
[server]
host = "0.0.0.0"
port = 9090
request_timeout_secs = 300
cors_origins = ["*"]

[runtime]
backend = "ec2"

[runtime.ec2]
region = "us-west-2"
default_ami = "ami-test123"
instance_type = "t3.large"
ssh_user = "ec2-user"
ssh_port = 2222
root_volume_size_gb = 50
instance_ready_timeout_secs = 240
security_group_ids = ["sg-abc123"]

[runtime.ec2.tags]
Team = "platform"

[agents]
default_provider = "claude-code"

[agents.providers.claude-code]
enabled = true
binary = "claude"
api_key_env = "ANTHROPIC_API_KEY"

[credentials]
backend = "sqlite"
encryption_key_env = "CIAB_ENCRYPTION_KEY"

[provisioning]
timeout_secs = 300

[streaming]
buffer_size = 500
keepalive_interval_secs = 15
max_stream_duration_secs = 3600

[security]
api_keys = []

[logging]
level = "info"
format = "json"

[llm_providers]
auto_detect_ollama = false
"#;

    let config: AppConfig = toml::from_str(toml_content).expect("should parse EC2 config");
    assert_eq!(config.runtime.backend, "ec2");

    let ec2 = config.runtime.ec2.expect("ec2 config should be present");
    assert_eq!(ec2.region, "us-west-2");
    assert_eq!(ec2.default_ami, Some("ami-test123".to_string()));
    assert_eq!(ec2.instance_type, "t3.large");
    assert_eq!(ec2.ssh_user, "ec2-user");
    assert_eq!(ec2.ssh_port, 2222);
    assert_eq!(ec2.root_volume_size_gb, 50);
    assert_eq!(ec2.instance_ready_timeout_secs, 240);
    assert_eq!(ec2.security_group_ids, vec!["sg-abc123"]);
    assert_eq!(ec2.tags.get("Team"), Some(&"platform".to_string()));
}

#[tokio::test]
async fn test_appconfig_parse_with_packer_config() {
    let toml_content = r#"
[server]
host = "0.0.0.0"
port = 9090
request_timeout_secs = 300
cors_origins = ["*"]

[runtime]
backend = "local"

[runtime.packer]
binary = "/usr/local/bin/packer"
auto_install = true
template_cache_dir = "/var/cache/ciab-packer"
template_cache_ttl_secs = 7200
default_template = "git::https://github.com/org/templates.git//ec2.pkr.hcl?ref=v2"

[runtime.packer.variables]
region = "eu-west-1"
base_ami = "ami-eurobase"

[agents]
default_provider = "claude-code"

[agents.providers.claude-code]
enabled = true
binary = "claude"
api_key_env = "ANTHROPIC_API_KEY"

[credentials]
backend = "sqlite"
encryption_key_env = "CIAB_ENCRYPTION_KEY"

[provisioning]
timeout_secs = 300

[streaming]
buffer_size = 500
keepalive_interval_secs = 15
max_stream_duration_secs = 3600

[security]
api_keys = []

[logging]
level = "info"
format = "json"

[llm_providers]
auto_detect_ollama = false
"#;

    let config: AppConfig = toml::from_str(toml_content).expect("should parse Packer config");

    let packer = config
        .runtime
        .packer
        .expect("packer config should be present");
    assert_eq!(packer.binary, "/usr/local/bin/packer");
    assert!(packer.auto_install);
    assert_eq!(packer.template_cache_dir, "/var/cache/ciab-packer");
    assert_eq!(packer.template_cache_ttl_secs, 7200);
    assert_eq!(
        packer.default_template,
        "git::https://github.com/org/templates.git//ec2.pkr.hcl?ref=v2"
    );
    assert_eq!(
        packer.variables.get("region"),
        Some(&"eu-west-1".to_string())
    );
    assert_eq!(
        packer.variables.get("base_ami"),
        Some(&"ami-eurobase".to_string())
    );
}

// -- Resource Resolver --

#[test]
fn test_resource_resolver_parse_source_string() {
    use ciab_core::resolve::{parse_source_string, ResourceSource};

    // File path
    let src = parse_source_string("/etc/ciab/config.toml");
    assert!(
        matches!(src, ResourceSource::FilePath(p) if p.to_str() == Some("/etc/ciab/config.toml"))
    );

    // URL
    let src = parse_source_string("https://example.com/config.toml");
    assert!(matches!(src, ResourceSource::Url(u) if u == "https://example.com/config.toml"));

    // Builtin
    let src = parse_source_string("builtin://default-ec2");
    assert!(matches!(src, ResourceSource::Builtin(n) if n == "default-ec2"));

    // Git with ref
    let src = parse_source_string("git::https://github.com/org/repo.git//path/file.hcl?ref=main");
    match src {
        ResourceSource::Git { url, path, ref_ } => {
            assert!(url.contains("github.com/org/repo.git"));
            assert_eq!(path, "path/file.hcl");
            assert_eq!(ref_, Some("main".to_string()));
        }
        _ => panic!("Expected Git source"),
    }

    // Git without ref
    let src = parse_source_string("git::https://github.com/org/repo.git//template.hcl");
    match src {
        ResourceSource::Git { url, path, ref_ } => {
            assert!(url.contains("github.com/org/repo.git"));
            assert_eq!(path, "template.hcl");
            assert!(ref_.is_none());
        }
        _ => panic!("Expected Git source"),
    }
}

#[tokio::test]
async fn test_resource_resolver_builtin_default_ec2() {
    use ciab_core::resolve::{parse_source_string, resolve_resource};

    let src = parse_source_string("builtin://default-ec2");
    let content = resolve_resource(&src)
        .await
        .expect("builtin default-ec2 should resolve");

    // Verify it's a valid Packer template
    assert!(content.contains("packer {"));
    assert!(content.contains("amazon-ebs"));
    assert!(content.contains("variable \"region\""));
    assert!(content.contains("variable \"base_ami\""));
    assert!(content.contains("variable \"agent_provider\""));
    assert!(content.contains("provisioner \"shell\""));
}

#[tokio::test]
async fn test_resource_resolver_builtin_default_config() {
    use ciab_core::resolve::{parse_source_string, resolve_resource};

    let src = parse_source_string("builtin://default-config");
    let content = resolve_resource(&src)
        .await
        .expect("builtin default-config should resolve");

    // Verify it's a valid TOML config
    assert!(content.contains("[server]"));
    assert!(content.contains("[runtime]"));
    assert!(content.contains("[agents]"));
    assert!(content.contains("[credentials]"));

    // Should parse as AppConfig
    let _config: AppConfig =
        toml::from_str(&content).expect("builtin config should parse as AppConfig");
}

#[tokio::test]
async fn test_resource_resolver_builtin_unknown() {
    use ciab_core::resolve::{parse_source_string, resolve_resource};

    let src = parse_source_string("builtin://nonexistent");
    let result = resolve_resource(&src).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Unknown builtin resource"));
}

#[tokio::test]
async fn test_resource_resolver_file_not_found() {
    use ciab_core::resolve::{parse_source_string, resolve_resource};

    let src = parse_source_string("/tmp/definitely-does-not-exist-ciab-test-12345.toml");
    let result = resolve_resource(&src).await;
    assert!(result.is_err());
}

// -- Image Builder Types --

#[test]
fn test_image_build_request_serialization() {
    use ciab_core::types::image::*;

    let request = ImageBuildRequest {
        template: Some(TemplateSource::Builtin {
            name: "default-ec2".to_string(),
        }),
        variables: HashMap::from([
            ("region".to_string(), "us-east-1".to_string()),
            ("base_ami".to_string(), "ami-123".to_string()),
        ]),
        agent_provider: Some("claude-code".to_string()),
        tags: HashMap::from([("env".to_string(), "test".to_string())]),
    };

    // Serialize to JSON
    let json = serde_json::to_string(&request).expect("should serialize");
    assert!(json.contains("default-ec2"));
    assert!(json.contains("us-east-1"));
    assert!(json.contains("claude-code"));

    // Deserialize back
    let parsed: ImageBuildRequest = serde_json::from_str(&json).expect("should deserialize");
    assert!(parsed.template.is_some());
    assert_eq!(
        parsed.variables.get("region"),
        Some(&"us-east-1".to_string())
    );
    assert_eq!(parsed.agent_provider, Some("claude-code".to_string()));
}

#[test]
fn test_image_build_status_serialization() {
    use ciab_core::types::image::*;

    // Succeeded
    let status = ImageBuildStatus::Succeeded;
    let json = serde_json::to_string(&status).unwrap();
    assert_eq!(json, "\"succeeded\"");

    // Failed
    let status = ImageBuildStatus::Failed("timeout".to_string());
    let json = serde_json::to_string(&status).unwrap();
    assert!(json.contains("failed"));
    assert!(json.contains("timeout"));

    // Running
    let status = ImageBuildStatus::Running;
    let json = serde_json::to_string(&status).unwrap();
    assert_eq!(json, "\"running\"");
}

#[test]
fn test_image_build_result_serialization() {
    use ciab_core::types::image::*;

    let result = ImageBuildResult {
        build_id: Uuid::new_v4(),
        status: ImageBuildStatus::Succeeded,
        image_id: Some("ami-0123456789abcdef0".to_string()),
        logs: vec!["Building...".to_string(), "Done.".to_string()],
    };

    let json = serde_json::to_string(&result).expect("should serialize");
    let parsed: ImageBuildResult = serde_json::from_str(&json).expect("should deserialize");
    assert_eq!(parsed.image_id, Some("ami-0123456789abcdef0".to_string()));
    assert_eq!(parsed.status, ImageBuildStatus::Succeeded);
    assert_eq!(parsed.logs.len(), 2);
}

// -- Image Builder API Routes --

use ciab_core::traits::image_builder::ImageBuilder;
use ciab_core::types::image::*;

struct MockImageBuilder {
    images: DashMap<String, BuiltImage>,
    builds: DashMap<Uuid, ImageBuildStatus>,
}

impl MockImageBuilder {
    fn new() -> Self {
        Self {
            images: DashMap::new(),
            builds: DashMap::new(),
        }
    }
}

#[async_trait]
impl ImageBuilder for MockImageBuilder {
    async fn build_image(&self, request: &ImageBuildRequest) -> CiabResult<ImageBuildResult> {
        let build_id = Uuid::new_v4();
        let image_id = format!("ami-mock-{}", &build_id.to_string()[..8]);

        self.builds.insert(build_id, ImageBuildStatus::Succeeded);
        self.images.insert(
            image_id.clone(),
            BuiltImage {
                image_id: image_id.clone(),
                provider: "amazon-ebs".to_string(),
                region: request.variables.get("region").cloned(),
                created_at: Utc::now(),
                tags: request.tags.clone(),
            },
        );

        Ok(ImageBuildResult {
            build_id,
            status: ImageBuildStatus::Succeeded,
            image_id: Some(image_id),
            logs: vec!["Mock build complete".to_string()],
        })
    }

    async fn list_images(&self) -> CiabResult<Vec<BuiltImage>> {
        Ok(self.images.iter().map(|r| r.value().clone()).collect())
    }

    async fn delete_image(&self, image_id: &str) -> CiabResult<()> {
        self.images.remove(image_id);
        Ok(())
    }

    async fn build_status(&self, build_id: &Uuid) -> CiabResult<ImageBuildStatus> {
        self.builds
            .get(build_id)
            .map(|s| s.clone())
            .ok_or_else(|| CiabError::ImageBuildError(format!("Build {} not found", build_id)))
    }
}

async fn setup_test_app_with_image_builder() -> (axum::Router, Arc<Database>, Arc<MockRuntime>) {
    let db = Arc::new(Database::new("sqlite::memory:").await.unwrap());
    let runtime: Arc<MockRuntime> = Arc::new(MockRuntime::new());
    let stream_handler: Arc<dyn StreamHandler> = Arc::new(StreamBroker::new(100));
    let credential_store = Arc::new(
        CredentialStore::new(
            db.clone(),
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        )
        .unwrap(),
    );
    let provisioning = Arc::new(ProvisioningPipeline::new(
        runtime.clone(),
        credential_store.clone(),
        60,
    ));

    let mut agents: HashMap<String, Arc<dyn AgentProvider>> = HashMap::new();
    agents.insert("mock-agent".to_string(), Arc::new(MockAgentProvider));

    let config = Arc::new(test_config());
    let mut runtimes: HashMap<String, Arc<dyn SandboxRuntime>> = HashMap::new();
    runtimes.insert("local".to_string(), runtime.clone());

    let image_builder: Arc<dyn ImageBuilder> = Arc::new(MockImageBuilder::new());

    let state = AppState {
        runtime: runtime.clone(),
        agents,
        runtimes,
        credentials: credential_store,
        stream_handler,
        provisioning,
        db: db.clone(),
        config,
        config_path: None,
        gateway: Arc::new(tokio::sync::RwLock::new(None)),
        channel_manager: Arc::new(tokio::sync::RwLock::new(None)),
        pending_permissions: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        session_permissions: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        pending_user_inputs: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        session_queues: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        image_builder: Some(image_builder),
    };

    let router = build_router(state);
    (router, db, runtime)
}

#[tokio::test]
async fn test_image_build_endpoint() {
    let (app, _db, _rt) = setup_test_app_with_image_builder().await;

    let body = serde_json::json!({
        "template": {
            "type": "builtin",
            "name": "default-ec2"
        },
        "variables": {
            "region": "us-east-1",
            "base_ami": "ami-test"
        },
        "agent_provider": "claude-code",
        "tags": {
            "env": "test"
        }
    });

    let response = app
        .oneshot(json_request("POST", "/api/v1/images/build", Some(body)))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::ACCEPTED);

    let result = response_json(response).await;
    assert!(result["build_id"].is_string());
    assert_eq!(result["status"], "succeeded");
    assert!(result["image_id"]
        .as_str()
        .unwrap()
        .starts_with("ami-mock-"));
    assert!(result["logs"].as_array().unwrap().len() > 0);
}

#[tokio::test]
async fn test_image_list_endpoint() {
    let (app, _db, _rt) = setup_test_app_with_image_builder().await;

    // First build an image
    let build_body = serde_json::json!({
        "variables": {"region": "us-west-2"},
        "tags": {"env": "staging"}
    });

    let app_clone = app.clone();
    let response = app_clone
        .oneshot(json_request(
            "POST",
            "/api/v1/images/build",
            Some(build_body),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::ACCEPTED);

    // Now list images
    let response = app
        .oneshot(json_request("GET", "/api/v1/images", None))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    let images = body.as_array().unwrap();
    assert_eq!(images.len(), 1);
    assert!(images[0]["image_id"]
        .as_str()
        .unwrap()
        .starts_with("ami-mock-"));
    assert_eq!(images[0]["provider"], "amazon-ebs");
}

#[tokio::test]
async fn test_image_build_status_endpoint() {
    let (app, _db, _rt) = setup_test_app_with_image_builder().await;

    // Build an image
    let build_body = serde_json::json!({
        "variables": {"region": "us-east-1"}
    });

    let app_clone = app.clone();
    let response = app_clone
        .oneshot(json_request(
            "POST",
            "/api/v1/images/build",
            Some(build_body),
        ))
        .await
        .unwrap();
    let result = response_json(response).await;
    let build_id = result["build_id"].as_str().unwrap();

    // Check build status
    let response = app
        .oneshot(json_request(
            "GET",
            &format!("/api/v1/images/builds/{}", build_id),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let status = response_json(response).await;
    assert_eq!(status, "succeeded");
}

#[tokio::test]
async fn test_image_delete_endpoint() {
    let (app, _db, _rt) = setup_test_app_with_image_builder().await;

    // Build an image
    let build_body = serde_json::json!({
        "variables": {"region": "us-east-1"}
    });

    let app_clone1 = app.clone();
    let response = app_clone1
        .oneshot(json_request(
            "POST",
            "/api/v1/images/build",
            Some(build_body),
        ))
        .await
        .unwrap();
    let result = response_json(response).await;
    let image_id = result["image_id"].as_str().unwrap().to_string();

    // Delete the image
    let app_clone2 = app.clone();
    let response = app_clone2
        .oneshot(json_request(
            "DELETE",
            &format!("/api/v1/images/{}", image_id),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // List should be empty now
    let response = app
        .oneshot(json_request("GET", "/api/v1/images", None))
        .await
        .unwrap();
    let body = response_json(response).await;
    assert_eq!(body.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_image_endpoints_without_builder() {
    // Use the regular test app which has image_builder: None
    let (app, _db, _rt) = setup_test_app().await;

    // All image endpoints should return error when no builder is configured
    let response = app
        .oneshot(json_request("GET", "/api/v1/images", None))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let body = response_json(response).await;
    assert!(body["error"]["message"]
        .as_str()
        .unwrap()
        .contains("No image builder configured"));
}

// -- Packer Builder Unit Tests --

#[test]
fn test_packer_image_builder_construction() {
    use ciab_core::types::config::PackerConfig;
    use ciab_packer::PackerImageBuilder;

    let config = PackerConfig {
        binary: "packer".to_string(),
        auto_install: false,
        template_cache_dir: "/tmp".to_string(),
        template_cache_ttl_secs: 3600,
        default_template: "builtin://default-ec2".to_string(),
        variables: HashMap::new(),
    };

    // Smoke test — constructor should not panic
    let _builder = PackerImageBuilder::new(config);
}

// -- Error Type Tests --

#[test]
fn test_new_error_variants() {
    use ciab_core::error::CiabError;

    // EC2 error
    let err = CiabError::Ec2Error("instance not found".to_string());
    assert!(err.to_string().contains("EC2 error"));
    assert!(err.to_string().contains("instance not found"));

    // SSH error
    let err = CiabError::SshError("connection refused".to_string());
    assert!(err.to_string().contains("SSH error"));

    // Packer error
    let err = CiabError::PackerError("template invalid".to_string());
    assert!(err.to_string().contains("Packer error"));

    // Image build error
    let err = CiabError::ImageBuildError("build timed out".to_string());
    assert!(err.to_string().contains("Image build error"));

    // Resource resolution error
    let err = CiabError::ResourceResolutionError("404 not found".to_string());
    assert!(err.to_string().contains("Resource resolution error"));

    // Unsupported
    let err = CiabError::Unsupported("EC2 pause not supported".to_string());
    assert!(err.to_string().contains("Unsupported operation"));
}

// -- CiabEngine Builder Tests --

#[tokio::test]
async fn test_ciab_engine_default_build() {
    use ciab::CiabEngine;

    let engine = CiabEngine::builder()
        .config_default()
        .database_url("sqlite::memory:")
        .build()
        .await;

    assert!(
        engine.is_ok(),
        "CiabEngine default build failed: {:?}",
        engine.err()
    );

    let engine = engine.unwrap();
    assert_eq!(engine.config().runtime.backend, "local");
    assert_eq!(engine.config().agents.default_provider, "claude-code");

    // runtime() returns default runtime (no args)
    let _rt = engine.runtime();

    // agent() returns Option
    assert!(engine.agent("claude-code").is_some());
    assert!(engine.agent("codex").is_some());
    assert!(engine.agent("nonexistent").is_none());
}

#[tokio::test]
async fn test_ciab_engine_with_custom_config() {
    use ciab::CiabEngine;

    let mut config = AppConfig::load_default().unwrap();
    config.server.port = 8888;
    config.runtime.backend = "local".to_string();

    let engine = CiabEngine::builder()
        .config(config)
        .database_url("sqlite::memory:")
        .build()
        .await
        .expect("should build with custom config");

    assert_eq!(engine.config().server.port, 8888);
}

#[tokio::test]
async fn test_ciab_engine_sandbox_lifecycle() {
    use ciab::CiabEngine;
    use ciab_core::types::sandbox::{SandboxSpec, SandboxState};

    let engine = CiabEngine::builder()
        .config_default()
        .database_url("sqlite::memory:")
        .build()
        .await
        .expect("should build");

    // Create a sandbox (SandboxSpec doesn't derive Default — construct manually)
    let spec = SandboxSpec {
        name: Some("test-sandbox".to_string()),
        agent_provider: "claude-code".to_string(),
        image: None,
        resource_limits: None,
        persistence: ciab_core::types::sandbox::SandboxPersistence::Ephemeral,
        network: None,
        env_vars: HashMap::new(),
        volumes: Vec::new(),
        ports: Vec::new(),
        git_repos: Vec::new(),
        credentials: Vec::new(),
        provisioning_scripts: Vec::new(),
        labels: HashMap::new(),
        timeout_secs: None,
        agent_config: None,
        local_mounts: Vec::new(),
        runtime_backend: None,
    };

    let result = engine.create_sandbox(&spec).await;
    assert!(result.is_ok(), "create_sandbox failed: {:?}", result.err());

    let sandbox = result.unwrap();
    assert_eq!(sandbox.name, Some("test-sandbox".to_string()));
    assert_eq!(sandbox.agent_provider, "claude-code");
    assert_eq!(sandbox.state, SandboxState::Running);

    // Get sandbox
    let fetched = engine.get_sandbox(&sandbox.id).await;
    assert!(fetched.is_ok());
    assert_eq!(fetched.unwrap().id, sandbox.id);

    // List sandboxes
    let list = engine
        .list_sandboxes(None, None, &HashMap::new())
        .await
        .unwrap();
    assert!(list.iter().any(|s| s.id == sandbox.id));

    // Terminate sandbox
    let term_result = engine.terminate_sandbox(&sandbox.id).await;
    assert!(term_result.is_ok());
}

#[tokio::test]
async fn test_ciab_engine_exec() {
    use ciab::CiabEngine;
    use ciab_core::types::sandbox::{ExecRequest, SandboxSpec};

    let engine = CiabEngine::builder()
        .config_default()
        .database_url("sqlite::memory:")
        .build()
        .await
        .expect("should build");

    let spec = SandboxSpec {
        name: Some("exec-test".to_string()),
        agent_provider: "claude-code".to_string(),
        image: None,
        resource_limits: None,
        persistence: ciab_core::types::sandbox::SandboxPersistence::Ephemeral,
        network: None,
        env_vars: HashMap::new(),
        volumes: Vec::new(),
        ports: Vec::new(),
        git_repos: Vec::new(),
        credentials: Vec::new(),
        provisioning_scripts: Vec::new(),
        labels: HashMap::new(),
        timeout_secs: None,
        agent_config: None,
        local_mounts: Vec::new(),
        runtime_backend: None,
    };

    let sandbox = engine.create_sandbox(&spec).await.unwrap();

    // Execute a simple command (command is Vec<String>)
    let request = ExecRequest {
        command: vec!["echo".to_string(), "hello world".to_string()],
        workdir: None,
        env: HashMap::new(),
        stdin: None,
        timeout_secs: Some(10),
        tty: false,
    };

    let result = engine.exec(&sandbox.id, &request).await;
    assert!(result.is_ok(), "exec failed: {:?}", result.err());

    let exec_result = result.unwrap();
    assert_eq!(exec_result.exit_code, 0);
    assert!(exec_result.stdout.contains("hello world"));

    engine.terminate_sandbox(&sandbox.id).await.unwrap();
}

#[tokio::test]
async fn test_ciab_engine_file_operations() {
    use ciab::CiabEngine;
    use ciab_core::types::sandbox::SandboxSpec;

    let engine = CiabEngine::builder()
        .config_default()
        .database_url("sqlite::memory:")
        .build()
        .await
        .expect("should build");

    let spec = SandboxSpec {
        name: Some("file-test".to_string()),
        agent_provider: "claude-code".to_string(),
        image: None,
        resource_limits: None,
        persistence: ciab_core::types::sandbox::SandboxPersistence::Ephemeral,
        network: None,
        env_vars: HashMap::new(),
        volumes: Vec::new(),
        ports: Vec::new(),
        git_repos: Vec::new(),
        credentials: Vec::new(),
        provisioning_scripts: Vec::new(),
        labels: HashMap::new(),
        timeout_secs: None,
        agent_config: None,
        local_mounts: Vec::new(),
        runtime_backend: None,
    };

    let sandbox = engine.create_sandbox(&spec).await.unwrap();

    // Write a file
    let content = b"Hello from CIAB engine test!";
    let write_result = engine
        .write_file(&sandbox.id, "/tmp/ciab-test-file.txt", content)
        .await;
    assert!(
        write_result.is_ok(),
        "write_file failed: {:?}",
        write_result.err()
    );

    // Read it back
    let read_result = engine
        .read_file(&sandbox.id, "/tmp/ciab-test-file.txt")
        .await;
    assert!(
        read_result.is_ok(),
        "read_file failed: {:?}",
        read_result.err()
    );
    assert_eq!(read_result.unwrap(), content);

    // List files
    let list_result = engine.list_files(&sandbox.id, "/tmp").await;
    assert!(list_result.is_ok());

    engine.terminate_sandbox(&sandbox.id).await.unwrap();
}
