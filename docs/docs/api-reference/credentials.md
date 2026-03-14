# Credentials API

Manage encrypted credentials for agent access.

## Create Credential

```
POST /api/v1/credentials
```

**Request Body:**

```json
{
  "name": "anthropic-key",
  "credential_type": "api_key",
  "data": {
    "ANTHROPIC_API_KEY": "sk-ant-..."
  },
  "labels": {
    "provider": "anthropic"
  },
  "expires_at": null
}
```

**Credential Types:**

| Type | Description | Data Format |
|------|-------------|-------------|
| `api_key` | Single API key | `{ "KEY_NAME": "value" }` |
| `env_vars` | Multiple environment variables | `{ "VAR1": "val1", "VAR2": "val2" }` |
| `git_token` | Git authentication token | `{ "token": "ghp_..." }` |
| `oauth_token` | OAuth access/refresh tokens | `{ "access_token": "...", "refresh_token": "..." }` |
| `ssh_key` | SSH private key | `{ "private_key": "-----BEGIN..." }` |
| `file` | Arbitrary file content | `{ "path": "/home/user/.config", "content": "..." }` |

**Response:** `201 Created`

```json
{
  "id": "cred-550e8400-...",
  "name": "anthropic-key",
  "credential_type": "api_key",
  "labels": { "provider": "anthropic" },
  "created_at": "2026-03-11T10:00:00Z",
  "expires_at": null
}
```

!!! warning "Secrets are Encrypted at Rest"
    Credential data is encrypted using AES-GCM before storage. The encryption key is read from the environment variable specified in `credentials.encryption_key_env`.

---

## List Credentials

```
GET /api/v1/credentials
```

Returns metadata only — secrets are never included in list responses.

---

## Get Credential

```
GET /api/v1/credentials/{id}
```

Returns metadata only.

---

## Delete Credential

```
DELETE /api/v1/credentials/{id}
```

**Response:** `204 No Content`
