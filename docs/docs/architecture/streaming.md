# Streaming Architecture

CIAB uses Server-Sent Events (SSE) for real-time communication between the server and clients.

## Components

### StreamBroker

The central pub/sub system. Each sandbox has a dedicated broadcast channel.

- **Publish**: Any component can publish events via `StreamHandler::publish()`
- **Subscribe**: Clients subscribe to a sandbox's event stream
- **Buffering**: An `EventBuffer` stores up to 500 events per sandbox (configurable) for replay

### StreamEvent

Every event flowing through the system is a `StreamEvent`:

```json
{
  "id": "evt_abc123",
  "sandbox_id": "550e8400-e29b-41d4-a716-446655440000",
  "session_id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
  "event_type": "text_delta",
  "data": { "text": "Let me analyze " },
  "timestamp": "2026-03-11T10:30:00Z"
}
```

### Event Types

| Type | Description |
|------|-------------|
| `connected` | Client successfully connected to stream |
| `reconnect` | Server requests client reconnection |
| `keepalive` | Heartbeat (every 15s by default) |
| `text_delta` | Incremental text from agent |
| `text_complete` | Full text message complete |
| `tool_use_start` | Agent started using a tool |
| `tool_use_complete` | Tool execution finished |
| `tool_result` | Tool returned a result |
| `sandbox_state_changed` | Sandbox state transition |
| `provisioning_step` | Provisioning pipeline progress |
| `provisioning_complete` | Provisioning succeeded |
| `provisioning_failed` | Provisioning failed |
| `session_created` | New session started |
| `session_completed` | Session finished |
| `session_failed` | Session errored |
| `error` | General error |
| `stats` | Resource usage snapshot |
| `log_line` | Container log output |

## SSE Endpoints

### Sandbox Stream

```
GET /api/v1/sandboxes/{id}/stream
```

Streams **all** events for a sandbox: state changes, provisioning, all sessions, logs, stats.

### Session Stream

```
GET /api/v1/sessions/{sid}/stream
```

Streams events **filtered to one session**: text deltas, tool use, session state changes. Also includes sandbox-level broadcast events (state changes, keepalives).

## Client Integration

### EventSource (Browser/Desktop)

```javascript
const source = new EventSource(
  `http://localhost:8080/api/v1/sessions/${sessionId}/stream`
);

source.onmessage = (event) => {
  const data = JSON.parse(event.data);
  switch (data.event_type) {
    case 'text_delta':
      appendText(data.data.text);
      break;
    case 'tool_use_start':
      showToolUse(data.data);
      break;
  }
};
```

### curl

```bash
curl -N http://localhost:8080/api/v1/sandboxes/<id>/stream
```

## Configuration

```toml
[streaming]
buffer_size = 500              # Events buffered per sandbox
keepalive_interval_secs = 15   # Heartbeat interval
max_stream_duration_secs = 3600  # Max connection duration
```
