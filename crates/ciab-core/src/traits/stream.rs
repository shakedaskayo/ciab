use async_trait::async_trait;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::error::CiabResult;
use crate::types::stream::StreamEvent;

#[async_trait]
pub trait StreamHandler: Send + Sync {
    /// Publish an event for a sandbox.
    async fn publish(&self, event: StreamEvent) -> CiabResult<()>;

    /// Subscribe to events for a sandbox, returning a broadcast receiver.
    async fn subscribe(&self, sandbox_id: &Uuid) -> CiabResult<broadcast::Receiver<StreamEvent>>;

    /// Subscribe to events and replay buffered events for reconnection support.
    /// Returns `(replayed_events, live_receiver)`.
    ///
    /// When `last_event_id` is provided, replays events after that ID.
    /// When `None`, replays the entire buffer so reconnecting clients catch up.
    async fn subscribe_with_replay(
        &self,
        sandbox_id: &Uuid,
        last_event_id: Option<&str>,
    ) -> CiabResult<(Vec<StreamEvent>, broadcast::Receiver<StreamEvent>)>;

    /// Unsubscribe / clean up resources for a sandbox stream.
    async fn unsubscribe(&self, sandbox_id: &Uuid) -> CiabResult<()>;
}
