use std::collections::HashMap;
use std::convert::Infallible;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::sse::{Event, Sse};
use axum::response::IntoResponse;
use axum::Json;
use ciab_core::error::CiabError;
use ciab_core::types::sandbox::{
    LogOptions, SandboxFilters, SandboxInfo, SandboxSpec, SandboxState,
};
use ciab_core::types::stream::StreamEvent;
use futures::stream::Stream;
use serde::Deserialize;
use uuid::Uuid;

use crate::state::AppState;

// ---------------------------------------------------------------------------
// create_sandbox
// ---------------------------------------------------------------------------

pub async fn create_sandbox(
    State(state): State<AppState>,
    Json(spec): Json<SandboxSpec>,
) -> Result<impl IntoResponse, CiabError> {
    let provider_name = spec.agent_provider.clone();
    let agent = state
        .agents
        .get(&provider_name)
        .ok_or_else(|| CiabError::AgentProviderNotFound(provider_name.clone()))?
        .clone();

    // Create a channel for provisioning events and publish them through the
    // stream handler so SSE subscribers can follow along.
    let (tx, mut rx) = tokio::sync::mpsc::channel::<StreamEvent>(64);
    let stream_handler = state.stream_handler.clone();

    // Generate a sandbox ID up-front so the client can subscribe to SSE
    // events immediately and see provisioning progress.
    let sandbox_id = Uuid::new_v4();

    // Insert a placeholder record in the DB so the frontend can fetch it
    // right after creation and see it in state "creating".
    let now = chrono::Utc::now();
    let placeholder = SandboxInfo {
        id: sandbox_id,
        name: spec.name.clone(),
        state: SandboxState::Creating,
        persistence: spec.persistence.clone(),
        agent_provider: spec.agent_provider.clone(),
        endpoint_url: None,
        resource_stats: None,
        labels: spec.labels.clone(),
        created_at: now,
        updated_at: now,
        spec: spec.clone(),
    };
    state.db.insert_sandbox(&placeholder).await?;

    // Forward provisioning events to the stream handler, rewriting the
    // sandbox_id so SSE subscribers (using the ID we returned) see them.
    let fwd_sandbox_id = sandbox_id;
    tokio::spawn(async move {
        while let Some(mut event) = rx.recv().await {
            event.sandbox_id = fwd_sandbox_id;
            let _ = stream_handler.publish(event).await;
        }
    });

    // Kick off provisioning in the background.
    let provisioning = state.provisioning.clone();
    let db = state.db.clone();
    let spec_clone = spec.clone();
    tokio::spawn(async move {
        match provisioning
            .provision_with_id(&spec_clone, agent.as_ref(), tx, Some(sandbox_id))
            .await
        {
            Ok(info) => {
                // Update the placeholder record with the real sandbox info.
                // The runtime may have produced a different ID, so delete the
                // placeholder and insert the real one.
                if info.id != sandbox_id {
                    let _ = db.delete_sandbox(&sandbox_id).await;
                    if let Err(e) = db.insert_sandbox(&info).await {
                        tracing::error!(error = %e, "failed to persist sandbox after provisioning");
                    }
                } else {
                    // Same ID — just update the state to running.
                    let _ = db
                        .update_sandbox_state(&sandbox_id, &SandboxState::Running)
                        .await;
                }
            }
            Err(e) => {
                tracing::error!(error = %e, "provisioning failed");
                let _ = db
                    .update_sandbox_state(&sandbox_id, &SandboxState::Failed)
                    .await;
            }
        }
    });

    Ok((
        StatusCode::ACCEPTED,
        Json(serde_json::json!({
            "sandbox_id": sandbox_id,
            "status": "provisioning",
        })),
    ))
}

// ---------------------------------------------------------------------------
// list_sandboxes
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Default)]
pub struct ListSandboxesQuery {
    pub state: Option<SandboxState>,
    pub provider: Option<String>,
    #[serde(default)]
    pub labels: Option<String>,
}

pub async fn list_sandboxes(
    State(state): State<AppState>,
    Query(params): Query<ListSandboxesQuery>,
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

    let filters = SandboxFilters {
        state: params.state,
        provider: params.provider,
        labels: label_map,
    };

    let sandboxes = state.db.list_sandboxes(&filters).await?;
    Ok(Json(sandboxes))
}

// ---------------------------------------------------------------------------
// get_sandbox
// ---------------------------------------------------------------------------

