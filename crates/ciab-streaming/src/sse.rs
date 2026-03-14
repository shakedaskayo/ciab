use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use axum::response::sse::{Event, Sse};
use futures::stream::Stream;
use uuid::Uuid;

use ciab_core::types::stream::StreamEvent;

use crate::broker::StreamBroker;

/// Format a `StreamEvent` as an SSE text block.
pub fn format_sse_event(event: &StreamEvent) -> String {
    let event_type =
        serde_json::to_string(&event.event_type).unwrap_or_else(|_| "\"unknown\"".to_string());
    let event_type = event_type.trim_matches('"');
    let data = serde_json::to_string(&event).unwrap_or_default();
    format!(
        "id: {}\nevent: {}\ndata: {}\n\n",
        event.id, event_type, data
    )
}

/// Format a keepalive SSE comment.
pub fn format_keepalive() -> String {
    ": keepalive\n\n".to_string()
}

fn event_matches_session(event: &StreamEvent, session_id: Option<Uuid>) -> bool {
    match session_id {
        Some(sid) => match event.session_id {
            Some(event_sid) => event_sid == sid,
            None => true, // events without a session_id are broadcast to all
        },
        None => true,
    }
}

fn stream_event_to_sse(event: &StreamEvent) -> Event {
    Event::default()
        .id(event.id.clone())
        .event(
            serde_json::to_string(&event.event_type)
                .unwrap_or_default()
                .trim_matches('"')
                .to_string(),
        )
        .data(serde_json::to_string(event).unwrap_or_default())
}

/// Build an SSE response that replays missed events, then streams live events
/// with periodic keepalive comments.
pub fn sse_stream(
    broker: Arc<StreamBroker>,
    sandbox_id: Uuid,
    session_id: Option<Uuid>,
    last_event_id: Option<String>,
    keepalive_interval_secs: u64,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = async_stream::stream! {
        // Subscribe first so we don't miss events between replay and live streaming
        let (replay_events, receiver) = match broker
            .subscribe_with_options(&sandbox_id, session_id, last_event_id.as_deref())
            .await
        {
            Ok(r) => r,
            Err(_) => return,
        };

        // 1. Replay buffered events
        for event in &replay_events {
            if event_matches_session(event, session_id) {
                yield Ok(stream_event_to_sse(event));
            }
        }

        // 2. Stream live events with keepalive
        let mut rx_stream = tokio_stream::wrappers::BroadcastStream::new(receiver);
        let mut keepalive = tokio::time::interval(Duration::from_secs(keepalive_interval_secs));
        // Consume the first immediate tick
        keepalive.tick().await;

        loop {
            tokio::select! {
                item = futures::StreamExt::next(&mut rx_stream) => {
                    match item {
                        Some(Ok(event)) => {
                            if event_matches_session(&event, session_id) {
                                yield Ok(stream_event_to_sse(&event));
                            }
                        }
                        Some(Err(_)) => continue, // lagged
                        None => break,
                    }
                }
                _ = keepalive.tick() => {
                    yield Ok(Event::default().comment("keepalive"));
                }
            }
        }
    };

    Sse::new(stream)
}
