use std::collections::HashMap;
use std::sync::Arc;

use uuid::Uuid;

use ciab_core::error::{CiabError, CiabResult};
use ciab_core::traits::agent::AgentProvider;
#[cfg(feature = "packer")]
use ciab_core::traits::image_builder::ImageBuilder;
use ciab_core::traits::runtime::SandboxRuntime;
use ciab_core::types::config::AppConfig;
#[cfg(feature = "packer")]
use ciab_core::types::image::{BuiltImage, ImageBuildRequest, ImageBuildResult};
use ciab_core::types::sandbox::{
    ExecRequest, ExecResult, FileInfo, SandboxInfo, SandboxSpec, SandboxState,
};
use ciab_credentials::CredentialStore;
use ciab_db::Database;
use ciab_provisioning::ProvisioningPipeline;
use ciab_streaming::StreamBroker;

/// The main CIAB facade — a single entry point for managing coding agent sandboxes.
pub struct CiabEngine {
    config: AppConfig,
    default_runtime: Arc<dyn SandboxRuntime>,
    runtimes: HashMap<String, Arc<dyn SandboxRuntime>>,
    agents: HashMap<String, Arc<dyn AgentProvider>>,
    provisioning: ProvisioningPipeline,
    db: Arc<Database>,
    stream_broker: Arc<StreamBroker>,
    credential_store: Arc<CredentialStore>,
    #[cfg(feature = "packer")]
    image_builder: Option<Arc<dyn ImageBuilder>>,
}

impl CiabEngine {
    /// Create a new builder.
    pub fn builder() -> CiabEngineBuilder {
        CiabEngineBuilder::default()
    }

    // -------------------------------------------------------------------------
    // Accessors
    // -------------------------------------------------------------------------

    /// The resolved configuration.
    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    /// The default runtime backend.
    pub fn runtime(&self) -> &Arc<dyn SandboxRuntime> {
        &self.default_runtime
    }

    /// Look up an agent provider by name.
    pub fn agent(&self, name: &str) -> Option<&Arc<dyn AgentProvider>> {
        self.agents.get(name)
    }

    /// The database handle.
    pub fn db(&self) -> &Arc<Database> {
        &self.db
    }

    /// The SSE/stream broker.
    pub fn stream_broker(&self) -> &Arc<StreamBroker> {
        &self.stream_broker
    }

    /// The encrypted credential store.
    pub fn credential_store(&self) -> &Arc<CredentialStore> {
        &self.credential_store
    }

    /// All registered runtimes.
    pub fn runtimes(&self) -> &HashMap<String, Arc<dyn SandboxRuntime>> {
        &self.runtimes
    }

    /// All registered agent providers.
    pub fn agents(&self) -> &HashMap<String, Arc<dyn AgentProvider>> {
        &self.agents
    }

    /// The provisioning pipeline.
    pub fn provisioning(&self) -> &ProvisioningPipeline {
        &self.provisioning
    }

    // -------------------------------------------------------------------------
    // Runtime resolution
    // -------------------------------------------------------------------------

    /// Resolve the runtime to use for a given sandbox spec.
    /// Uses spec.runtime_backend if set, otherwise falls back to the default.
    fn resolve_runtime(&self, spec: &SandboxSpec) -> CiabResult<Arc<dyn SandboxRuntime>> {
        if let Some(ref backend) = spec.runtime_backend {
            self.runtimes
                .get(backend)
                .cloned()
                .ok_or_else(|| CiabError::ConfigError(format!("runtime not found: {}", backend)))
        } else {
            Ok(self.default_runtime.clone())
        }
    }

    // -------------------------------------------------------------------------
    // Sandbox lifecycle
    // -------------------------------------------------------------------------

    /// Create a new sandbox from the given spec.
    pub async fn create_sandbox(&self, spec: &SandboxSpec) -> CiabResult<SandboxInfo> {
        let runtime = self.resolve_runtime(spec)?;
        runtime.create_sandbox(spec).await
    }