pub async fn get_sandbox(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, CiabError> {
    let mut info = state
        .db
        .get_sandbox(&id)
        .await?
        .ok_or_else(|| CiabError::SandboxNotFound(id.to_string()))?;

    // For "active" sandboxes, validate state against the runtime and update
    // the DB if it has drifted (e.g. container was deleted externally).
    if matches!(
        info.state,
        SandboxState::Running
            | SandboxState::Creating
            | SandboxState::Pending
            | SandboxState::Paused
    ) {
        match state.runtime.get_sandbox(&id).await {
            Ok(runtime_info) => {
                if runtime_info.state != info.state {
                    let _ = state
                        .db
                        .update_sandbox_state(&id, &runtime_info.state)
                        .await;
                    info.state = runtime_info.state;
                }
            }
            Err(_) => {
                // Runtime doesn't know about this sandbox — mark terminated.
                let _ = state
                    .db
                    .update_sandbox_state(&id, &SandboxState::Terminated)
                    .await;
                info.state = SandboxState::Terminated;
            }
        }
    }

    Ok(Json(info))
}

// ---------------------------------------------------------------------------
// delete_sandbox
// ---------------------------------------------------------------------------

pub async fn delete_sandbox(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, CiabError> {
    state.runtime.terminate_sandbox(&id).await?;
    state.db.delete_sandbox(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// start / stop / pause / resume
// ---------------------------------------------------------------------------

pub async fn start_sandbox(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, CiabError> {
    state.runtime.start_sandbox(&id).await?;
    state
        .db
        .update_sandbox_state(&id, &SandboxState::Running)
        .await?;
    Ok(Json(serde_json::json!({"status": "running"})))
}

pub async fn stop_sandbox(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, CiabError> {
    state.runtime.stop_sandbox(&id).await?;
    state
        .db
        .update_sandbox_state(&id, &SandboxState::Stopped)
        .await?;
    Ok(Json(serde_json::json!({"status": "stopped"})))
}

pub async fn pause_sandbox(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, CiabError> {
    state.runtime.pause_sandbox(&id).await?;
    state
        .db
        .update_sandbox_state(&id, &SandboxState::Paused)
        .await?;
    Ok(Json(serde_json::json!({"status": "paused"})))
}

pub async fn resume_sandbox(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, CiabError> {
    state.runtime.resume_sandbox(&id).await?;
    state
        .db
        .update_sandbox_state(&id, &SandboxState::Running)
        .await?;
    Ok(Json(serde_json::json!({"status": "running"})))
}

// ---------------------------------------------------------------------------
// sandbox_stats
// ---------------------------------------------------------------------------

pub async fn sandbox_stats(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, CiabError> {
    let stats = state.runtime.get_stats(&id).await?;
    Ok(Json(stats))
}

// ---------------------------------------------------------------------------
// sandbox_logs
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Default)]
pub struct LogsQuery {
    #[serde(default)]
    pub follow: Option<bool>,
    #[serde(default)]
    pub tail: Option<u32>,
}

pub async fn sandbox_logs(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(params): Query<LogsQuery>,
) -> Result<impl IntoResponse, CiabError> {
    let options = LogOptions {
        follow: params.follow.unwrap_or(false),
        tail: params.tail,
        since: None,
    };

    if options.follow {
        // Streaming logs via SSE
        let mut rx = state.runtime.stream_logs(&id, &options).await?;
        let stream = async_stream::stream! {
            while let Some(line) = rx.recv().await {
                let event = Event::default().data(line);
                yield Ok::<_, Infallible>(event);
            }
        };
        Ok(Sse::new(stream).into_response())
    } else {
        let mut rx = state.runtime.stream_logs(&id, &options).await?;
        let mut lines = Vec::new();
        while let Some(line) = rx.recv().await {
            lines.push(line);
        }
        Ok(Json(serde_json::json!({"logs": lines})).into_response())
    }
}

// ---------------------------------------------------------------------------
// sandbox_stream — SSE stream of all sandbox events
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Default)]
pub struct StreamQuery {
    #[serde(default)]
    pub last_event_id: Option<String>,
}

pub async fn sandbox_stream(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(params): Query<StreamQuery>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, CiabError> {
    // Use subscribe_with_replay to get buffered events + live receiver.
    // This lets clients reconnect without missing events.
    let (replay_events, mut rx) = state
        .stream_handler
        .subscribe_with_replay(&id, params.last_event_id.as_deref())
        .await?;
    let keepalive_secs = state.config.streaming.keepalive_interval_secs;

    let stream = async_stream::stream! {
        // 1. Replay buffered events so reconnecting clients catch up
        for event in &replay_events {
            let data = serde_json::to_string(event).unwrap_or_default();
            let event_type = serde_json::to_string(&event.event_type)
                .unwrap_or_default()
                .trim_matches('"')
                .to_string();
            yield Ok::<_, Infallible>(
                Event::default()
                    .id(event.id.clone())
                    .event(event_type)
                    .data(data),
            );
        }

        // 2. Stream live events with keepalive
        let mut keepalive = tokio::time::interval(
            std::time::Duration::from_secs(keepalive_secs),
        );
        // consume the first immediate tick
        keepalive.tick().await;

        loop {
            tokio::select! {
                result = rx.recv() => {
                    match result {
                        Ok(event) => {
                            let data = serde_json::to_string(&event).unwrap_or_default();
                            let event_type = serde_json::to_string(&event.event_type)
                                .unwrap_or_default()
                                .trim_matches('"')
                                .to_string();
                            yield Ok::<_, Infallible>(
                                Event::default()
                                    .id(event.id)
                                    .event(event_type)
                                    .data(data),
                            );
                        }
                        Err(_) => continue,
                    }
                }
                _ = keepalive.tick() => {
                    yield Ok(Event::default().comment("keepalive"));
                }
            }
        }
    };

    Ok(Sse::new(stream))
}
