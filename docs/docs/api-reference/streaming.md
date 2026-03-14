# Streaming API

CIAB uses Server-Sent Events (SSE) for real-time event delivery.

## Endpoints

| Endpoint | Scope |
|----------|-------|
| `GET /api/v1/sandboxes/{id}/stream` | All events for a sandbox |
| `GET /api/v1/sessions/{sid}/stream` | Events filtered to one session |
| `POST /api/v1/sandboxes/{id}/exec/stream` | Streaming command execution |

## Event Format

Each SSE message contains a JSON `StreamEvent`:

```
data: {"id":"evt_001","sandbox_id":"550e8400-...","session_id":"6ba7b810-...","event_type":"text_delta","data":{"text":"Hello"},"timestamp":"2026-03-11T10:05:00Z"}
```

## Event Types Reference

### Connection Events

| Type | Data | Description |
|------|------|-------------|
| `connected` | `{}` | Stream connection established |
| `reconnect` | `{ "after_ms": 1000 }` | Server requests reconnection |
| `keepalive` | `{}` | Heartbeat (every 15s) |

### Agent Output Events

| Type | Data | Description |
|------|------|-------------|
| `text_delta` | `{ "text": "..." }` | Incremental text chunk |
| `text_complete` | `{ "text": "..." }` | Full completed message |
| `tool_use_start` | `{ "id": "...", "name": "Read", "input": {...} }` | Agent started a tool |
| `tool_use_complete` | `{ "id": "..." }` | Tool execution finished |
| `tool_result` | `{ "tool_use_id": "...", "content": "...", "is_error": false }` | Tool result |

### Sandbox Events

| Type | Data | Description |
|------|------|-------------|
| `sandbox_state_changed` | `{ "from": "creating", "to": "running" }` | State transition |
| `stats` | `{ "cpu_usage_percent": 23.5, ... }` | Resource snapshot |
| `log_line` | `{ "line": "...", "stream": "stdout" }` | Container log |

### Provisioning Events

| Type | Data | Description |
|------|------|-------------|
| `provisioning_step` | `{ "step": "CloneRepositories", "status": "in_progress" }` | Pipeline step progress |
| `provisioning_complete` | `{ "sandbox_id": "..." }` | Pipeline succeeded |
| `provisioning_failed` | `{ "error": "...", "step": "StartAgent" }` | Pipeline failed |

### Session Events

| Type | Data | Description |
|------|------|-------------|
| `session_created` | `{ "session_id": "..." }` | New session |
| `session_completed` | `{ "session_id": "..." }` | Session finished |
| `session_failed` | `{ "session_id": "...", "error": "..." }` | Session error |
| `error` | `{ "code": "...", "message": "..." }` | General error |

## Client Examples

=== "JavaScript"

    ```javascript
    const es = new EventSource(
      `${baseUrl}/api/v1/sessions/${sessionId}/stream`
    );

    es.onmessage = (event) => {
      const data = JSON.parse(event.data);
      console.log(data.event_type, data.data);
    };

    es.onerror = () => {
      console.log('Connection lost, reconnecting...');
    };
    ```

=== "curl"

    ```bash
    curl -N http://localhost:8080/api/v1/sandboxes/<id>/stream
    ```

=== "Python"

    ```python
    import sseclient
    import requests

    response = requests.get(
        f"{base_url}/api/v1/sessions/{sid}/stream",
        stream=True
    )
    client = sseclient.SSEClient(response)
    for event in client.events():
        data = json.loads(event.data)
        print(data["event_type"], data["data"])
    ```
