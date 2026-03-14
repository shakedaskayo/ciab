# Sessions API

Sessions represent conversations with an agent inside a sandbox.

## Create Session

```
POST /api/v1/sandboxes/{id}/sessions
```

**Request Body:**

```json
{
  "metadata": {
    "purpose": "code-review"
  }
}
```

**Response:** `201 Created`

```json
{
  "id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
  "sandbox_id": "550e8400-...",
  "state": "active",
  "metadata": { "purpose": "code-review" },
  "created_at": "2026-03-11T10:00:00Z",
  "updated_at": "2026-03-11T10:00:00Z"
}
```

---

## List Sessions

```
GET /api/v1/sandboxes/{id}/sessions
```

**Response:** `200 OK` — Array of `Session` objects.

---

## Get Session

```
GET /api/v1/sessions/{sid}
```

Returns the session with its full message history.

**Response:** `200 OK`

```json
{
  "id": "6ba7b810-...",
  "sandbox_id": "550e8400-...",
  "state": "active",
  "metadata": {},
  "messages": [
    {
      "id": "msg-001",
      "session_id": "6ba7b810-...",
      "role": "user",
      "content": [
        { "type": "text", "text": "Explain the codebase" }
      ],
      "timestamp": "2026-03-11T10:05:00Z"
    },
    {
      "id": "msg-002",
      "session_id": "6ba7b810-...",
      "role": "assistant",
      "content": [
        { "type": "text", "text": "The project is structured as..." }
      ],
      "timestamp": "2026-03-11T10:05:03Z"
    }
  ],
  "created_at": "2026-03-11T10:00:00Z",
  "updated_at": "2026-03-11T10:05:03Z"
}
```

---

## Send Message

```
POST /api/v1/sessions/{sid}/messages
```

Sends a message to the agent and waits for a response.

**Request Body:**

```json
{
  "role": "user",
  "content": [
    { "type": "text", "text": "Refactor the auth module" }
  ]
}
```

**Response:** `200 OK` — The assistant's response message.

!!! tip "Use Streaming for Real-time Output"
    This endpoint blocks until the agent completes its response. For real-time output, subscribe to `GET /api/v1/sessions/{sid}/stream` before sending the message.

### Message Content Types

| Type | Fields | Description |
|------|--------|-------------|
| `text` | `text` | Plain text content |
| `tool_use` | `id`, `name`, `input` | Agent invoking a tool |
| `tool_result` | `tool_use_id`, `content`, `is_error` | Tool execution result |
| `image` | `media_type`, `data` | Base64-encoded image |

---

## Interrupt Session

```
POST /api/v1/sessions/{sid}/interrupt
```

Interrupts the agent's current processing.

**Response:** `200 OK`

---

## Session Event Stream

```
GET /api/v1/sessions/{sid}/stream
```

Returns SSE events filtered to this session: `text_delta`, `text_complete`, `tool_use_start`, `tool_use_complete`, `tool_result`, `session_completed`, `session_failed`. Also includes sandbox-wide events like `keepalive`.
