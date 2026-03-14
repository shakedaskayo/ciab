use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{Context, Result};

use ciab_core::traits::agent::AgentProvider;
use ciab_core::types::config::AppConfig;
use tokio::sync::RwLock;

use super::ServerCommand;

pub async fn execute(command: ServerCommand) -> Result<()> {
    match command {
        ServerCommand::Start {
            config,
            database_url,
        } => {
            // 1. Load config from TOML file.
            let config_path = config.clone();
            let config_content = tokio::fs::read_to_string(&config)
                .await
                .with_context(|| format!("reading config file: {}", config))?;
            let app_config: AppConfig =
                toml::from_str(&config_content).with_context(|| "parsing config TOML")?;
            let app_config = Arc::new(app_config);

            // 2. Initialize Database.
            let db = ciab_db::Database::new(&database_url)
                .await
                .map_err(|e| anyhow::anyhow!("database init failed: {}", e))?;
            let db = Arc::new(db);

            // 3. Initialize Runtime(s).
            // Always create local runtime; create opensandbox if configured.
            let mut runtimes: HashMap<String, Arc<dyn ciab_core::traits::runtime::SandboxRuntime>> =
                HashMap::new();

            let local_runtime: Arc<dyn ciab_core::traits::runtime::SandboxRuntime> =
                Arc::new(ciab_sandbox::LocalProcessRuntime::new(
                    app_config.runtime.local_workdir.clone(),
                    app_config.runtime.local_max_processes,
                ));
            runtimes.insert("local".to_string(), local_runtime.clone());

            if app_config.runtime.opensandbox_url.is_some() {
                let url = app_config
                    .runtime
                    .opensandbox_url
                    .clone()
                    .unwrap_or_else(|| "http://localhost:8000".to_string());
                let opensandbox_runtime: Arc<dyn ciab_core::traits::runtime::SandboxRuntime> =
                    Arc::new(ciab_sandbox::OpenSandboxRuntime::new(
                        url,
                        app_config.runtime.opensandbox_api_key.clone(),
                    ));
                runtimes.insert("opensandbox".to_string(), opensandbox_runtime);
            }

            // Select default runtime from config
            let runtime: Arc<dyn ciab_core::traits::runtime::SandboxRuntime> =
                match app_config.runtime.backend.as_str() {
                    "opensandbox" => runtimes
                        .get("opensandbox")
                        .cloned()
                        .unwrap_or(local_runtime),
                    _ => runtimes.get("local").cloned().unwrap(),
                };

            // 4. Initialize StreamBroker.
            let stream_broker = ciab_streaming::StreamBroker::new(app_config.streaming.buffer_size);
            let stream_handler = Arc::new(stream_broker);

            // 5. Initialize CredentialStore.
            let encryption_key = std::env::var(&app_config.credentials.encryption_key_env)
                .unwrap_or_else(|_| {
                    tracing::warn!(
                        "Encryption key env var '{}' not set, using default (NOT SECURE)",
                        app_config.credentials.encryption_key_env
                    );
                    "0000000000000000000000000000000000000000000000000000000000000000".to_string()
                });
            let credential_store =
                ciab_credentials::CredentialStore::new(db.clone(), &encryption_key)
                    .map_err(|e| anyhow::anyhow!("credential store init failed: {}", e))?;
            let credential_store = Arc::new(credential_store);

            // 6. Initialize ProvisioningPipeline.
            let provisioning = ciab_provisioning::ProvisioningPipeline::new(
                runtime.clone(),
                credential_store.clone(),
                app_config.provisioning.timeout_secs,
            );
            let provisioning = Arc::new(provisioning);

            // 7. Register agent providers.
            let mut agents: HashMap<String, Arc<dyn AgentProvider>> = HashMap::new();

            let claude_provider = ciab_agent_claude::ClaudeCodeProvider;
            agents.insert("claude-code".to_string(), Arc::new(claude_provider));

            let codex_provider = ciab_agent_codex::CodexProvider;
            agents.insert("codex".to_string(), Arc::new(codex_provider));

            let gemini_provider = ciab_agent_gemini::GeminiProvider;
            agents.insert("gemini".to_string(), Arc::new(gemini_provider));

            let cursor_provider = ciab_agent_cursor::CursorProvider;
            agents.insert("cursor".to_string(), Arc::new(cursor_provider));

            // 8. Initialize GatewayManager (if enabled).
            let gateway = if app_config.gateway.enabled {
                let gw = ciab_gateway::GatewayManager::new(app_config.gateway.clone(), db.clone());
                gw.start()
                    .await
                    .map_err(|e| anyhow::anyhow!("gateway init failed: {}", e))?;
                tracing::info!("Gateway subsystem initialized");
                Some(Arc::new(gw))
            } else {
                None
            };

            // 9. Build AppState.
            let state = ciab_api::AppState {
                runtime,
                agents,
                runtimes,
                credentials: credential_store,
                stream_handler: stream_handler,
                provisioning,
                db,
                config: app_config,
                config_path: Some(config_path),
                gateway: Arc::new(RwLock::new(gateway)),
                channel_manager: Arc::new(RwLock::new(None)),
                pending_permissions: Arc::new(RwLock::new(std::collections::HashMap::new())),
                session_permissions: Arc::new(RwLock::new(std::collections::HashMap::new())),
                pending_user_inputs: Arc::new(RwLock::new(std::collections::HashMap::new())),
                session_queues: Arc::new(RwLock::new(std::collections::HashMap::new())),
            };

            // 10. Start the API server.
            println!(
                "Starting ciab server on {}:{}",
                state.config.server.host, state.config.server.port
            );

            ciab_api::start_server(state)
                .await
                .map_err(|e| anyhow::anyhow!("server error: {}", e))?;

            Ok(())
        }
    }
}
