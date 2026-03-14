# Gemini CLI

The Gemini provider runs Gemini CLI inside a sandboxed container.

## Configuration

```toml
[agents.providers.gemini]
enabled = false
image = "ghcr.io/shakedaskayo/ciab-gemini:latest"
api_key_env = "GOOGLE_API_KEY"
```

## Required Environment

| Variable | Description |
|----------|-------------|
| `GOOGLE_API_KEY` | Google AI API key |

## Container Image

Based on `node:22-slim` with:

- Node.js 22 LTS
- Gemini CLI (installed via npm)
- Git, curl, ca-certificates, openssh-client

## Example

```bash
ciab sandbox create \
  --provider gemini \
  --env GOOGLE_API_KEY=$GOOGLE_API_KEY

ciab agent chat --sandbox-id <id> -m "Explain this code" --stream
```
