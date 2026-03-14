use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use chrono::Utc;
use tokio::sync::mpsc;
use uuid::Uuid;

use ciab_core::error::{CiabError, CiabResult};
use ciab_core::traits::agent::AgentProvider;
use ciab_core::traits::runtime::SandboxRuntime;
use ciab_core::types::sandbox::{ExecRequest, SandboxInfo, SandboxSpec};
use ciab_core::types::stream::{StreamEvent, StreamEventType};
use ciab_credentials::injection::CredentialInjector;
use ciab_credentials::CredentialStore;

use crate::agentfs;
use crate::git;
use crate::local_mount;
use crate::scripts;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProvisioningStep {
    Validate,
    PrepareImage,
    ResolveCredentials,
    CreateSandbox,
    StartSandbox,
    MountLocalDirs,
    InjectCredentials,
    CloneRepositories,
    SetupAgentFs,
    RunScripts,
    StartAgent,
}

impl fmt::Display for ProvisioningStep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Validate => "validate",
            Self::PrepareImage => "prepare_image",
            Self::ResolveCredentials => "resolve_credentials",
            Self::CreateSandbox => "create_sandbox",
            Self::StartSandbox => "start_sandbox",
            Self::MountLocalDirs => "mount_local_dirs",
            Self::InjectCredentials => "inject_credentials",
            Self::CloneRepositories => "clone_repositories",
            Self::SetupAgentFs => "setup_agentfs",
            Self::RunScripts => "run_scripts",
            Self::StartAgent => "start_agent",
        };
        write!(f, "{}", s)
    }
}

pub struct ProvisioningPipeline {
    runtime: Arc<dyn SandboxRuntime>,
    credential_store: Arc<CredentialStore>,
    timeout_secs: u64,
}

impl ProvisioningPipeline {
    pub fn new(
        runtime: Arc<dyn SandboxRuntime>,
        credential_store: Arc<CredentialStore>,
        timeout_secs: u64,
    ) -> Self {
        Self {
            runtime,
            credential_store,
            timeout_secs,
        }
    }

    /// Run the full 9-step provisioning pipeline.
    ///
    /// If `pre_assigned_id` is provided, provisioning events will use that ID
    /// so that SSE subscribers (who received this ID from the create endpoint)
    /// can follow along from the start. Once the runtime creates the real
    /// sandbox the actual ID is used for subsequent events.
    pub async fn provision(
        &self,
        spec: &SandboxSpec,
        agent: &dyn AgentProvider,
        tx: mpsc::Sender<StreamEvent>,
    ) -> CiabResult<SandboxInfo> {
        self.provision_with_id(spec, agent, tx, None).await
    }

    /// Same as `provision` but with an optional pre-assigned sandbox ID.
    pub async fn provision_with_id(
        &self,
        spec: &SandboxSpec,
        agent: &dyn AgentProvider,
        tx: mpsc::Sender<StreamEvent>,
        pre_assigned_id: Option<Uuid>,
    ) -> CiabResult<SandboxInfo> {
        let placeholder_id = pre_assigned_id.unwrap_or_else(Uuid::new_v4);

        // We'll track the actual sandbox id once created, for cleanup on error
        let mut created_sandbox_id: Option<Uuid> = None;

        let result = self
            .provision_inner(spec, agent, &tx, placeholder_id, &mut created_sandbox_id)
            .await;

        match result {
            Ok(info) => Ok(info),
            Err(e) => {
                // Send provisioning failed event
                let sandbox_id = created_sandbox_id.unwrap_or(placeholder_id);
                let fail_event = StreamEvent {
                    id: format!("prov-{}", Uuid::new_v4()),
                    sandbox_id,
                    session_id: None,
                    event_type: StreamEventType::ProvisioningFailed,
                    data: serde_json::json!({ "error": e.to_string() }),
                    timestamp: Utc::now(),
                };
                let _ = tx.send(fail_event).await;

                // Attempt cleanup
                if let Some(sid) = created_sandbox_id {
                    tracing::warn!(sandbox_id = %sid, "provisioning failed, attempting cleanup");
                    if let Err(cleanup_err) = self.runtime.terminate_sandbox(&sid).await {
                        tracing::error!(
                            sandbox_id = %sid,
                            error = %cleanup_err,
                            "failed to clean up sandbox after provisioning failure"
                        );
                    }
                }

                Err(e)
            }
        }
    }

