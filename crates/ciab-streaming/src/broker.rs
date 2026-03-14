use async_trait::async_trait;
use dashmap::DashMap;
use tokio::sync::{broadcast, Mutex};
use uuid::Uuid;

use ciab_core::error::CiabResult;
use ciab_core::traits::stream::StreamHandler;
use ciab_core::types::stream::StreamEvent;

use crate::buffer::EventBuffer;

const DEFAULT_BROADCAST_CAPACITY: usize = 1024;

pub struct StreamBroker {
    channels: DashMap<Uuid, broadcast::Sender<StreamEvent>>,
    buffers: DashMap<Uuid, Mutex<EventBuffer>>,
    buffer_capacity: usize,
}

impl StreamBroker {
    pub fn new(buffer_capacity: usize) -> Self {
        Self {
            channels: DashMap::new(),
            buffers: DashMap::new(),
            buffer_capacity,
        }
    }

    /// Get or create a broadcast sender for a sandbox.
    fn get_or_create_sender(&self, sandbox_id: &Uuid) -> broadcast::Sender<StreamEvent> {
        self.channels
            .entry(*sandbox_id)
            .or_insert_with(|| broadcast::channel(DEFAULT_BROADCAST_CAPACITY).0)
            .clone()
    }

    /// Get or create a buffer for a sandbox.
    fn get_or_create_buffer(
        &self,
        sandbox_id: &Uuid,
    ) -> dashmap::mapref::one::Ref<'_, Uuid, Mutex<EventBuffer>> {
        self.buffers
            .entry(*sandbox_id)
            .or_insert_with(|| Mutex::new(EventBuffer::new(self.buffer_capacity)));
        self.buffers.get(sandbox_id).unwrap()
    }

    /// Subscribe with optional session filtering and replay support.
    ///
    /// When `last_event_id` is provided, replays all events after that ID.
    /// When `replay_all` is true and no `last_event_id` is given, replays
    /// the entire buffer so reconnecting clients can catch up.
    pub async fn subscribe_with_options(
        &self,
        sandbox_id: &Uuid,
        _session_id: Option<Uuid>,
        last_event_id: Option<&str>,
    ) -> CiabResult<(Vec<StreamEvent>, broadcast::Receiver<StreamEvent>)> {
        let sender = self.get_or_create_sender(sandbox_id);
        let receiver = sender.subscribe();

        let replay = if let Some(last_id) = last_event_id {
            let buf_ref = self.get_or_create_buffer(sandbox_id);
            let buf = buf_ref.lock().await;
            buf.replay_from(last_id)
        } else {
            Vec::new()
        };

        Ok((replay, receiver))
    }

    /// Subscribe and always replay the full buffer. Used for fresh connections
    /// that want to catch up on all recent activity.
    pub async fn subscribe_with_replay(
        &self,
        sandbox_id: &Uuid,
        last_event_id: Option<&str>,
    ) -> CiabResult<(Vec<StreamEvent>, broadcast::Receiver<StreamEvent>)> {
        let sender = self.get_or_create_sender(sandbox_id);
        let receiver = sender.subscribe();

        let buf_ref = self.get_or_create_buffer(sandbox_id);
        let buf = buf_ref.lock().await;
        let replay = match last_event_id {
            Some(id) => buf.replay_from(id),
            None => buf.all(),
        };

        Ok((replay, receiver))
    }

    /// Replay events from the buffer after the given event ID.
    pub async fn replay_from(&self, sandbox_id: &Uuid, last_event_id: &str) -> Vec<StreamEvent> {
        let buf_ref = self.get_or_create_buffer(sandbox_id);
        let buf = buf_ref.lock().await;
        buf.replay_from(last_event_id)
    }

    /// Return the number of buffered events for a sandbox.
    pub async fn buffer_size(&self, sandbox_id: &Uuid) -> usize {
        let buf_ref = self.get_or_create_buffer(sandbox_id);
        let buf = buf_ref.lock().await;
        buf.len()
    }

    /// Remove all channels and buffers for a sandbox.
    pub fn remove_sandbox(&self, sandbox_id: &Uuid) {
        self.channels.remove(sandbox_id);
        self.buffers.remove(sandbox_id);
    }
}

#[async_trait]
impl StreamHandler for StreamBroker {
    async fn publish(&self, event: StreamEvent) -> CiabResult<()> {
        let sandbox_id = event.sandbox_id;

        // Buffer the event
        {
            let buf_ref = self.get_or_create_buffer(&sandbox_id);
            let mut buf = buf_ref.lock().await;
            buf.push(event.clone());
        }

        // Broadcast — ignore SendError (no active receivers is fine)
        let sender = self.get_or_create_sender(&sandbox_id);
        let _ = sender.send(event);

        Ok(())
    }

    async fn subscribe(&self, sandbox_id: &Uuid) -> CiabResult<broadcast::Receiver<StreamEvent>> {
        let sender = self.get_or_create_sender(sandbox_id);
        Ok(sender.subscribe())
    }

    async fn subscribe_with_replay(
        &self,
        sandbox_id: &Uuid,
        last_event_id: Option<&str>,
    ) -> CiabResult<(Vec<StreamEvent>, broadcast::Receiver<StreamEvent>)> {
        // Subscribe first so no events are missed between replay and live
        let sender = self.get_or_create_sender(sandbox_id);
        let receiver = sender.subscribe();

        let buf_ref = self.get_or_create_buffer(sandbox_id);
        let buf = buf_ref.lock().await;
        let replay = match last_event_id {
            Some(id) => buf.replay_from(id),
            None => buf.all(),
        };

        Ok((replay, receiver))
    }

