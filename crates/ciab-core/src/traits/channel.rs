use async_trait::async_trait;
use tokio::sync::mpsc;

use crate::error::CiabResult;
use crate::types::channel::{ChannelState, InboundMessage};

/// Adapter trait for messaging platform integrations.
#[async_trait]
pub trait ChannelAdapter: Send + Sync {
    /// Human-readable provider name (e.g. "whatsapp", "slack", "webhook").
    fn provider_name(&self) -> &str;

    /// Start the adapter and return a receiver for inbound messages.
    async fn start(&self) -> CiabResult<mpsc::Receiver<InboundMessage>>;

    /// Send a message back to the platform.
    async fn send(&self, recipient_id: &str, content: &str) -> CiabResult<()>;

    /// Current adapter state.
    fn state(&self) -> ChannelState;

    /// QR code data (WhatsApp pairing).
    fn qr_code(&self) -> Option<String> {
        None
    }

    /// Gracefully shut down the adapter.
    async fn shutdown(&self) -> CiabResult<()>;
}
