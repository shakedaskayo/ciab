use async_trait::async_trait;
use ciab_core::error::{CiabError, CiabResult};
use ciab_core::traits::channel::ChannelAdapter;
use ciab_core::types::channel::{ChannelState, InboundMessage};
use tokio::sync::mpsc;

/// Slack adapter — placeholder for Socket Mode / webhook integration.
pub struct SlackAdapter;

#[async_trait]
impl ChannelAdapter for SlackAdapter {
    fn provider_name(&self) -> &str {
        "slack"
    }

    async fn start(&self) -> CiabResult<mpsc::Receiver<InboundMessage>> {
        Err(CiabError::ChannelAdapterError(
            "Slack adapter not yet implemented".to_string(),
        ))
    }

    async fn send(&self, _recipient_id: &str, _content: &str) -> CiabResult<()> {
        Err(CiabError::ChannelAdapterError(
            "Slack adapter not yet implemented".to_string(),
        ))
    }

    fn state(&self) -> ChannelState {
        ChannelState::Inactive
    }

    async fn shutdown(&self) -> CiabResult<()> {
        Ok(())
    }
}
