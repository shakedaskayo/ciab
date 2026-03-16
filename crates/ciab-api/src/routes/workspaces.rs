use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use chrono::Utc;
use ciab_core::error::CiabError;
use ciab_core::traits::runtime::SandboxRuntime;
use ciab_core::types::workspace::{RuntimeBackend, Workspace, WorkspaceFilters, WorkspaceSpec};
use serde::Deserialize;
use uuid::Uuid;

use crate::state::AppState;

// ---------------------------------------------------------------------------
// create_workspace
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateWorkspaceRequest {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub spec: WorkspaceSpec,
    #[serde(default)]
    pub labels: HashMap<String, String>,
}

pub async fn create_workspace(
    State(state): State<AppState>,
    Json(body): Json<CreateWorkspaceRequest>,
) -> Result<impl IntoResponse, CiabError> {
    let now = Utc::now();
    let workspace = Workspace {
        id: Uuid::new_v4(),
        name: body.name,
        description: body.description,
        spec: body.spec,
        labels: body.labels,
        created_at: now,
        updated_at: now,
    };

    state.db.insert_workspace(&workspace).await?;
    Ok((StatusCode::CREATED, Json(workspace)))
}

// ---------------------------------------------------------------------------
// list_workspaces
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Default)]
pub struct ListWorkspacesQuery {
    pub name: Option<String>,
    #[serde(default)]
    pub labels: Option<String>,
}

pub async fn list_workspaces(
    State(state): State<AppState>,
    Query(params): Query<ListWorkspacesQuery>,
) -> Result<impl IntoResponse, CiabError> {
    let label_map: HashMap<String, String> = params
        .labels
        .as_deref()
        .unwrap_or("")
        .split(',')
        .filter(|s| !s.is_empty())
        .filter_map(|kv| {
            let mut parts = kv.splitn(2, '=');
            Some((parts.next()?.to_string(), parts.next()?.to_string()))
        })
        .collect();

    let filters = WorkspaceFilters {
        name: params.name,
        labels: label_map,
    };

    let workspaces = state.db.list_workspaces(&filters).await?;
    Ok(Json(workspaces))
}

// ---------------------------------------------------------------------------
// get_workspace
// ---------------------------------------------------------------------------

pub async fn get_workspace(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, CiabError> {
    let workspace = state
        .db
        .get_workspace(&id)
        .await?
        .ok_or_else(|| CiabError::WorkspaceNotFound(id.to_string()))?;
    Ok(Json(workspace))
}

// ---------------------------------------------------------------------------
// update_workspace
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct UpdateWorkspaceRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub spec: Option<WorkspaceSpec>,
    #[serde(default)]
    pub labels: Option<HashMap<String, String>>,
}

pub async fn update_workspace(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateWorkspaceRequest>,
) -> Result<impl IntoResponse, CiabError> {
    let mut workspace = state
        .db
        .get_workspace(&id)
        .await?
        .ok_or_else(|| CiabError::WorkspaceNotFound(id.to_string()))?;

    if let Some(name) = body.name {
        workspace.name = name;
    }
    if let Some(description) = body.description {
        workspace.description = Some(description);
    }
    if let Some(spec) = body.spec {
        workspace.spec = spec;
    }
    if let Some(labels) = body.labels {
        workspace.labels = labels;
    }
    workspace.updated_at = Utc::now();

    state.db.update_workspace(&id, &workspace).await?;
    Ok(Json(workspace))
}

// ---------------------------------------------------------------------------
// delete_workspace
// ---------------------------------------------------------------------------