    async fn provision_inner(
        &self,
        spec: &SandboxSpec,
        agent: &dyn AgentProvider,
        tx: &mpsc::Sender<StreamEvent>,
        placeholder_id: Uuid,
        created_sandbox_id: &mut Option<Uuid>,
    ) -> CiabResult<SandboxInfo> {
        // Step 1: VALIDATE
        self.validate_spec(spec, agent)?;
        let _ = tx
            .send(self.step_event(placeholder_id, ProvisioningStep::Validate, "spec validated"))
            .await;
        tracing::info!("provisioning step: validate complete");

        // Step 2: PREPARE IMAGE
        let image = spec
            .image
            .clone()
            .unwrap_or_else(|| agent.base_image().to_string());
        let _ = tx
            .send(self.step_event(
                placeholder_id,
                ProvisioningStep::PrepareImage,
                &format!("image: {}", image),
            ))
            .await;
        tracing::info!(image = %image, "provisioning step: prepare_image complete");

        // Step 3: RESOLVE CREDENTIALS
        let resolved_env_vars =
            CredentialInjector::resolve_env_vars(&self.credential_store, &spec.credentials).await?;
        let resolved_files =
            CredentialInjector::resolve_files(&self.credential_store, &spec.credentials).await?;
        let _ = tx
            .send(self.step_event(
                placeholder_id,
                ProvisioningStep::ResolveCredentials,
                &format!(
                    "resolved {} env vars, {} files",
                    resolved_env_vars.len(),
                    resolved_files.len()
                ),
            ))
            .await;
        tracing::info!(
            env_vars = resolved_env_vars.len(),
            files = resolved_files.len(),
            "provisioning step: resolve_credentials complete"
        );

        // Step 4: CREATE SANDBOX
        let mut resolved_spec = spec.clone();
        resolved_spec.image = Some(image);
        let sandbox_info = self.runtime.create_sandbox(&resolved_spec).await?;
        let sandbox_id = sandbox_info.id;
        *created_sandbox_id = Some(sandbox_id);
        let _ = tx
            .send(self.step_event(
                sandbox_id,
                ProvisioningStep::CreateSandbox,
                &format!("sandbox_id: {}", sandbox_id),
            ))
            .await;
        tracing::info!(sandbox_id = %sandbox_id, "provisioning step: create_sandbox complete");

        // Step 5: START SANDBOX
        self.runtime.start_sandbox(&sandbox_id).await?;
        let _ = tx
            .send(self.step_event(
                sandbox_id,
                ProvisioningStep::StartSandbox,
                "sandbox started",
            ))
            .await;
        tracing::info!(sandbox_id = %sandbox_id, "provisioning step: start_sandbox complete");

        // Step 5.5: MOUNT LOCAL DIRECTORIES
        for mount in &spec.local_mounts {
            local_mount::mount_local_dir(self.runtime.as_ref(), &sandbox_id, mount).await?;
            let _ = tx
                .send(self.step_event(
                    sandbox_id,
                    ProvisioningStep::MountLocalDirs,
                    &format!("mounted {} -> {}", mount.source, mount.dest_path),
                ))
                .await;
        }
        if !spec.local_mounts.is_empty() {
            tracing::info!(
                sandbox_id = %sandbox_id,
                mounts = spec.local_mounts.len(),
                "provisioning step: mount_local_dirs complete"
            );
        }

        // Step 6: INJECT CREDENTIALS
        if !resolved_env_vars.is_empty() {
            // Inject env vars by writing them and sourcing, or via exec with env
            // We inject env vars by writing a profile script that exports them
            let mut env_script = String::from("#!/bin/sh\n");
            for (key, value) in &resolved_env_vars {
                // Escape single quotes in values
                let escaped_value = value.replace('\'', "'\\''");
                env_script.push_str(&format!("export {}='{}'\n", key, escaped_value));
            }
            self.runtime
                .write_file(
                    &sandbox_id,
                    "/etc/profile.d/ciab-credentials.sh",
                    env_script.as_bytes(),
                )
                .await?;
        }

        // Upload credential files
        for (path, content) in &resolved_files {
            self.runtime.write_file(&sandbox_id, path, content).await?;
        }

        let _ = tx
            .send(self.step_event(
                sandbox_id,
                ProvisioningStep::InjectCredentials,
                &format!(
                    "injected {} env vars, {} files",
                    resolved_env_vars.len(),
                    resolved_files.len()
                ),
            ))
            .await;
        tracing::info!(sandbox_id = %sandbox_id, "provisioning step: inject_credentials complete");

        // Step 7: CLONE REPOSITORIES
        for repo in &spec.git_repos {
            git::provision_repo(self.runtime.as_ref(), &sandbox_id, repo).await?;
            let strategy = repo.strategy.as_deref().unwrap_or("clone");
            let _ = tx
                .send(self.step_event(
                    sandbox_id,
                    ProvisioningStep::CloneRepositories,
                    &format!(
                        "{} {} ({})",
                        if strategy == "worktree" {
                            "worktree"
                        } else {
                            "cloned"
                        },
                        repo.url,
                        strategy
                    ),
                ))
                .await;
        }
        tracing::info!(
            sandbox_id = %sandbox_id,
            repos = spec.git_repos.len(),
            "provisioning step: clone_repositories complete"
        );

        // Step 7.5: SETUP AGENTFS
        // AgentFS config is passed via labels (from workspace spec serialization)
        let agentfs_enabled = spec
            .labels
            .get("ciab/agentfs_enabled")
            .map(|v| v == "true")
            .unwrap_or(false);

        if agentfs_enabled {
            let agentfs_binary = spec
                .labels
                .get("ciab/agentfs_binary")
                .cloned()
                .unwrap_or_else(|| "agentfs".to_string());
            let agentfs_db_path = spec
                .labels
                .get("ciab/agentfs_db_path")
                .cloned()
                .unwrap_or_else(|| format!("/tmp/ciab-agentfs-{}.db", sandbox_id));

            // Check if agentfs is available
            let available = agentfs::check_agentfs_available(
                self.runtime.as_ref(),
                &sandbox_id,
                &agentfs_binary,
            )
            .await?;

            if available {
                agentfs::init_agentfs_db(
                    self.runtime.as_ref(),
                    &sandbox_id,
                    &agentfs_binary,
                    &agentfs_db_path,
                )
                .await?;
                let _ = tx
                    .send(self.step_event(
                        sandbox_id,
                        ProvisioningStep::SetupAgentFs,
                        &format!("agentfs initialized at {}", agentfs_db_path),
                    ))
                    .await;
            } else {
                tracing::warn!(
                    sandbox_id = %sandbox_id,
                    "agentfs binary not found, skipping CoW isolation"
                );
                let _ = tx
                    .send(self.step_event(
                        sandbox_id,
                        ProvisioningStep::SetupAgentFs,
                        "agentfs binary not found, skipping",
                    ))
                    .await;
            }
        }
        tracing::info!(sandbox_id = %sandbox_id, "provisioning step: setup_agentfs complete");

        // Step 8: RUN SCRIPTS
        for (i, script_content) in spec.provisioning_scripts.iter().enumerate() {
            scripts::run_script(
                self.runtime.as_ref(),
                &sandbox_id,
                script_content,
                tx.clone(),
                sandbox_id,
            )
            .await?;
            let _ = tx
                .send(self.step_event(
                    sandbox_id,
                    ProvisioningStep::RunScripts,
                    &format!(
                        "completed script {}/{}",
                        i + 1,
                        spec.provisioning_scripts.len()
                    ),
                ))
                .await;
        }
        tracing::info!(
            sandbox_id = %sandbox_id,
            scripts = spec.provisioning_scripts.len(),
            "provisioning step: run_scripts complete"
        );

        // Step 9: START AGENT
        let agent_config =
            spec.agent_config
                .clone()
                .unwrap_or_else(|| ciab_core::types::agent::AgentConfig {
                    provider: spec.agent_provider.clone(),
                    model: None,
                    system_prompt: None,
                    max_tokens: None,
                    temperature: None,
                    tools_enabled: true,
                    mcp_servers: Vec::new(),
                    allowed_tools: Vec::new(),
                    denied_tools: Vec::new(),
                    extra: HashMap::new(),
                });

        let mut agent_cmd = agent.build_start_command(&agent_config);

        // Wrap with agentfs if enabled
        if agentfs_enabled {
            let agentfs_binary = spec
                .labels
                .get("ciab/agentfs_binary")
                .cloned()
                .unwrap_or_else(|| "agentfs".to_string());
            let agentfs_db_path = spec
                .labels
                .get("ciab/agentfs_db_path")
                .cloned()
                .unwrap_or_else(|| format!("/tmp/ciab-agentfs-{}.db", sandbox_id));
            let agentfs_logging = spec
                .labels
                .get("ciab/agentfs_logging")
                .map(|v| v == "true")
                .unwrap_or(true);

            let (wrapped_cmd, wrapped_args) = agentfs::wrap_command_with_agentfs(
                &agent_cmd.command,
                &agent_cmd.args,
                &agentfs_binary,
                &agentfs_db_path,
                agentfs_logging,
            );
            agent_cmd.command = wrapped_cmd;
            agent_cmd.args = wrapped_args;
        }

        // Build the command with env vars (including resolved credentials)
        let mut exec_env = agent_cmd.env.clone();
        exec_env.extend(resolved_env_vars);
        // Clear CLAUDECODE to avoid nested-session detection in dev environments
        exec_env.insert("CLAUDECODE".to_string(), String::new());

        let exec_request = ExecRequest {
            command: {
                let mut cmd = vec![agent_cmd.command.clone()];
                cmd.extend(agent_cmd.args.clone());
                cmd
            },
            workdir: agent_cmd.workdir.clone(),
            env: exec_env,
            stdin: None,
            timeout_secs: Some(self.timeout_secs as u32),
            tty: false,
        };

        // In local mode, the agent binary may not be a persistent daemon — it's
        // invoked per-request in send_message.  We attempt to run the start command
        // but treat failure as non-fatal (the agent will be started on first message).
        match self.runtime.exec(&sandbox_id, &exec_request).await {
            Ok(exec_result) if exec_result.exit_code == 0 => {
                tracing::info!(sandbox_id = %sandbox_id, "agent start command succeeded");
            }
            Ok(exec_result) => {
                tracing::warn!(
                    sandbox_id = %sandbox_id,
                    exit_code = exec_result.exit_code,
                    stderr = %exec_result.stderr,
                    "agent start command exited non-zero (will start on first message)"
                );
            }
            Err(e) => {
                tracing::warn!(
                    sandbox_id = %sandbox_id,
                    error = %e,
                    "agent start command failed (will start on first message)"
                );
            }
        }

        // Health check — non-fatal; agent may not be running yet in local mode.
        match agent.health_check(&sandbox_id).await {
            Ok(health) if health.healthy => {
                let _ = tx
                    .send(self.step_event(
                        sandbox_id,
                        ProvisioningStep::StartAgent,
                        &format!("agent started, health: {}", health.status),
                    ))
                    .await;
            }
            Ok(health) => {
                tracing::warn!(sandbox_id = %sandbox_id, status = %health.status, "agent health check not healthy (non-fatal)");
                let _ = tx
                    .send(self.step_event(
                        sandbox_id,
                        ProvisioningStep::StartAgent,
                        "agent deferred (will start on first message)",
                    ))
                    .await;
            }
            Err(e) => {
                tracing::warn!(sandbox_id = %sandbox_id, error = %e, "agent health check failed (non-fatal)");
                let _ = tx
                    .send(self.step_event(
                        sandbox_id,
                        ProvisioningStep::StartAgent,
                        "agent deferred (will start on first message)",
                    ))
                    .await;
            }
        }
        tracing::info!(sandbox_id = %sandbox_id, "provisioning step: start_agent complete");

        // Send provisioning complete event
        let complete_event = StreamEvent {
            id: format!("prov-{}", Uuid::new_v4()),
            sandbox_id,
            session_id: None,
            event_type: StreamEventType::ProvisioningComplete,
            data: serde_json::json!({ "sandbox_id": sandbox_id.to_string() }),
            timestamp: Utc::now(),
        };
        let _ = tx.send(complete_event).await;

        // Fetch and return updated sandbox info
        let info = self.runtime.get_sandbox(&sandbox_id).await?;
        Ok(info)
    }

