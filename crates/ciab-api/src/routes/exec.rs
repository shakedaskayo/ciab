use std::convert::Infallible;

use axum::extract::{Path, State};
use axum::response::sse::{Event, Sse};
use axum::response::IntoResponse;
use axum::Json;
use ciab_core::error::CiabError;
use ciab_core::types::sandbox::ExecRequest;
use futures::stream::Stream;
use uuid::Uuid;

use crate::state::AppState;

// ---------------------------------------------------------------------------
// exec_command
// ---------------------------------------------------------------------------

pub async fn exec_command(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<ExecRequest>,
) -> Result<impl IntoResponse, CiabError> {
    let result = state.runtime.exec(&id, &req).await?;
    Ok(Json(result))
}

// ---------------------------------------------------------------------------
// exec_stream — SSE stream of exec output
// ---------------------------------------------------------------------------

pub async fn exec_stream(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<ExecRequest>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, CiabError> {
    // Execute the command and stream stdout/stderr line by line.
    let result = state.runtime.exec(&id, &req).await?;

    let stream = async_stream::stream! {
        // Emit stdout lines
        for line in result.stdout.lines() {
            yield Ok::<_, Infallible>(
                Event::default()
                    .event("stdout")
                    .data(line),
            );
        }
        // Emit stderr lines
        for line in result.stderr.lines() {
            yield Ok::<_, Infallible>(
                Event::default()
                    .event("stderr")
                    .data(line),
            );
        }
        // Emit exit code
        yield Ok(
            Event::default()
                .event("exit")
                .data(serde_json::json!({
                    "exit_code": result.exit_code,
                    "duration_ms": result.duration_ms,
                }).to_string()),
        );
    };

    Ok(Sse::new(stream))
}
