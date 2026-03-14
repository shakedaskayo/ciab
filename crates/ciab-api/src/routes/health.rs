use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use ciab_core::error::CiabError;

use crate::state::AppState;

pub async fn health() -> Result<impl IntoResponse, CiabError> {
    Ok(Json(serde_json::json!({"status": "healthy"})))
}

pub async fn ready(State(state): State<AppState>) -> Result<impl IntoResponse, CiabError> {
    // Verify database connectivity by accessing the pool.
    let _pool = state.db.pool();
    Ok(Json(serde_json::json!({"status": "ready"})))
}
