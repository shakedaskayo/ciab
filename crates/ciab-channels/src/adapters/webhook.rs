use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use ciab_core::error::{CiabError, CiabResult};
use ciab_core::traits::channel::ChannelAdapter;
use ciab_core::types::channel::{ChannelState, InboundMessage};
use tokio::sync::{mpsc, RwLock};

/// Webhook adapter: receives messages via API endpoint, sends via HTTP POST.
pub struct WebhookAdapter {
    outbound_url: Option<String>,
    outbound_headers: HashMap<String, String>,
    _inbound_secret: Option<String>,
    state: Arc<RwLock<ChannelState>>,
    tx: Arc<RwLock<Option<mpsc::Sender<InboundMessage>>>>,
}

impl WebhookAdapter {
    pub fn new(
        outbound_url: Option<String>,
        outbound_headers: HashMap<String, String>,
        inbound_secret: Option<String>,
    ) -> Self {
        Self {
            outbound_url,
            outbound_headers,
            _inbound_secret: inbound_secret,
            state: Arc::new(RwLock::new(ChannelState::Inactive)),
            tx: Arc::new(RwLock::new(None)),
        }
    }

    /// Push an inbound message from the webhook API endpoint.
    pub async fn push_inbound(&self, msg: InboundMessage) -> CiabResult<()> {
        let tx = self.tx.read().await;
        if let Some(ref sender) = *tx {
            sender
                .send(msg)
                .await
                .map_err(|e| CiabError::ChannelAdapterError(e.to_string()))?;
        }
        Ok(())
    }
}

#[async_trait]
impl ChannelAdapter for WebhookAdapter {
    fn provider_name(&self) -> &str {
        "webhook"
    }

    async fn start(&self) -> CiabResult<mpsc::Receiver<InboundMessage>> {
        let (tx, rx) = mpsc::channel(256);
        *self.tx.write().await = Some(tx);
        *self.state.write().await = ChannelState::Connected;
        Ok(rx)
    }

    async fn send(&self, _recipient_id: &str, content: &str) -> CiabResult<()> {
        if let Some(ref url) = self.outbound_url {
            let client = reqwest::Client::new();
            let mut req = client.post(url).json(&serde_json::json!({
                "content": content,
            }));

            for (key, value) in &self.outbound_headers {
                req = req.header(key, value);
            }

            req.send()
                .await
                .map_err(|e| CiabError::ChannelAdapterError(e.to_string()))?;
        }
        Ok(())
    }

    fn state(&self) -> ChannelState {
        // Use try_read to avoid blocking; fall back to Connected
        self.state
            .try_read()
            .map(|s| s.clone())
            .unwrap_or(ChannelState::Connected)
    }

    async fn shutdown(&self) -> CiabResult<()> {
        *self.state.write().await = ChannelState::Stopped;
        *self.tx.write().await = None;
        Ok(())
    }
}
