pub mod adapters;
pub mod router;

use std::collections::HashMap;
use std::sync::Arc;

use chrono::Utc;
use ciab_core::error::{CiabError, CiabResult};
use ciab_core::traits::channel::ChannelAdapter;
use ciab_core::types::channel::{Channel, ChannelFilters, ChannelState};
use ciab_db::Database;
use dashmap::DashMap;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use uuid::Uuid;

use crate::adapters::webhook::WebhookAdapter;

/// Per-sender session tracking
#[derive(Debug, Clone)]
pub struct SenderSession {
    pub sandbox_id: Uuid,
    pub session_id: Uuid,
    pub created_at: chrono::DateTime<Utc>,
}

/// A running channel with its adapter and routing task
struct RunningChannel {
    adapter: Arc<dyn ChannelAdapter>,
    routing_task: JoinHandle<()>,
    sender_sessions: Arc<DashMap<String, SenderSession>>,
}

/// Manages the lifecycle of all channels.
pub struct ChannelManager {
    db: Arc<Database>,
    running: Arc<RwLock<HashMap<Uuid, RunningChannel>>>,
}

impl ChannelManager {
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            db,
            running: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start a channel by ID.
    pub async fn start_channel(&self, id: &Uuid) -> CiabResult<()> {
        let channel = self
            .db
            .get_channel(id)
            .await?
            .ok_or_else(|| CiabError::ChannelNotFound(id.to_string()))?;

        let adapter: Arc<dyn ChannelAdapter> = match &channel.provider_config {
            ciab_core::types::channel::ChannelProviderConfig::Webhook {
                inbound_secret,
                outbound_url,
                outbound_headers,
            } => Arc::new(WebhookAdapter::new(
                outbound_url.clone(),
                outbound_headers.clone(),
                inbound_secret.clone(),
            )),
            _ => {
                return Err(CiabError::ChannelAdapterError(format!(
                    "adapter for {:?} not yet implemented",
                    channel.provider
                )))
            }
        };

        let rx = adapter.start().await?;

        self.db
            .update_channel_state(id, &ChannelState::Connected, None)
            .await?;

        let sender_sessions = Arc::new(DashMap::new());
        let db = self.db.clone();
        let channel_id = *id;
        let adapter_clone = adapter.clone();
        let sessions = sender_sessions.clone();

        let routing_task = tokio::spawn(async move {
            router::run_routing_loop(rx, channel_id, adapter_clone, db, sessions).await;
        });

        let mut running = self.running.write().await;
        running.insert(
            *id,
            RunningChannel {
                adapter,
                routing_task,
                sender_sessions,
            },
        );

        tracing::info!(channel_id = %id, "channel started");
        Ok(())
    }

    /// Stop a running channel.
    pub async fn stop_channel(&self, id: &Uuid) -> CiabResult<()> {
        let mut running = self.running.write().await;
        if let Some(rc) = running.remove(id) {
            rc.adapter.shutdown().await?;
            rc.routing_task.abort();
        }

        self.db
            .update_channel_state(id, &ChannelState::Stopped, None)
            .await?;

        tracing::info!(channel_id = %id, "channel stopped");
        Ok(())
    }

    /// Restart a channel (stop + start).
    pub async fn restart_channel(&self, id: &Uuid) -> CiabResult<()> {
        self.stop_channel(id).await?;
        self.start_channel(id).await?;
        Ok(())
    }

    /// Get the current state of a channel.
    pub async fn channel_state(&self, id: &Uuid) -> CiabResult<ChannelState> {
        let running = self.running.read().await;
        if let Some(rc) = running.get(id) {
            Ok(rc.adapter.state())
        } else {
            let channel = self
                .db
                .get_channel(id)
                .await?
                .ok_or_else(|| CiabError::ChannelNotFound(id.to_string()))?;
            Ok(channel.state)
        }
    }

    /// Get QR code for WhatsApp pairing.
    pub async fn whatsapp_qr(&self, id: &Uuid) -> CiabResult<Option<String>> {
        let running = self.running.read().await;
        if let Some(rc) = running.get(id) {
            Ok(rc.adapter.qr_code())
        } else {
            let channel = self
                .db
                .get_channel(id)
                .await?
                .ok_or_else(|| CiabError::ChannelNotFound(id.to_string()))?;
            Ok(channel.qr_code)
        }
    }

    /// Start all channels that are in Connected state (for server restart).
    pub async fn start_all_active(&self) -> CiabResult<()> {
        let filters = ChannelFilters {
            state: Some(ChannelState::Connected),
            ..Default::default()
        };
        let channels = self.db.list_channels(&filters).await?;
        for channel in channels {
            if let Err(e) = self.start_channel(&channel.id).await {
                tracing::error!(channel_id = %channel.id, error = %e, "failed to restart channel");
            }
        }
        Ok(())
    }

    /// Shutdown all running channels.
    pub async fn shutdown(&self) -> CiabResult<()> {
        let mut running = self.running.write().await;
        for (id, rc) in running.drain() {
            if let Err(e) = rc.adapter.shutdown().await {
                tracing::error!(channel_id = %id, error = %e, "error shutting down channel");
            }
            rc.routing_task.abort();
        }
        Ok(())
    }
}
