# Workspace Endpoints

## Create Workspace

```
POST /api/v1/workspaces
```

### Request Body

```json
{
  "name": "my-workspace",
  "description": "Project workspace",
  "spec": { ... },
  "labels": { "team": "platform" }
}
```

### Response `201 Created`

Returns the created `Workspace` object.

---

## List Workspaces

```
GET /api/v1/workspaces
```

### Query Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `name` | string | Filter by name (substring match) |
| `labels` | string | Comma-separated `key=value` pairs |

---

## Get Workspace

```
GET /api/v1/workspaces/{id}
```

---

## Update Workspace

```
PUT /api/v1/workspaces/{id}
```

### Request Body

All fields optional — only provided fields are updated.

```json
{
  "name": "updated-name",
  "description": "updated description",
  "spec": { ... },
  "labels": { ... }
}
```

---

## Delete Workspace

```
DELETE /api/v1/workspaces/{id}
```

Returns `204 No Content`.

---

## Launch Workspace

Create a sandbox from a workspace definition.

```
POST /api/v1/workspaces/{id}/launch
```

### Response `202 Accepted`

```json
{
  "sandbox_id": "uuid",
  "workspace_id": "uuid",
  "status": "provisioning"
}
```

---

## List Workspace Sandboxes

```
GET /api/v1/workspaces/{id}/sandboxes
```

Returns sandbox IDs created from this workspace.

---

## Export as TOML

```
GET /api/v1/workspaces/{id}/export
```

Returns `Content-Type: application/toml` with the workspace spec as TOML.

---

## Import from TOML

```
POST /api/v1/workspaces/import
Content-Type: text/plain
```

Body is raw TOML content. Returns the created `Workspace` object.
