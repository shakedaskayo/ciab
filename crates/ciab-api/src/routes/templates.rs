use std::collections::HashMap;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use chrono::Utc;
use ciab_core::error::CiabError;
use ciab_core::types::workspace::{
    Workspace, WorkspaceFilters, WorkspaceSpec, WorkspaceToml, TEMPLATE_KIND_LABEL,
    TEMPLATE_KIND_VALUE, TEMPLATE_SOURCE_FILE_LABEL, TEMPLATE_SOURCE_ID_LABEL,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::state::AppState;

// ---------------------------------------------------------------------------
// list_templates — list all workspaces with ciab/kind=template label
// ---------------------------------------------------------------------------

pub async fn list_templates(State(state): State<AppState>) -> Result<impl IntoResponse, CiabError> {
    let mut labels = HashMap::new();
    labels.insert(
        TEMPLATE_KIND_LABEL.to_string(),
        TEMPLATE_KIND_VALUE.to_string(),
    );

    let filters = WorkspaceFilters { name: None, labels };

    let templates = state.db.list_workspaces(&filters).await?;
    Ok(Json(templates))
}

// ---------------------------------------------------------------------------
// create_template — create a workspace marked as a template
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateTemplateRequest {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub spec: WorkspaceSpec,
    #[serde(default)]
    pub labels: HashMap<String, String>,
}

pub async fn create_template(
    State(state): State<AppState>,
    Json(body): Json<CreateTemplateRequest>,
) -> Result<impl IntoResponse, CiabError> {
    let now = Utc::now();
    let mut labels = body.labels;
    labels.insert(
        TEMPLATE_KIND_LABEL.to_string(),
        TEMPLATE_KIND_VALUE.to_string(),
    );

    let workspace = Workspace {
        id: Uuid::new_v4(),
        name: body.name,
        description: body.description,
        spec: body.spec,
        labels,
        created_at: now,
        updated_at: now,
    };

    state.db.insert_workspace(&workspace).await?;
    Ok((StatusCode::CREATED, Json(workspace)))
}

// ---------------------------------------------------------------------------
// create_from_template — clone a template into a new workspace
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateFromTemplateRequest {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub overrides: Option<WorkspaceSpec>,
}

pub async fn create_from_template(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<CreateFromTemplateRequest>,
) -> Result<impl IntoResponse, CiabError> {
    let template = state
        .db
        .get_workspace(&id)
        .await?
        .ok_or_else(|| CiabError::WorkspaceNotFound(id.to_string()))?;

    // Verify it's actually a template
    if template.labels.get(TEMPLATE_KIND_LABEL) != Some(&TEMPLATE_KIND_VALUE.to_string()) {
        return Err(CiabError::WorkspaceValidationError(
            "workspace is not a template".to_string(),
        ));
    }

    let spec = if let Some(overrides) = body.overrides {
        merge_spec(&template.spec, &overrides)
    } else {
        template.spec.clone()
    };

    let now = Utc::now();
    let mut labels = HashMap::new();
    labels.insert("ciab/from_template".to_string(), id.to_string());

    let workspace = Workspace {
        id: Uuid::new_v4(),
        name: body.name,
        description: body.description.or(template.description),
        spec,
        labels,
        created_at: now,
        updated_at: now,
    };

    state.db.insert_workspace(&workspace).await?;
    Ok((StatusCode::CREATED, Json(workspace)))
}

// ---------------------------------------------------------------------------
// Template Sources
// ---------------------------------------------------------------------------

pub async fn list_template_sources(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, CiabError> {
    let sources = state.db.list_template_sources().await?;
    Ok(Json(sources))
}

#[derive(Debug, Deserialize)]
pub struct AddTemplateSourceRequest {
    pub name: String,
    pub url: String,
    #[serde(default = "default_branch")]
    pub branch: String,
    #[serde(default = "default_templates_path")]
    pub templates_path: String,
}

fn default_branch() -> String {
    "main".to_string()
}

fn default_templates_path() -> String {
    ".ciab/templates".to_string()
}

pub async fn add_template_source(
    State(state): State<AppState>,
    Json(body): Json<AddTemplateSourceRequest>,
) -> Result<impl IntoResponse, CiabError> {
    let now = Utc::now();
    let source = ciab_core::types::workspace::TemplateSource {
        id: Uuid::new_v4(),
        name: body.name,
        url: body.url,
        branch: body.branch,
        templates_path: body.templates_path,
        last_synced_at: None,
        template_count: 0,
        created_at: now,
        updated_at: now,
    };

    state.db.insert_template_source(&source).await?;
    Ok((StatusCode::CREATED, Json(source)))
}

pub async fn delete_template_source(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, CiabError> {
    let _source = state
        .db
        .get_template_source(&id)
        .await?
        .ok_or_else(|| CiabError::TemplateSourceNotFound(id.to_string()))?;

    state.db.delete_template_source(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// sync_template_source — git clone and import TOML templates
// ---------------------------------------------------------------------------

pub async fn sync_template_source(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, CiabError> {
    let mut source = state
        .db
        .get_template_source(&id)
        .await?
        .ok_or_else(|| CiabError::TemplateSourceNotFound(id.to_string()))?;

    // Create temp directory for clone
    let tmp_dir = std::env::temp_dir().join(format!("ciab-template-sync-{}", Uuid::new_v4()));

    // Clone the repo (shallow)
    let clone_output = tokio::process::Command::new("git")
        .args([
            "clone",
            "--depth",
            "1",
            "--branch",
            &source.branch,
            "--single-branch",
            &source.url,
            tmp_dir.to_str().unwrap_or("/tmp/ciab-sync"),
        ])
        .output()
        .await
        .map_err(|e| CiabError::TemplateSyncFailed(format!("git clone failed: {}", e)))?;

    if !clone_output.status.success() {
        let stderr = String::from_utf8_lossy(&clone_output.stderr);
        let _ = tokio::fs::remove_dir_all(&tmp_dir).await;
        return Err(CiabError::TemplateSyncFailed(format!(
            "git clone failed: {}",
            stderr.trim()
        )));
    }

    // Find TOML files under templates_path
    let templates_dir = tmp_dir.join(&source.templates_path);
    let mut synced_count: u32 = 0;

    if templates_dir.is_dir() {
        let mut entries = tokio::fs::read_dir(&templates_dir)
            .await
            .map_err(|e| CiabError::TemplateSyncFailed(format!("read templates dir: {}", e)))?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| CiabError::TemplateSyncFailed(e.to_string()))?
        {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("toml") {
                continue;
            }

            let filename = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown.toml")
                .to_string();

            let content = tokio::fs::read_to_string(&path)
                .await
                .map_err(|e| CiabError::TemplateSyncFailed(format!("read {}: {}", filename, e)))?;

            let toml_def: WorkspaceToml = match toml::from_str(&content) {
                Ok(t) => t,
                Err(e) => {
                    tracing::warn!(file = %filename, error = %e, "skipping invalid template TOML");
                    continue;
                }
            };

            let spec = toml_def.workspace;
            let source_id_str = source.id.to_string();

            // Build labels for this template
            let mut labels = HashMap::new();
            labels.insert(
                TEMPLATE_KIND_LABEL.to_string(),
                TEMPLATE_KIND_VALUE.to_string(),
            );
            labels.insert(TEMPLATE_SOURCE_ID_LABEL.to_string(), source_id_str.clone());
            labels.insert(TEMPLATE_SOURCE_FILE_LABEL.to_string(), filename.clone());

            // Check if a template from this source with this filename already exists
            let existing_filters = WorkspaceFilters {
                name: None,
                labels: {
                    let mut l = HashMap::new();
                    l.insert(TEMPLATE_SOURCE_ID_LABEL.to_string(), source_id_str.clone());
                    l.insert(TEMPLATE_SOURCE_FILE_LABEL.to_string(), filename.clone());
                    l
                },
            };
            let existing = state.db.list_workspaces(&existing_filters).await?;

            if let Some(existing_ws) = existing.first() {
                // Update existing template
                let mut updated = existing_ws.clone();
                updated.spec = spec.clone();
                updated.labels = labels;
                updated.updated_at = Utc::now();
                if let Some(ref n) = spec.name {
                    updated.name = n.clone();
                }
                if spec.description.is_some() {
                    updated.description = spec.description;
                }
                state.db.update_workspace(&existing_ws.id, &updated).await?;
            } else {
                // Create new template
                let name = spec
                    .name
                    .clone()
                    .unwrap_or_else(|| filename.trim_end_matches(".toml").to_string());
                let description = spec.description.clone();
                let now = Utc::now();

                let workspace = Workspace {
                    id: Uuid::new_v4(),
                    name,
                    description,
                    spec,
                    labels,
                    created_at: now,
                    updated_at: now,
                };

                state.db.insert_workspace(&workspace).await?;
            }

            synced_count += 1;
        }
    }

    // Clean up temp directory
    let _ = tokio::fs::remove_dir_all(&tmp_dir).await;

    // Update source metadata
    source.last_synced_at = Some(Utc::now());
    source.template_count = synced_count;
    source.updated_at = Utc::now();
    state.db.update_template_source(&id, &source).await?;

    Ok(Json(serde_json::json!({
        "synced": synced_count,
        "source_id": id,
    })))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Merge overrides into a base spec. Non-empty override fields replace the base.
fn merge_spec(base: &WorkspaceSpec, overrides: &WorkspaceSpec) -> WorkspaceSpec {
    WorkspaceSpec {
        name: overrides.name.clone().or_else(|| base.name.clone()),
        description: overrides
            .description
            .clone()
            .or_else(|| base.description.clone()),
        repositories: if overrides.repositories.is_empty() {
            base.repositories.clone()
        } else {
            overrides.repositories.clone()
        },
        skills: if overrides.skills.is_empty() {
            base.skills.clone()
        } else {
            overrides.skills.clone()
        },
        pre_commands: if overrides.pre_commands.is_empty() {
            base.pre_commands.clone()
        } else {
            overrides.pre_commands.clone()
        },
        binaries: if overrides.binaries.is_empty() {
            base.binaries.clone()
        } else {
            overrides.binaries.clone()
        },
        filesystem: overrides.filesystem.clone(),
        agent: overrides.agent.clone().or_else(|| base.agent.clone()),
        subagents: if overrides.subagents.is_empty() {
            base.subagents.clone()
        } else {
            overrides.subagents.clone()
        },
        credentials: if overrides.credentials.is_empty() {
            base.credentials.clone()
        } else {
            overrides.credentials.clone()
        },
        env_vars: if overrides.env_vars.is_empty() {
            base.env_vars.clone()
        } else {
            overrides.env_vars.clone()
        },
        resource_limits: overrides
            .resource_limits
            .clone()
            .or_else(|| base.resource_limits.clone()),
        network: overrides.network.clone().or_else(|| base.network.clone()),
        volumes: if overrides.volumes.is_empty() {
            base.volumes.clone()
        } else {
            overrides.volumes.clone()
        },
        ports: if overrides.ports.is_empty() {
            base.ports.clone()
        } else {
            overrides.ports.clone()
        },
        labels: if overrides.labels.is_empty() {
            base.labels.clone()
        } else {
            overrides.labels.clone()
        },
        local_mounts: if overrides.local_mounts.is_empty() {
            base.local_mounts.clone()
        } else {
            overrides.local_mounts.clone()
        },
        env_file: overrides.env_file.clone().or_else(|| base.env_file.clone()),
        timeout_secs: overrides.timeout_secs.or(base.timeout_secs),
        image: overrides.image.clone().or_else(|| base.image.clone()),
        runtime: overrides.runtime.clone().or_else(|| base.runtime.clone()),
    }
}