    async fn unsubscribe(&self, sandbox_id: &Uuid) -> CiabResult<()> {
        // Unsubscribe cleans up the sandbox resources
        self.remove_sandbox(sandbox_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ciab_core::types::stream::StreamEventType;
    use serde_json::json;

    fn make_event(id: &str, sandbox_id: Uuid) -> StreamEvent {
        StreamEvent {
            id: id.to_string(),
            sandbox_id,
            session_id: None,
            event_type: StreamEventType::TextDelta,
            data: json!({"text": id}),
            timestamp: chrono::Utc::now(),
        }
    }

    #[tokio::test]
    async fn publish_buffers_events() {
        let broker = StreamBroker::new(100);
        let sid = Uuid::new_v4();

        broker.publish(make_event("e1", sid)).await.unwrap();
        broker.publish(make_event("e2", sid)).await.unwrap();

        assert_eq!(broker.buffer_size(&sid).await, 2);
    }

    #[tokio::test]
    async fn subscribe_receives_live_events() {
        let broker = StreamBroker::new(100);
        let sid = Uuid::new_v4();

        let mut rx = broker.subscribe(&sid).await.unwrap();
        broker.publish(make_event("e1", sid)).await.unwrap();

        let event = rx.recv().await.unwrap();
        assert_eq!(event.id, "e1");
    }

    #[tokio::test]
    async fn subscribe_with_replay_returns_full_buffer_when_no_last_id() {
        let broker = StreamBroker::new(100);
        let sid = Uuid::new_v4();

        broker.publish(make_event("e1", sid)).await.unwrap();
        broker.publish(make_event("e2", sid)).await.unwrap();
        broker.publish(make_event("e3", sid)).await.unwrap();

        let (replay, _rx) = broker.subscribe_with_replay(&sid, None).await.unwrap();
        let ids: Vec<_> = replay.iter().map(|e| e.id.as_str()).collect();
        assert_eq!(ids, vec!["e1", "e2", "e3"]);
    }

    #[tokio::test]
    async fn subscribe_with_replay_returns_events_after_last_id() {
        let broker = StreamBroker::new(100);
        let sid = Uuid::new_v4();

        broker.publish(make_event("e1", sid)).await.unwrap();
        broker.publish(make_event("e2", sid)).await.unwrap();
        broker.publish(make_event("e3", sid)).await.unwrap();

        let (replay, _rx) = broker
            .subscribe_with_replay(&sid, Some("e1"))
            .await
            .unwrap();
        let ids: Vec<_> = replay.iter().map(|e| e.id.as_str()).collect();
        assert_eq!(ids, vec!["e2", "e3"]);
    }

    #[tokio::test]
    async fn subscribe_with_replay_unknown_id_returns_all() {
        let broker = StreamBroker::new(100);
        let sid = Uuid::new_v4();

        broker.publish(make_event("e1", sid)).await.unwrap();
        broker.publish(make_event("e2", sid)).await.unwrap();

        let (replay, _rx) = broker
            .subscribe_with_replay(&sid, Some("unknown"))
            .await
            .unwrap();
        let ids: Vec<_> = replay.iter().map(|e| e.id.as_str()).collect();
        assert_eq!(ids, vec!["e1", "e2"]);
    }

    #[tokio::test]
    async fn replay_does_not_miss_events_published_during_subscribe() {
        let broker = StreamBroker::new(100);
        let sid = Uuid::new_v4();

        // Publish before subscribe
        broker.publish(make_event("e1", sid)).await.unwrap();

        // Subscribe with replay — gets e1 in replay, plus live receiver
        let (replay, mut rx) = broker.subscribe_with_replay(&sid, None).await.unwrap();
        assert_eq!(replay.len(), 1);
        assert_eq!(replay[0].id, "e1");

        // Publish after subscribe — should arrive on live receiver
        broker.publish(make_event("e2", sid)).await.unwrap();
        let live = rx.recv().await.unwrap();
        assert_eq!(live.id, "e2");
    }

    #[tokio::test]
    async fn publish_without_subscribers_still_buffers() {
        let broker = StreamBroker::new(100);
        let sid = Uuid::new_v4();

        // No subscribers — publish should still succeed and buffer
        broker.publish(make_event("e1", sid)).await.unwrap();
        broker.publish(make_event("e2", sid)).await.unwrap();
        assert_eq!(broker.buffer_size(&sid).await, 2);

        // Late subscriber can get replay
        let (replay, _rx) = broker.subscribe_with_replay(&sid, None).await.unwrap();
        assert_eq!(replay.len(), 2);
    }

    #[tokio::test]
    async fn remove_sandbox_clears_everything() {
        let broker = StreamBroker::new(100);
        let sid = Uuid::new_v4();

        broker.publish(make_event("e1", sid)).await.unwrap();
        assert_eq!(broker.buffer_size(&sid).await, 1);

        broker.remove_sandbox(&sid);

        // After removal, buffer should be fresh (empty)
        assert_eq!(broker.buffer_size(&sid).await, 0);
    }

    #[tokio::test]
    async fn multiple_sandboxes_are_isolated() {
        let broker = StreamBroker::new(100);
        let s1 = Uuid::new_v4();
        let s2 = Uuid::new_v4();

        broker.publish(make_event("a1", s1)).await.unwrap();
        broker.publish(make_event("b1", s2)).await.unwrap();
        broker.publish(make_event("b2", s2)).await.unwrap();

        assert_eq!(broker.buffer_size(&s1).await, 1);
        assert_eq!(broker.buffer_size(&s2).await, 2);

        let (replay_s1, _) = broker.subscribe_with_replay(&s1, None).await.unwrap();
        let (replay_s2, _) = broker.subscribe_with_replay(&s2, None).await.unwrap();

        assert_eq!(replay_s1.len(), 1);
        assert_eq!(replay_s1[0].id, "a1");
        assert_eq!(replay_s2.len(), 2);
    }
}
