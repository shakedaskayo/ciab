use std::sync::Arc;

use chrono::Utc;
use ciab_core::traits::channel::ChannelAdapter;
use ciab_core::types::channel::{ChannelMessage, InboundMessage, MessageDirection};
use ciab_db::Database;
use dashmap::DashMap;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::SenderSession;

/// Core routing loop that processes inbound messages from a channel adapter.
pub async fn run_routing_loop(
    mut rx: mpsc::Receiver<InboundMessage>,
    channel_id: Uuid,
    adapter: Arc<dyn ChannelAdapter>,
    db: Arc<Database>,
    _sender_sessions: Arc<DashMap<String, SenderSession>>,
) {
    while let Some(msg) = rx.recv().await {
        let adapter = adapter.clone();
        let db = db.clone();
        let channel_id = channel_id;

        tokio::spawn(async move {
            if let Err(e) = process_message(&msg, channel_id, adapter.as_ref(), db.as_ref()).await {
                tracing::error!(
                    channel_id = %channel_id,
                    sender = %msg.sender_id,
                    error = %e,
                    "failed to process inbound message"
                );
            }
        });
    }

    tracing::info!(channel_id = %channel_id, "routing loop ended");
}

async fn process_message(
    msg: &InboundMessage,
    channel_id: Uuid,
    adapter: &dyn ChannelAdapter,
    db: &Database,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Log inbound message
    let inbound_log = ChannelMessage {
        id: Uuid::new_v4(),
        channel_id,
        direction: MessageDirection::Inbound,
        sender_id: msg.sender_id.clone(),
        sender_name: msg.sender_name.clone(),
        sandbox_id: None,
        session_id: None,
        content: msg.content.clone(),
        platform_metadata: msg.platform_metadata.clone(),
        timestamp: Utc::now(),
    };
    db.insert_channel_message(&inbound_log).await?;

    // For now, echo back a placeholder response.
    // Full routing (sandbox resolution, agent interaction) will be added
    // when the ChannelManager is wired to runtime + agents.
    let response = format!(
        "Message received by CIAB channel. Agent integration coming soon. You said: {}",
        truncate(&msg.content, 100)
    );

    adapter.send(&msg.sender_id, &response).await?;

    // Log outbound message
    let outbound_log = ChannelMessage {
        id: Uuid::new_v4(),
        channel_id,
        direction: MessageDirection::Outbound,
        sender_id: "ciab".to_string(),
        sender_name: Some("CIAB".to_string()),
        sandbox_id: None,
        session_id: None,
        content: response,
        platform_metadata: Default::default(),
        timestamp: Utc::now(),
    };
    db.insert_channel_message(&outbound_log).await?;

    Ok(())
}

fn truncate(s: &str, max_len: usize) -> &str {
    if s.len() <= max_len {
        s
    } else {
        &s[..max_len]
    }
}
