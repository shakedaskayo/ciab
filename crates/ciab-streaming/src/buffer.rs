use std::collections::VecDeque;

use ciab_core::types::stream::StreamEvent;

/// A ring buffer that stores recent stream events for replay on reconnect.
pub struct EventBuffer {
    events: VecDeque<StreamEvent>,
    capacity: usize,
}

impl EventBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            events: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    /// Push an event into the buffer. If at capacity, the oldest event is removed.
    pub fn push(&mut self, event: StreamEvent) {
        if self.events.len() >= self.capacity {
            self.events.pop_front();
        }
        self.events.push_back(event);
    }

    /// Return all events that occurred after the event with the given ID.
    /// If the ID is not found, returns **all** buffered events so the client
    /// can catch up even if its last known event has been evicted.
    pub fn replay_from(&self, last_event_id: &str) -> Vec<StreamEvent> {
        let position = self.events.iter().position(|e| e.id == last_event_id);
        match position {
            Some(idx) => self.events.iter().skip(idx + 1).cloned().collect(),
            None => self.events.iter().cloned().collect(),
        }
    }

    /// Return all buffered events.
    pub fn all(&self) -> Vec<StreamEvent> {
        self.events.iter().cloned().collect()
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ciab_core::types::stream::StreamEventType;
    use serde_json::json;
    use uuid::Uuid;

    fn make_event(id: &str) -> StreamEvent {
        StreamEvent {
            id: id.to_string(),
            sandbox_id: Uuid::nil(),
            session_id: None,
            event_type: StreamEventType::TextDelta,
            data: json!({"text": id}),
            timestamp: chrono::Utc::now(),
        }
    }

    #[test]
    fn replay_from_known_id_returns_subsequent_events() {
        let mut buf = EventBuffer::new(10);
        buf.push(make_event("a"));
        buf.push(make_event("b"));
        buf.push(make_event("c"));

        let replayed = buf.replay_from("a");
        let ids: Vec<_> = replayed.iter().map(|e| e.id.as_str()).collect();
        assert_eq!(ids, vec!["b", "c"]);
    }

    #[test]
    fn replay_from_last_event_returns_empty() {
        let mut buf = EventBuffer::new(10);
        buf.push(make_event("a"));
        buf.push(make_event("b"));

        let replayed = buf.replay_from("b");
        assert!(replayed.is_empty());
    }

    #[test]
    fn replay_from_unknown_id_returns_all_events() {
        let mut buf = EventBuffer::new(10);
        buf.push(make_event("a"));
        buf.push(make_event("b"));
        buf.push(make_event("c"));

        let replayed = buf.replay_from("unknown");
        let ids: Vec<_> = replayed.iter().map(|e| e.id.as_str()).collect();
        assert_eq!(ids, vec!["a", "b", "c"]);
    }

    #[test]
    fn all_returns_everything() {
        let mut buf = EventBuffer::new(10);
        buf.push(make_event("x"));
        buf.push(make_event("y"));

        let all = buf.all();
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].id, "x");
        assert_eq!(all[1].id, "y");
    }

    #[test]
    fn capacity_evicts_oldest() {
        let mut buf = EventBuffer::new(3);
        buf.push(make_event("a"));
        buf.push(make_event("b"));
        buf.push(make_event("c"));
        buf.push(make_event("d")); // evicts "a"

        assert_eq!(buf.len(), 3);
        let all = buf.all();
        let ids: Vec<_> = all.iter().map(|e| e.id.as_str()).collect();
        assert_eq!(ids, vec!["b", "c", "d"]);
    }

    #[test]
    fn replay_from_evicted_id_returns_all_remaining() {
        let mut buf = EventBuffer::new(3);
        buf.push(make_event("a"));
        buf.push(make_event("b"));
        buf.push(make_event("c"));
        buf.push(make_event("d")); // evicts "a"

        // "a" was evicted, so replay_from returns everything in the buffer
        let replayed = buf.replay_from("a");
        let ids: Vec<_> = replayed.iter().map(|e| e.id.as_str()).collect();
        assert_eq!(ids, vec!["b", "c", "d"]);
    }

    #[test]
    fn empty_buffer() {
        let buf = EventBuffer::new(10);
        assert!(buf.is_empty());
        assert_eq!(buf.len(), 0);
        assert!(buf.all().is_empty());
        assert!(buf.replay_from("anything").is_empty());
    }
}