pub async fn delete_workspace(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, CiabError> {
    state.db.delete_workspace(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// launch_workspace — create a sandbox from a workspace spec
// ---------------------------------------------------------------------------

pub async fn launch_workspace(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, CiabError> {
    let workspace = state
        .db
        .get_workspace(&id)
        .await?
        .ok_or_else(|| CiabError::WorkspaceNotFound(id.to_string()))?;

    // Convert workspace spec into a SandboxSpec
    let spec = workspace_to_sandbox_spec(&workspace.spec)?;

    let provider_name = spec.agent_provider.clone();
    let agent = state
        .agents
        .get(&provider_name)
        .ok_or_else(|| CiabError::AgentProviderNotFound(provider_name.clone()))?
        .clone();

    // Resolve runtime from workspace spec (or fall back to server default)
    let resolved_runtime = resolve_runtime(&state, &workspace.spec);

    let (tx, mut rx) = tokio::sync::mpsc::channel::<ciab_core::types::stream::StreamEvent>(64);
    let stream_handler = state.stream_handler.clone();

    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            let _ = stream_handler.publish(event).await;
        }
    });

    let sandbox_id = Uuid::new_v4();
    let workspace_id = workspace.id;

    // Create a per-launch provisioning pipeline with the resolved runtime
    let provisioning = Arc::new(ciab_provisioning::ProvisioningPipeline::new(
        resolved_runtime,
        state.credentials.clone(),
        state.config.provisioning.timeout_secs,
    ));
    let db = state.db.clone();
    tokio::spawn(async move {
        match provisioning.provision(&spec, agent.as_ref(), tx).await {
            Ok(info) => {
                let sid = info.id;
                if let Err(e) = db.insert_sandbox(&info).await {
                    tracing::error!(error = %e, "failed to persist sandbox after provisioning");
                }
                if let Err(e) = db.link_sandbox_to_workspace(&workspace_id, &sid).await {
                    tracing::error!(error = %e, "failed to link sandbox to workspace");
                }
            }
            Err(e) => {
                tracing::error!(error = %e, "workspace launch failed");
            }
        }
    });

    Ok((
        StatusCode::ACCEPTED,
        Json(serde_json::json!({
            "sandbox_id": sandbox_id,
            "workspace_id": workspace.id,
            "status": "provisioning",
        })),
    ))
}

// ---------------------------------------------------------------------------
// list_workspace_sandboxes
// ---------------------------------------------------------------------------

pub async fn list_workspace_sandboxes(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, CiabError> {
    let _workspace = state
        .db
        .get_workspace(&id)
        .await?
        .ok_or_else(|| CiabError::WorkspaceNotFound(id.to_string()))?;

    let sandbox_ids = state.db.list_workspace_sandboxes(&id).await?;
    Ok(Json(serde_json::json!({"sandbox_ids": sandbox_ids})))
}

// ---------------------------------------------------------------------------
// import_workspace_toml — create workspace from TOML body
// ---------------------------------------------------------------------------

pub async fn import_workspace_toml(
    State(state): State<AppState>,
    body: String,
) -> Result<impl IntoResponse, CiabError> {
    let toml_def: ciab_core::types::workspace::WorkspaceToml =
        toml::from_str(&body).map_err(|e| CiabError::WorkspaceValidationError(e.to_string()))?;

    let spec = toml_def.workspace;
    let now = Utc::now();
    let workspace = Workspace {
        id: Uuid::new_v4(),
        name: spec
            .name
            .clone()
            .unwrap_or_else(|| format!("workspace-{}", &Uuid::new_v4().to_string()[..8])),
        description: spec.description.clone(),
        spec,
        labels: HashMap::new(),
        created_at: now,
        updated_at: now,
    };

    state.db.insert_workspace(&workspace).await?;
    Ok((StatusCode::CREATED, Json(workspace)))
}

// ---------------------------------------------------------------------------
// export_workspace_toml — export workspace as TOML
// ---------------------------------------------------------------------------

pub async fn export_workspace_toml(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, CiabError> {
    let workspace = state
        .db
        .get_workspace(&id)
        .await?
        .ok_or_else(|| CiabError::WorkspaceNotFound(id.to_string()))?;

    let toml_def = ciab_core::types::workspace::WorkspaceToml {
        workspace: workspace.spec,
    };

    let toml_str =
        toml::to_string_pretty(&toml_def).map_err(|e| CiabError::Internal(e.to_string()))?;

    Ok((
        StatusCode::OK,
        [("content-type", "application/toml")],
        toml_str,
    ))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Resolve the runtime to use for a workspace, falling back to the server default.
fn resolve_runtime(state: &AppState, spec: &WorkspaceSpec) -> Arc<dyn SandboxRuntime> {
    if let Some(ref rt_config) = spec.runtime {
        let backend_key = match rt_config.backend {
            RuntimeBackend::Local => "local",
            RuntimeBackend::OpenSandbox => "opensandbox",
            RuntimeBackend::Docker => "docker",
            RuntimeBackend::Kubernetes => "kubernetes",
            RuntimeBackend::Default => return state.runtime.clone(),
        };
        if let Some(rt) = state.runtimes.get(backend_key) {
            return rt.clone();
        }
        tracing::warn!(
            backend = backend_key,
            "requested runtime backend not available, falling back to default"
        );
    }
    state.runtime.clone()
}

fn workspace_to_sandbox_spec(
    ws: &WorkspaceSpec,
) -> Result<ciab_core::types::sandbox::SandboxSpec, CiabError> {
    let agent_config = ws.agent.as_ref().map(|a| a.into());
    let provider = ws
        .agent
        .as_ref()
        .map(|a| a.provider.clone())
        .ok_or_else(|| {
            CiabError::WorkspaceValidationError(
                "workspace must have an agent provider configured".to_string(),
            )
        })?;

    // Merge pre-commands and skill install commands into provisioning_scripts
    let mut scripts = Vec::new();

    // Install skills
    for skill in &ws.skills {
        if skill.enabled {
            scripts.push(format!("npx skillsadd {}", skill.source));
        }
    }

    // Install binaries
    for binary in &ws.binaries {
        if let Some(ref cmd) = binary.install_command {
            scripts.push(cmd.clone());
        } else {
            let install_cmd = match &binary.method {
                ciab_core::types::workspace::BinaryInstallMethod::Apt => {
                    if let Some(ref v) = binary.version {
                        format!("apt-get install -y {}={}", binary.name, v)
                    } else {
                        format!("apt-get install -y {}", binary.name)
                    }
                }
                ciab_core::types::workspace::BinaryInstallMethod::Cargo => {
                    if let Some(ref v) = binary.version {
                        format!("cargo install {} --version {}", binary.name, v)
                    } else {
                        format!("cargo install {}", binary.name)
                    }
                }
                ciab_core::types::workspace::BinaryInstallMethod::Npm => {
                    if let Some(ref v) = binary.version {
                        format!("npm install -g {}@{}", binary.name, v)
                    } else {
                        format!("npm install -g {}", binary.name)
                    }
                }
                ciab_core::types::workspace::BinaryInstallMethod::Pip => {
                    if let Some(ref v) = binary.version {
                        format!("pip install {}=={}", binary.name, v)
                    } else {
                        format!("pip install {}", binary.name)
                    }
                }
                ciab_core::types::workspace::BinaryInstallMethod::Url { url } => {
                    format!(
                        "curl -fsSL {} -o /usr/local/bin/{} && chmod +x /usr/local/bin/{}",
                        url, binary.name, binary.name
                    )
                }
                ciab_core::types::workspace::BinaryInstallMethod::Custom => {
                    continue;
                }
            };
            scripts.push(install_cmd);
        }
    }

    // Pre-commands
    for cmd in &ws.pre_commands {
        let mut full_cmd = cmd.command.clone();
        if !cmd.args.is_empty() {
            full_cmd.push(' ');
            full_cmd.push_str(&cmd.args.join(" "));
        }
        scripts.push(full_cmd);
    }

    // Convert repos
    let git_repos: Vec<ciab_core::types::sandbox::GitRepoSpec> =
        ws.repositories.iter().map(|r| r.into()).collect();

    // Collect credential IDs
    let credential_ids: Vec<String> = ws
        .credentials
        .iter()
        .filter_map(|c| c.id.clone().or_else(|| c.name.clone()))
        .collect();

    // Convert local mounts
    let local_mounts: Vec<ciab_core::types::sandbox::LocalMountSpec> = ws
        .local_mounts
        .iter()
        .map(|m| {
            let dir_name = std::path::Path::new(&m.source)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "mount".to_string());
            ciab_core::types::sandbox::LocalMountSpec {
                source: m.source.clone(),
                dest_path: m
                    .dest_path
                    .clone()
                    .unwrap_or_else(|| format!("/workspace/{}", dir_name)),
                sync_mode: match m.sync_mode {
                    ciab_core::types::workspace::SyncMode::Copy => "copy".to_string(),
                    ciab_core::types::workspace::SyncMode::Link => "link".to_string(),
                    ciab_core::types::workspace::SyncMode::Bind => "bind".to_string(),
                },
                exclude_patterns: m.exclude_patterns.clone(),
                writeback: m.writeback,
            }
        })
        .collect();

    // Parse env_file if present and merge into env_vars
    let mut env_vars = ws.env_vars.clone();
    if let Some(ref env_file) = ws.env_file {
        match std::fs::read_to_string(env_file) {
            Ok(content) => {
                for line in content.lines() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }
                    if let Some((key, value)) = line.split_once('=') {
                        let key = key.trim().to_string();
                        let value = value
                            .trim()
                            .trim_matches('"')
                            .trim_matches('\'')
                            .to_string();
                        // .env values don't override explicit env_vars
                        env_vars.entry(key).or_insert(value);
                    }
                }
            }
            Err(e) => {
                tracing::warn!(env_file = %env_file, error = %e, "failed to read .env file, skipping");
            }
        }
    }

    // Add bind-mode local mounts as volume mounts
    let mut volumes = ws.volumes.clone();
    for mount in &local_mounts {
        if mount.sync_mode == "bind" {
            volumes.push(ciab_core::types::sandbox::VolumeMount {
                source: mount.source.clone(),
                dest: mount.dest_path.clone(),
                read_only: false,
            });
        }
    }

    // Determine runtime_backend string from workspace config
    let runtime_backend = ws.runtime.as_ref().and_then(|rt| match rt.backend {
        ciab_core::types::workspace::RuntimeBackend::Default => None,
        ciab_core::types::workspace::RuntimeBackend::Local => Some("local".to_string()),
        ciab_core::types::workspace::RuntimeBackend::OpenSandbox => Some("opensandbox".to_string()),
        ciab_core::types::workspace::RuntimeBackend::Docker => Some("docker".to_string()),
        ciab_core::types::workspace::RuntimeBackend::Kubernetes => Some("kubernetes".to_string()),
    });

    // Pass AgentFS config through labels so the pipeline can read it
    let mut labels = ws.labels.clone();
    if let Some(ref fs_config) = ws.filesystem.agentfs {
        if fs_config.enabled {
            labels.insert("ciab/agentfs_enabled".to_string(), "true".to_string());
            labels.insert("ciab/agentfs_binary".to_string(), fs_config.binary.clone());
            if let Some(ref db_path) = fs_config.db_path {
                labels.insert("ciab/agentfs_db_path".to_string(), db_path.clone());
            }
            labels.insert(
                "ciab/agentfs_logging".to_string(),
                fs_config.operation_logging.to_string(),
            );
        }
    }

    Ok(ciab_core::types::sandbox::SandboxSpec {
        name: ws.name.clone(),
        agent_provider: provider,
        image: ws.image.clone(),
        resource_limits: ws.resource_limits.clone(),
        persistence: ciab_core::types::sandbox::SandboxPersistence::Ephemeral,
        network: ws.network.clone(),
        env_vars,
        volumes,
        ports: ws.ports.clone(),
        git_repos,
        credentials: credential_ids,
        provisioning_scripts: scripts,
        labels,
        timeout_secs: ws.timeout_secs,
        agent_config,
        local_mounts,
        runtime_backend,
    })
}