    /// Get info about a sandbox by ID.
    /// Tries the default runtime first, then iterates all runtimes.
    pub async fn get_sandbox(&self, id: &Uuid) -> CiabResult<SandboxInfo> {
        // Try default runtime first
        match self.default_runtime.get_sandbox(id).await {
            Ok(info) => return Ok(info),
            Err(CiabError::SandboxNotFound(_)) => {}
            Err(e) => return Err(e),
        }

        // Try all other runtimes
        for runtime in self.runtimes.values() {
            match runtime.get_sandbox(id).await {
                Ok(info) => return Ok(info),
                Err(CiabError::SandboxNotFound(_)) => continue,
                Err(e) => return Err(e),
            }
        }

        Err(CiabError::SandboxNotFound(id.to_string()))
    }

    /// List all sandboxes, optionally filtered.
    pub async fn list_sandboxes(
        &self,
        state: Option<SandboxState>,
        provider: Option<&str>,
        labels: &HashMap<String, String>,
    ) -> CiabResult<Vec<SandboxInfo>> {
        self.default_runtime
            .list_sandboxes(state, provider, labels)
            .await
    }

    /// Start a sandbox.
    pub async fn start_sandbox(&self, id: &Uuid) -> CiabResult<()> {
        self.default_runtime.start_sandbox(id).await
    }

    /// Stop a sandbox.
    pub async fn stop_sandbox(&self, id: &Uuid) -> CiabResult<()> {
        self.default_runtime.stop_sandbox(id).await
    }

    /// Terminate and remove a sandbox.
    pub async fn terminate_sandbox(&self, id: &Uuid) -> CiabResult<()> {
        self.default_runtime.terminate_sandbox(id).await
    }

    // -------------------------------------------------------------------------
    // Execution
    // -------------------------------------------------------------------------

    /// Execute a command inside a sandbox.
    pub async fn exec(&self, id: &Uuid, request: &ExecRequest) -> CiabResult<ExecResult> {
        self.default_runtime.exec(id, request).await
    }

    // -------------------------------------------------------------------------
    // Files
    // -------------------------------------------------------------------------

    /// Read a file from a sandbox.
    pub async fn read_file(&self, id: &Uuid, path: &str) -> CiabResult<Vec<u8>> {
        self.default_runtime.read_file(id, path).await
    }

    /// Write a file to a sandbox.
    pub async fn write_file(&self, id: &Uuid, path: &str, content: &[u8]) -> CiabResult<()> {
        self.default_runtime.write_file(id, path, content).await
    }

    /// List files in a directory inside a sandbox.
    pub async fn list_files(&self, id: &Uuid, path: &str) -> CiabResult<Vec<FileInfo>> {
        self.default_runtime.list_files(id, path).await
    }

    // -------------------------------------------------------------------------
    // Image building (packer feature)
    // -------------------------------------------------------------------------

    /// Build a machine image (requires `packer` feature).
    #[cfg(feature = "packer")]
    pub async fn build_image(&self, request: &ImageBuildRequest) -> CiabResult<ImageBuildResult> {
        let builder = self
            .image_builder
            .as_ref()
            .ok_or_else(|| CiabError::ConfigError("no image builder configured".to_string()))?;
        builder.build_image(request).await
    }

    /// List built images (requires `packer` feature).
    #[cfg(feature = "packer")]
    pub async fn list_images(&self) -> CiabResult<Vec<BuiltImage>> {
        let builder = self
            .image_builder
            .as_ref()
            .ok_or_else(|| CiabError::ConfigError("no image builder configured".to_string()))?;
        builder.list_images().await
    }

    // -------------------------------------------------------------------------
    // Provisioning
    // -------------------------------------------------------------------------

