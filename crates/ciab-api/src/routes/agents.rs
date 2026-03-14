use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::Json;
use ciab_core::error::CiabError;

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
