# Cursor

The Cursor provider runs Cursor CLI inside a sandboxed container.

## Configuration

```toml
[agents.providers.cursor]
enabled = false
image = "ghcr.io/shakedaskayo/ciab-cursor:latest"
api_key_env = "CURSOR_API_KEY"
```

## Required Environment

| Variable | Description |
|----------|-------------|
| `CURSOR_API_KEY` | Cursor API key |

## Container Image

Based on `node:22-slim` with:

- Cursor CLI (installed via shell script)
- Git, curl, ca-certificates, openssh-client

## Example

```bash
ciab sandbox create \
  --provider cursor \
  --env CURSOR_API_KEY=$CURSOR_API_KEY

ciab agent chat --sandbox-id <id> -m "Fix the failing tests" --stream
```
