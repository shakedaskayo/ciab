use async_trait::async_trait;
use ciab_core::error::{CiabError, CiabResult};
use ciab_core::traits::channel::ChannelAdapter;
use ciab_core::types::channel::{ChannelState, InboundMessage};
use tokio::sync::mpsc;

/// WhatsApp adapter — placeholder for whatsapp-rust integration.
pub struct WhatsAppAdapter;

#[async_trait]
impl ChannelAdapter for WhatsAppAdapter {
    fn provider_name(&self) -> &str {
        "whatsapp"
    }

    async fn start(&self) -> CiabResult<mpsc::Receiver<InboundMessage>> {
        Err(CiabError::ChannelAdapterError(
            "WhatsApp adapter not yet implemented".to_string(),
        ))
    }

    async fn send(&self, _recipient_id: &str, _content: &str) -> CiabResult<()> {
        Err(CiabError::ChannelAdapterError(
            "WhatsApp adapter not yet implemented".to_string(),
        ))
    }

    fn state(&self) -> ChannelState {
        ChannelState::Inactive
    }

    async fn shutdown(&self) -> CiabResult<()> {
        Ok(())
    }
}
