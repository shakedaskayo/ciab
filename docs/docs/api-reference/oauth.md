# OAuth API

OAuth flows for connecting third-party services (GitHub, GCP, etc.) to sandboxes.

## Authorization Code Flow

### Start Authorization

```
GET /api/v1/oauth/{provider}/authorize
```

Redirects the user to the OAuth provider's authorization page.

**Query Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `redirect_uri` | string | Override callback URL |

**Supported Providers:** Configured in `oauth.providers` in `config.toml`. Common providers: `github`, `gcp`, `azure`.

### Handle Callback

```
GET /api/v1/oauth/{provider}/callback
```

OAuth callback endpoint. Exchanges the authorization code for tokens and stores them as a credential.

---

## Device Code Flow

For environments without a browser (e.g., SSH sessions).

### Get Device Code

```
GET /api/v1/oauth/{provider}/device-code
```

**Response:** `200 OK`

```json
{
  "device_code": "abc123",
  "user_code": "ABCD-1234",
  "verification_uri": "https://github.com/login/device",
  "expires_in": 900,
  "interval": 5
}
```

Display the `user_code` and `verification_uri` to the user.

### Poll for Completion

```
POST /api/v1/oauth/{provider}/device-poll
```

**Request Body:**

```json
{
  "device_code": "abc123"
}
```

**Response:** `200 OK` when authorized, `202 Accepted` while pending.

---

## Refresh Token

```
POST /api/v1/oauth/{provider}/refresh
```

**Request Body:**

```json
{
  "credential_id": "cred-550e8400-..."
}
```

Refreshes the OAuth token for the specified credential.

**Response:** `200 OK`