    /// Run the full provisioning pipeline for a sandbox.
    pub async fn provision_sandbox(
        &self,
        spec: &SandboxSpec,
        agent: &dyn AgentProvider,
        tx: tokio::sync::mpsc::Sender<ciab_core::types::stream::StreamEvent>,
    ) -> CiabResult<SandboxInfo> {
        self.provisioning.provision(spec, agent, tx).await
    }
}

// =============================================================================
// Builder
// =============================================================================

/// Builder for [`CiabEngine`].
#[derive(Default)]
pub struct CiabEngineBuilder {
    config: Option<AppConfig>,
    config_source: Option<String>,
    database_url: Option<String>,
    runtimes: HashMap<String, Arc<dyn SandboxRuntime>>,
    agents: HashMap<String, Arc<dyn AgentProvider>>,
    #[cfg(feature = "packer")]
    image_builder: Option<Arc<dyn ImageBuilder>>,
}

impl CiabEngineBuilder {
    /// Provide an explicit config.
    pub fn config(mut self, config: AppConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// Load config from a file path or URL.
    pub fn config_from_file(mut self, path: &str) -> Self {
        self.config_source = Some(path.to_string());
        self
    }

    /// Load config from a URL.
    pub fn config_from_url(mut self, url: &str) -> Self {
        self.config_source = Some(url.to_string());
        self
    }

    /// Use the embedded default config.
    pub fn config_default(mut self) -> Self {
        self.config_source = Some("__default__".to_string());
        self
    }

    /// Register a runtime backend under a name (e.g., "local", "ec2", "kubernetes").
    pub fn runtime(mut self, name: &str, runtime: Arc<dyn SandboxRuntime>) -> Self {
        self.runtimes.insert(name.to_string(), runtime);
        self
    }

    /// Register an agent provider under a name (e.g., "claude-code", "codex").
    pub fn agent(mut self, name: &str, agent: Arc<dyn AgentProvider>) -> Self {
        self.agents.insert(name.to_string(), agent);
        self
    }

    /// Set the database URL (default: "sqlite:ciab.db?mode=rwc").
    pub fn database_url(mut self, url: &str) -> Self {
        self.database_url = Some(url.to_string());
        self
    }

    /// Set a custom image builder (requires `packer` feature).
    #[cfg(feature = "packer")]
    pub fn image_builder(mut self, builder: Arc<dyn ImageBuilder>) -> Self {
        self.image_builder = Some(builder);
        self
    }

    /// Build the engine, resolving config and initializing all subsystems.
    pub async fn build(self) -> CiabResult<CiabEngine> {
        // 1. Resolve config
        let config = if let Some(cfg) = self.config {
            cfg
        } else if let Some(ref source) = self.config_source {
            if source == "__default__" {
                AppConfig::load_default().map_err(|e| {
                    CiabError::ConfigError(format!("Failed to parse default config: {}", e))
                })?
            } else {
                AppConfig::load(Some(source)).await?
            }
        } else {
            AppConfig::load(None).await?
        };

        // 2. Init database
        let db_url = self
            .database_url
            .unwrap_or_else(|| "sqlite:ciab.db?mode=rwc".to_string());
        let db = Arc::new(Database::new(&db_url).await?);

        // 3. Init stream broker
        let stream_broker = Arc::new(StreamBroker::new(config.streaming.buffer_size));

        // 4. Init credential store
        let encryption_key =
            std::env::var(&config.credentials.encryption_key_env).unwrap_or_else(|_| {
                // Generate a random key for development/testing
                use rand::Rng;
                let key: [u8; 32] = rand::thread_rng().gen();
                hex::encode(key)
            });
        let credential_store = Arc::new(CredentialStore::new(db.clone(), &encryption_key)?);

        // 5. Auto-register runtimes from config if not manually provided
        let mut runtimes = self.runtimes;

        #[cfg(feature = "local")]
        {
            if !runtimes.contains_key("local") {
                let rt = ciab_sandbox::LocalProcessRuntime::new(
                    config.runtime.local_workdir.clone(),
                    config.runtime.local_max_processes,
                );
                runtimes.insert("local".to_string(), Arc::new(rt));
            }
        }

        #[cfg(feature = "ec2")]
        {
            if !runtimes.contains_key("ec2") {
                if let Some(ref ec2_config) = config.runtime.ec2 {
                    let rt = ciab_sandbox_ec2::Ec2Runtime::new(ec2_config.clone()).await?;
                    runtimes.insert("ec2".to_string(), Arc::new(rt));
                }
            }
        }

        #[cfg(feature = "kubernetes")]
        {
            if !runtimes.contains_key("kubernetes") {
                if let Some(ref kc) = config.runtime.kubernetes {
                    let k8s_cfg = ciab_sandbox_k8s::KubernetesRuntimeConfig {
                        kubeconfig: kc.kubeconfig.clone(),
                        context: kc.context.clone(),
                        namespace: kc.namespace.clone(),
                        agent_image: kc.agent_image.clone(),
                        runtime_class: kc.runtime_class.clone(),
                        node_selector: kc.node_selector.clone(),
                        tolerations: Vec::new(),
                        image_pull_secrets: Vec::new(),
                        storage_class: kc.storage_class.clone(),
                        workspace_pvc_size: kc.workspace_pvc_size.clone(),
                        service_account: kc.service_account.clone(),
                        create_network_policy: kc.create_network_policy,
                        run_as_non_root: kc.run_as_non_root,
                        drop_all_capabilities: kc.drop_all_capabilities,
                        default_cpu_request: kc.default_cpu_request.clone(),
                        default_cpu_limit: kc.default_cpu_limit.clone(),
                        default_memory_request: kc.default_memory_request.clone(),
                        default_memory_limit: kc.default_memory_limit.clone(),
                    };
                    let rt = ciab_sandbox_k8s::KubernetesRuntime::new(k8s_cfg)
                        .await
                        .map_err(|e| CiabError::KubernetesError(e.to_string()))?;
                    runtimes.insert("kubernetes".to_string(), Arc::new(rt));
                }
            }
        }

        // 6. Select default runtime from config
        let default_backend = &config.runtime.backend;
        let default_runtime = runtimes.get(default_backend).cloned().ok_or_else(|| {
            CiabError::ConfigError(format!(
                "default runtime backend '{}' not registered (available: {:?})",
                default_backend,
                runtimes.keys().collect::<Vec<_>>()
            ))
        })?;

        // 7. Auto-register agents from config if not manually provided
        let mut agents = self.agents;
        if agents.is_empty() {
            agents.insert(
                "claude-code".to_string(),
                Arc::new(ciab_agent_claude::ClaudeCodeProvider),
            );
            agents.insert(
                "codex".to_string(),
                Arc::new(ciab_agent_codex::CodexProvider),
            );
            agents.insert(
                "gemini".to_string(),
                Arc::new(ciab_agent_gemini::GeminiProvider),
            );
            agents.insert(
                "cursor".to_string(),
                Arc::new(ciab_agent_cursor::CursorProvider),
            );
        }

        // 8. Init provisioning pipeline
        let provisioning = ProvisioningPipeline::new(
            default_runtime.clone(),
            credential_store.clone(),
            config.provisioning.timeout_secs,
        );

        // 9. Auto-create packer image builder if configured
        #[cfg(feature = "packer")]
        let image_builder = if self.image_builder.is_some() {
            self.image_builder
        } else if let Some(ref packer_config) = config.runtime.packer {
            Some(Arc::new(ciab_packer::PackerImageBuilder::new(
                packer_config.clone(),
            )) as Arc<dyn ImageBuilder>)
        } else {
            None
        };

        Ok(CiabEngine {
            config,
            default_runtime,
            runtimes,
            agents,
            provisioning,
            db,
            stream_broker,
            credential_store,
            #[cfg(feature = "packer")]
            image_builder,
        })
    }
}
