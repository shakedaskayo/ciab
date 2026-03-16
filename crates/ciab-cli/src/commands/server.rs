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

            if app_config.runtime.backend == "kubernetes" || app_config.runtime.kubernetes.is_some()
            {
                let kc = app_config.runtime.kubernetes.clone().unwrap_or_default();
                let k8s_cfg = ciab_sandbox_k8s::KubernetesRuntimeConfig {
                    kubeconfig: kc.kubeconfig,
                    context: kc.context,
                    namespace: kc.namespace,
                    agent_image: kc.agent_image,
                    runtime_class: kc.runtime_class,
                    node_selector: kc.node_selector,
                    tolerations: Vec::new(),
                    image_pull_secrets: Vec::new(),
                    storage_class: kc.storage_class,
                    workspace_pvc_size: kc.workspace_pvc_size,
                    service_account: kc.service_account,
                    create_network_policy: kc.create_network_policy,
                    run_as_non_root: kc.run_as_non_root,
                    drop_all_capabilities: kc.drop_all_capabilities,
                    default_cpu_request: kc.default_cpu_request,
                    default_cpu_limit: kc.default_cpu_limit,
                    default_memory_request: kc.default_memory_request,
                    default_memory_limit: kc.default_memory_limit,
                };
                match ciab_sandbox_k8s::KubernetesRuntime::new(k8s_cfg).await {
                    Ok(k8s_runtime) => {
                        let k8s_runtime: Arc<dyn ciab_core::traits::runtime::SandboxRuntime> =
                            Arc::new(k8s_runtime);
                        runtimes.insert("kubernetes".to_string(), k8s_runtime);
                        tracing::info!("Kubernetes runtime initialized");
                    }
                    Err(e) => {
                        tracing::warn!(error = %e, "failed to initialize Kubernetes runtime");
                    }
                }
            }

            // Select default runtime from config
            let runtime: Arc<dyn ciab_core::traits::runtime::SandboxRuntime> =
                match app_config.runtime.backend.as_str() {
                    "opensandbox" => runtimes
                        .get("opensandbox")
                        .cloned()
                        .unwrap_or(local_runtime),
                    "kubernetes" => runtimes.get("kubernetes").cloned().unwrap_or(local_runtime),
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

            // 7b. Seed LLM providers from config (on first run).
            {
                let existing_providers = db
                    .list_llm_providers()
                    .await
                    .map_err(|e| anyhow::anyhow!("listing llm providers: {}", e))?;

                if existing_providers.is_empty() {
                    // Seed from config
                    for (name, seed) in &app_config.llm_providers.providers {
                        let kind: ciab_core::types::llm_provider::LlmProviderKind = seed
                            .kind
                            .parse()
                            .map_err(|e: String| anyhow::anyhow!("{}", e))?;

                        let is_local =
                            kind == ciab_core::types::llm_provider::LlmProviderKind::Ollama;

                        // If api_key_env is set, try to read from environment
                        let mut api_key_credential_id = None;
                        if let Some(ref env_var) = seed.api_key_env {
                            if let Ok(key_value) = std::env::var(env_var) {
                                let cred = credential_store
                                    .store_credential(
                                        &format!("llm-{}-key", name),
                                        ciab_core::types::credentials::CredentialType::ApiKey,
                                        key_value.as_bytes(),
                                        std::collections::HashMap::new(),
                                        None,
                                    )
                                    .await
                                    .map_err(|e| anyhow::anyhow!("storing credential: {}", e))?;
                                api_key_credential_id = Some(cred.id);
                            }
                        }

                        let now = chrono::Utc::now();
                        let provider = ciab_core::types::llm_provider::LlmProvider {
                            id: uuid::Uuid::new_v4(),
                            name: name.clone(),
                            kind,
                            enabled: true,
                            base_url: seed.base_url.clone(),
                            api_key_credential_id,
                            default_model: seed.default_model.clone(),
                            is_local,
                            auto_detected: false,
                            extra: std::collections::HashMap::new(),
                            created_at: now,
                            updated_at: now,
                        };
                        db.insert_llm_provider(&provider)
                            .await
                            .map_err(|e| anyhow::anyhow!("inserting llm provider: {}", e))?;
                        tracing::info!(name = %name, kind = %provider.kind, "seeded LLM provider from config");
                    }

                    // Auto-detect Ollama
                    if app_config.llm_providers.auto_detect_ollama {
                        if let Some(client) = ciab_api::ollama::OllamaClient::detect().await {
                            let version = client.version().await.ok();
                            let now = chrono::Utc::now();
                            let provider = ciab_core::types::llm_provider::LlmProvider {
                                id: uuid::Uuid::new_v4(),
                                name: "Ollama (local)".to_string(),
                                kind: ciab_core::types::llm_provider::LlmProviderKind::Ollama,
                                enabled: true,
                                base_url: Some(client.base_url().to_string()),
                                api_key_credential_id: None,
                                default_model: None,
                                is_local: true,
                                auto_detected: true,
                                extra: if let Some(v) = version {
                                    [("version".to_string(), serde_json::Value::String(v))]
                                        .into_iter()
                                        .collect()
                                } else {
                                    std::collections::HashMap::new()
                                },
                                created_at: now,
                                updated_at: now,
                            };
                            db.insert_llm_provider(&provider)
                                .await
                                .map_err(|e| anyhow::anyhow!("inserting ollama provider: {}", e))?;
                            tracing::info!("auto-detected Ollama at {}", client.base_url());
                        }
                    }
                }
            }

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
                stream_handler,
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
