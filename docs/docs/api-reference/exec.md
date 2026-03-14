# Execution API

Execute commands inside a running sandbox.

## Execute Command

```
POST /api/v1/sandboxes/{id}/exec
```

Runs a command and returns the result when complete.

**Request Body:**

```json
{
  "command": ["ls", "-la", "/workspace"],
  "workdir": "/workspace",
  "env": {
    "MY_VAR": "value"
  },
  "stdin": null,
  "timeout_secs": 30,
  "tty": false
}
```

Only `command` is required.

**Response:** `200 OK`

```json
{
  "exit_code": 0,
  "stdout": "total 24\ndrwxr-xr-x 3 user user 4096 Mar 11 10:00 .\n...",
  "stderr": "",
  "duration_ms": 45
}
```

---

## Execute with Streaming

```
POST /api/v1/sandboxes/{id}/exec/stream
```

Same request body as above, but returns an SSE stream with stdout/stderr as it happens.

**SSE Events:**

```
data: {"type": "stdout", "text": "Installing dependencies...\n"}

data: {"type": "stderr", "text": "npm warn deprecated pkg@1.0\n"}

data: {"type": "exit", "code": 0, "duration_ms": 12340}
```

This is useful for long-running commands where you want real-time output.
