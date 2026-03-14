use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use ciab_core::error::CiabError;
use serde::Deserialize;
use uuid::Uuid;

use crate::state::AppState;

// ---------------------------------------------------------------------------
// list_files
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Default)]
pub struct ListFilesQuery {
    #[serde(default = "default_path")]
    pub path: String,
}

fn default_path() -> String {
    "/".to_string()
}

pub async fn list_files(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(params): Query<ListFilesQuery>,
) -> Result<impl IntoResponse, CiabError> {
    let files = state.runtime.list_files(&id, &params.path).await?;
    Ok(Json(files))
}

// ---------------------------------------------------------------------------
// download_file
// ---------------------------------------------------------------------------

pub async fn download_file(
    State(state): State<AppState>,
    Path((id, path)): Path<(Uuid, String)>,
) -> Result<impl IntoResponse, CiabError> {
    let file_path = format!("/{}", path);
    let content = state.runtime.read_file(&id, &file_path).await?;
    Ok((
        [(axum::http::header::CONTENT_TYPE, "application/octet-stream")],
        content,
    ))
}

// ---------------------------------------------------------------------------
// upload_file
// ---------------------------------------------------------------------------

pub async fn upload_file(
    State(state): State<AppState>,
    Path((id, path)): Path<(Uuid, String)>,
    body: Bytes,
) -> Result<impl IntoResponse, CiabError> {
    let file_path = format!("/{}", path);
    state.runtime.write_file(&id, &file_path, &body).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// delete_file
// ---------------------------------------------------------------------------

pub async fn delete_file(
    State(state): State<AppState>,
    Path((id, path)): Path<(Uuid, String)>,
) -> Result<impl IntoResponse, CiabError> {
    let file_path = format!("/{}", path);
    // Use exec to remove the file.
    let exec_req = ciab_core::types::sandbox::ExecRequest {
        command: vec!["rm".to_string(), "-rf".to_string(), file_path],
        workdir: None,
        env: Default::default(),
        stdin: None,
        timeout_secs: Some(30),
        tty: false,
    };
    let result = state.runtime.exec(&id, &exec_req).await?;
    if result.exit_code != 0 {
        return Err(CiabError::ExecFailed(format!(
            "rm failed (exit {}): {}",
            result.exit_code, result.stderr
        )));
    }
    Ok(StatusCode::NO_CONTENT)
}
