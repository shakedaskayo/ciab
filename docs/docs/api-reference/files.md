# Files API

Manage files inside a sandbox's filesystem.

## List Files

```
GET /api/v1/sandboxes/{id}/files
```

**Query Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `path` | string | `/` | Directory path to list |

**Response:** `200 OK`

```json
[
  {
    "path": "/workspace/src/main.rs",
    "size": 1234,
    "is_dir": false,
    "mode": 33188,
    "modified_at": "2026-03-11T10:00:00Z"
  },
  {
    "path": "/workspace/src",
    "size": 4096,
    "is_dir": true,
    "mode": 16877,
    "modified_at": "2026-03-11T10:00:00Z"
  }
]
```

---

## Download File

```
GET /api/v1/sandboxes/{id}/files/{*path}
```

Returns the raw file content as `application/octet-stream`.

**Example:**

```bash
curl -o main.rs http://localhost:8080/api/v1/sandboxes/<id>/files/workspace/src/main.rs
```

---

## Upload File

```
PUT /api/v1/sandboxes/{id}/files/{*path}
```

Uploads a file. The request body is the raw file content.

**Example:**

```bash
curl -X PUT \
  --data-binary @./config.json \
  http://localhost:8080/api/v1/sandboxes/<id>/files/workspace/config.json
```

**Response:** `200 OK`

---

## Delete File

```
DELETE /api/v1/sandboxes/{id}/files/{*path}
```

**Response:** `204 No Content`
