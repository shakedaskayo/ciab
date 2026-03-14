# Sandboxes API

## Create Sandbox

```
POST /api/v1/sandboxes
```

Creates a new sandbox and starts the provisioning pipeline.

**Request Body:**

```json
{
  "agent_provider": "claude-code",
  "name": "my-project",
  "image": null,
  "resource_limits": {
    "cpu_cores": 2,
    "memory_mb": 2048,
    "disk_mb": 10240,
    "max_processes": 100
  },
  "persistence": "ephemeral",
  "network": {
    "enabled": true,
    "allowed_hosts": [],
    "dns_servers": []
  },
  "env_vars": {
    "ANTHROPIC_API_KEY": "sk-ant-..."
  },
  "volumes": [],
  "ports": [],
  "git_repos": [
    {
      "url": "https://github.com/user/repo.git",
      "branch": "main",
      "dest_path": "/workspace/repo",
      "credential_id": null,
      "depth": 1
    }
  ],
  "credentials": [],
  "provisioning_scripts": [],
  "labels": { "team": "backend" },
  "timeout_secs": 300,
  "agent_config": {
    "provider": "claude-code",
    "model": "claude-sonnet-4-20250514",
    "system_prompt": null,
    "max_tokens": null,
    "tools_enabled": true,
    "allowed_tools": [],
    "denied_tools": []
  }
}
```

Only `agent_provider` is required. All other fields have defaults.

**Response:** `202 Accepted`

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "state": "pending",
  "name": "my-project",
  "agent_provider": "claude-code"
}
```

Subscribe to `GET /api/v1/sandboxes/{id}/stream` to follow provisioning progress.

---

## List Sandboxes

```
GET /api/v1/sandboxes
```

**Query Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `state` | string | Filter by state (e.g., `running`, `paused`) |
| `provider` | string | Filter by agent provider |
| `labels` | string | Comma-separated `key=value` label filters |

**Response:** `200 OK`

```json
[
  {
    "id": "550e8400-...",
    "name": "my-project",
    "state": "running",
    "persistence": "ephemeral",
    "agent_provider": "claude-code",
    "created_at": "2026-03-11T10:00:00Z",
    "updated_at": "2026-03-11T10:01:00Z"
  }
]
```

---

## Get Sandbox

```
GET /api/v1/sandboxes/{id}
```

**Response:** `200 OK` — Full `SandboxInfo` object including `spec`, `resource_stats`, and `endpoint_url`.

---

## Delete Sandbox

```
DELETE /api/v1/sandboxes/{id}
```

Terminates and deletes the sandbox. This is irreversible.

**Response:** `204 No Content`

---

## Sandbox Actions

### Start

```
POST /api/v1/sandboxes/{id}/start
```

Starts a stopped sandbox. **Response:** `200 OK`

### Stop

```
POST /api/v1/sandboxes/{id}/stop
```

Stops a running or paused sandbox. **Response:** `200 OK`

### Pause

```
POST /api/v1/sandboxes/{id}/pause
```

Freezes a running sandbox, preserving state. **Response:** `200 OK`

### Resume

```
POST /api/v1/sandboxes/{id}/resume
```

Resumes a paused sandbox. **Response:** `200 OK`

---

## Get Statistics

```
GET /api/v1/sandboxes/{id}/stats
```

**Response:** `200 OK`

```json
{
  "cpu_usage_percent": 23.5,
  "memory_used_mb": 512,
  "memory_limit_mb": 2048,
  "disk_used_mb": 150,
  "disk_limit_mb": 10240,
  "network_rx_bytes": 1048576,
  "network_tx_bytes": 524288
}
```

---

## Get Logs

```
GET /api/v1/sandboxes/{id}/logs
```

**Query Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `follow` | bool | false | Stream logs in real-time |
| `tail` | int | 100 | Number of recent lines |

---

## Event Stream

```
GET /api/v1/sandboxes/{id}/stream
```

Returns a Server-Sent Events stream with all sandbox events. See [Streaming](streaming.md) for event types.
