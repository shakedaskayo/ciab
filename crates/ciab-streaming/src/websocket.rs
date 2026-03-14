use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::IntoResponse;
use futures::{SinkExt, StreamExt};
use tokio_stream::wrappers::BroadcastStream;
use tracing::{debug, warn};
use uuid::Uuid;

use ciab_core::traits::stream::StreamHandler;
use ciab_core::types::stream::StreamEvent;

/// Handle a WebSocket upgrade request for bidirectional sandbox event streaming.
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    handler: Arc<dyn StreamHandler>,
    sandbox_id: Uuid,
    session_id: Option<Uuid>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, handler, sandbox_id, session_id))
}

async fn handle_socket(
    socket: WebSocket,
    handler: Arc<dyn StreamHandler>,
    sandbox_id: Uuid,
    session_id: Option<Uuid>,
) {
    let receiver = match handler.subscribe(&sandbox_id).await {
        Ok(rx) => rx,
        Err(e) => {
            warn!("Failed to subscribe to sandbox {}: {}", sandbox_id, e);
            return;
        }
    };

    let (mut ws_sender, mut ws_receiver) = socket.split();
    let mut broadcast_stream = BroadcastStream::new(receiver);

    let handler_clone = handler.clone();

    // Task: forward broadcast events to the WebSocket client
    let send_task = tokio::spawn(async move {
        while let Some(result) = broadcast_stream.next().await {
            match result {
                Ok(event) => {
                    // Filter by session_id if provided
                    if let Some(sid) = session_id {
                        if let Some(event_sid) = event.session_id {
                            if event_sid != sid {
                                continue;
                            }
                        }
                    }

                    let json = match serde_json::to_string(&event) {
                        Ok(j) => j,
                        Err(e) => {
                            warn!("Failed to serialize event: {}", e);
                            continue;
                        }
                    };

                    if ws_sender.send(Message::Text(json.into())).await.is_err() {
                        debug!("WebSocket send failed, client likely disconnected");
                        break;
                    }
                }
                Err(_) => {
                    // Lagged behind — skip
                    continue;
                }
            }
        }
    });

    // Task: receive messages from the WebSocket client
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_receiver.next().await {
            match msg {
                Message::Text(text) => {
                    // Parse incoming JSON commands
                    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&text);
                    match parsed {
                        Ok(value) => {
                            debug!("Received WS command: {:?}", value);
                            // Handle client commands — e.g. send_message
                            if let Some(cmd) = value.get("command").and_then(|v| v.as_str()) {
                                match cmd {
                                    "send_message" => {
                                        if let Some(data) = value.get("data") {
                                            let event = StreamEvent {
                                                id: Uuid::new_v4().to_string(),
                                                sandbox_id,
                                                session_id,
                                                event_type:
                                                    ciab_core::types::stream::StreamEventType::TextDelta,
                                                data: data.clone(),
                                                timestamp: chrono::Utc::now(),
                                            };
                                            if let Err(e) = handler_clone.publish(event).await {
                                                warn!("Failed to publish client message: {}", e);
                                            }
                                        }
                                    }
                                    other => {
                                        debug!("Unknown WS command: {}", other);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            warn!("Invalid JSON from WS client: {}", e);
                        }
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    });

    // Wait for either task to finish, then abort the other
    tokio::select! {
        _ = send_task => {}
        _ = recv_task => {}
    }
}
