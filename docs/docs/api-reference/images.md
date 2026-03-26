# Images API

## Build Image

```
POST /api/v1/images/build
```

Start an asynchronous image build using Packer.

**Request Body:**

```json
{
  "provider": "claude-code",
  "region": "us-east-1",
  "instance_type": "t3.medium",
  "template_source": null,
  "variables": {
    "extra_packages": "ripgrep"
  }
}
```

Only `provider` is required. All other fields use config defaults.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `provider` | string | Yes | Agent provider to install (`claude-code`, `codex`, `gemini`, `cursor`) |
| `region` | string | No | AWS region for the AMI |
| `instance_type` | string | No | EC2 instance type for the build |
| `template_source` | string | No | Packer template source override |
| `variables` | object | No | Extra variables passed to the Packer template |

**Response:** `202 Accepted`

```json
{
  "build_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "queued",
  "provider": "claude-code",
  "region": "us-east-1",
  "created_at": "2026-03-25T14:30:00Z"
}
```

---

## List Images

```
GET /api/v1/images
```

List all available machine images.

**Query Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `region` | string | Filter by AWS region |
| `provider` | string | Filter by agent provider |

**Response:** `200 OK`

```json
{
  "images": [
    {
      "image_id": "ami-0abcdef1234567890",
      "provider": "claude-code",
      "region": "us-east-1",
      "status": "available",
      "created_at": "2026-03-25T14:30:00Z",
      "build_id": "550e8400-e29b-41d4-a716-446655440000"
    }
  ]
}
```

---

## Get Build Status

```
GET /api/v1/images/builds/{build_id}
```

Get the current status of an image build.

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `build_id` | UUID | The build ID returned from `POST /api/v1/images/build` |

**Response:** `200 OK`

```json
{
  "build_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "succeeded",
  "provider": "claude-code",
  "region": "us-east-1",
  "image_id": "ami-0abcdef1234567890",
  "started_at": "2026-03-25T14:30:00Z",
  "completed_at": "2026-03-25T14:45:00Z",
  "error": null
}
```

Build status values:

| Status | Description |
|--------|-------------|
| `queued` | Build is waiting to start |
| `building` | Packer is running |
| `succeeded` | AMI created successfully |
| `failed` | Build failed (see `error` field) |

When `status` is `failed`, the `error` field contains a description of the failure:

```json
{
  "build_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "failed",
  "error": "Packer build timed out after 1800 seconds"
}
```

---

## Delete Image

```
DELETE /api/v1/images/{image_id}
```

Deregister an AMI and delete the associated EBS snapshot.

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `image_id` | string | The AMI ID (e.g., `ami-0abcdef1234567890`) |

**Response:** `204 No Content`

Returns an empty response on success.

!!! warning
    This is irreversible. Running instances launched from this AMI are not affected, but new sandboxes cannot use the deleted image.
