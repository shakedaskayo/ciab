use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::Json;
use ciab_core::error::CiabError;
use serde::Serialize;
use std::process::Command;
use tracing::{debug, warn};

use crate::state::AppState;

/// GET /api/v1/agents — list available agent providers.
pub async fn list_providers(State(state): State<AppState>) -> Result<impl IntoResponse, CiabError> {
    let providers: Vec<String> = state.agents.keys().cloned().collect();
    Ok(Json(providers))
}

/// GET /api/v1/agents/{provider}/commands — list slash commands for a provider.
pub async fn get_slash_commands(
    State(state): State<AppState>,
    Path(provider): Path<String>,
) -> Result<impl IntoResponse, CiabError> {
    let agent = state
        .agents
        .get(&provider)
        .ok_or(CiabError::AgentProviderNotFound(provider))?;
    Ok(Json(agent.slash_commands()))
}

/// Status of a single agent provider's binary on the host.
#[derive(Debug, Serialize)]
pub struct AgentProviderStatus {
    pub name: String,
    pub installed: bool,
    pub binary: String,
    pub binary_path: Option<String>,
    pub version: Option<String>,
    pub install_command: Option<String>,
    pub required_env_vars: Vec<String>,
}

/// GET /api/v1/agents/status — detect installed agent binaries.
pub async fn provider_status(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, CiabError> {
    let mut statuses = Vec::new();

    for (name, agent) in &state.agents {
        let binary = agent_binary_name(name);
        let (installed, binary_path, version) = detect_binary(&binary);

        let install_cmd = agent.install_commands().first().cloned();

        statuses.push(AgentProviderStatus {
            name: name.clone(),
            installed,
            binary: binary.clone(),
            binary_path,
            version,
            install_command: install_cmd,
            required_env_vars: agent.required_env_vars(),
        });
    }

    // Sort by name for stable ordering.
    statuses.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(Json(statuses))
}

/// Response for install attempt.
#[derive(Debug, Serialize)]
pub struct AgentInstallResult {
    pub provider: String,
    pub success: bool,
    pub message: String,
    pub version: Option<String>,
}

/// POST /api/v1/agents/{provider}/install — install an agent binary.
pub async fn install_provider(
    State(state): State<AppState>,
    Path(provider): Path<String>,
) -> Result<impl IntoResponse, CiabError> {
    let agent = state
        .agents
        .get(&provider)
        .ok_or_else(|| CiabError::AgentProviderNotFound(provider.clone()))?;

    let commands = agent.install_commands();
    if commands.is_empty() {
        return Ok(Json(AgentInstallResult {
            provider,
            success: false,
            message: "No install command available for this provider".to_string(),
            version: None,
        }));
    }

    let install_cmd = &commands[0];
    debug!(provider = %provider, cmd = %install_cmd, "Installing agent provider");

    // Run the install command via sh -c.
    let output = tokio::task::spawn_blocking({
        let cmd = install_cmd.clone();
        move || {
            Command::new("sh")
                .arg("-c")
                .arg(&cmd)
                .output()
        }
    })
    .await
    .map_err(|e| CiabError::Internal(format!("Task join error: {e}")))?
    .map_err(|e| CiabError::Internal(format!("Failed to run install command: {e}")))?;

    if output.status.success() {
        let binary = agent_binary_name(&provider);
        let (_, _, version) = detect_binary(&binary);
        Ok(Json(AgentInstallResult {
            provider,
            success: true,
            message: "Installation completed successfully".to_string(),
            version,
        }))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let msg = if stderr.is_empty() { stdout } else { stderr };
        warn!(provider = %provider, error = %msg, "Agent install failed");
        Ok(Json(AgentInstallResult {
            provider,
            success: false,
            message: format!("Installation failed: {}", msg.trim()),
            version: None,
        }))
    }
}

/// Map provider name to its CLI binary name.
fn agent_binary_name(provider: &str) -> String {
    match provider {
        "claude-code" => "claude".to_string(),
        "codex" => "codex".to_string(),
        "gemini" => "gemini".to_string(),
        "cursor" => "cursor".to_string(),
        other => other.to_string(),
    }
}

/// Try to detect a binary on the system PATH.
/// Returns (installed, full_path, version).
fn detect_binary(binary: &str) -> (bool, Option<String>, Option<String>) {
    // Try `which` to find the binary path.
    let path = Command::new("which")
        .arg(binary)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());

    if path.is_none() {
        return (false, None, None);
    }

    // Try `binary --version` to get version info.
    let version = Command::new(binary)
        .arg("--version")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| {
            let raw = String::from_utf8_lossy(&o.stdout).trim().to_string();
            // Take first line, strip common prefixes.
            raw.lines()
                .next()
                .unwrap_or(&raw)
                .trim()
                .to_string()
        });

    (true, path, version)
}
