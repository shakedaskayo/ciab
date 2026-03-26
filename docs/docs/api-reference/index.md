# API Reference

The CIAB REST API provides full programmatic control over sandboxes, sessions, files, credentials, and OAuth flows.

## Base URL

```
http://localhost:8080
```

The port is configurable via `server.port` in `config.toml`.

## Authentication

If API keys are configured in `security.api_keys`, requests must include an `Authorization` header:

```
Authorization: Bearer <api-key>
```

If no API keys are configured, authentication is disabled.

## Common Response Format

### Success

Responses return JSON with appropriate HTTP status codes:

- `200 OK` — Successful read/update
- `201 Created` — Resource created
- `202 Accepted` — Async operation started (e.g., sandbox creation)
- `204 No Content` — Successful deletion

### Errors

All errors return JSON with an `error` object:

```json
{
  "error": {
    "code": "sandbox_not_found",
    "message": "Sandbox with ID 550e8400-... not found"
  }
}
```

### Error Codes

| HTTP Status | Error Codes |
|-------------|-------------|
| 400 | `config_error`, `config_validation_error`, `agent_communication_error` |
| 401 | `unauthorized` |
| 403 | `forbidden` |
| 404 | `sandbox_not_found`, `session_not_found`, `credential_not_found`, `file_not_found`, `agent_provider_not_found` |
| 409 | `sandbox_already_exists`, `sandbox_invalid_state`, `session_invalid_state` |
| 429 | `rate_limited` |
| 504 | `sandbox_timeout`, `timeout` |
| 500 | Internal server errors |

## Health Endpoints

### `GET /health`

Returns `200 OK` if the server is running.

### `GET /ready`

Returns `200 OK` if the server is ready to accept requests (database connected, runtime available).

## Content Types

| Operation | Content Type |
|-----------|-------------|
| Most requests/responses | `application/json` |
| File downloads | `application/octet-stream` |
| File uploads | Raw bytes in request body |
| SSE streams | `text/event-stream` |

## API Sections

| Section | Description |
|---------|-------------|
| [Sandboxes](sandboxes.md) | Sandbox lifecycle management |
| [Sessions](sessions.md) | Chat session management |
| [Execution](exec.md) | Command execution |
| [Files](files.md) | File operations |
| [Credentials](credentials.md) | Credential management |
| [OAuth](oauth.md) | OAuth flows |
| [Workspaces](workspaces.md) | Workspace management |
| [Images](images.md) | Machine image builds (Packer / AMI) |
| [Streaming](streaming.md) | SSE event streams |