    fn validate_spec(&self, spec: &SandboxSpec, agent: &dyn AgentProvider) -> CiabResult<()> {
        if spec.agent_provider.is_empty() {
            return Err(CiabError::ConfigValidationError(
                "agent_provider must not be empty".to_string(),
            ));
        }

        // Validate agent config if present
        if let Some(ref config) = spec.agent_config {
            agent.validate_config(config)?;
        }

        // Check that required env vars are provided.
        // Sources (checked in order): spec.env_vars, credentials, host environment.
        let required = agent.required_env_vars();
        let available_env_keys: Vec<&str> = spec.env_vars.keys().map(|k| k.as_str()).collect();
        let has_credentials = !spec.credentials.is_empty();

        for var in &required {
            let in_spec = available_env_keys.contains(&var.as_str());
            let in_host = std::env::var(var).is_ok();
            if !in_spec && !has_credentials && !in_host {
                tracing::warn!(
                    env_var = %var,
                    "required env var not found in spec, credentials, or host — agent may fail at runtime"
                );
            }
        }

        Ok(())
    }

    fn step_event(&self, sandbox_id: Uuid, step: ProvisioningStep, detail: &str) -> StreamEvent {
        StreamEvent {
            id: format!("prov-{}", Uuid::new_v4()),
            sandbox_id,
            session_id: None,
            event_type: StreamEventType::ProvisioningStep,
            data: serde_json::json!({ "step": step.to_string(), "detail": detail }),
            timestamp: Utc::now(),
        }
    }
}
