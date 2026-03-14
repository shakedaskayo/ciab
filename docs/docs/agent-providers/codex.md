# Codex

The Codex provider runs [OpenAI Codex CLI](https://github.com/openai/codex) inside a sandboxed container.

## Configuration

```toml
[agents.providers.codex]
enabled = true
image = "ghcr.io/shakedaskayo/ciab-codex:latest"
api_key_env = "OPENAI_API_KEY"
```

## Required Environment

| Variable | Description |
|----------|-------------|
| `OPENAI_API_KEY` | OpenAI API key |

## Container Image

Based on `node:22-slim` with:

- Node.js 22 LTS
- `@openai/codex` (installed globally via npm)
- Git, curl, ca-certificates, openssh-client

## Example

```bash
ciab sandbox create \
  --provider codex \
  --env OPENAI_API_KEY=$OPENAI_API_KEY

ciab agent chat --sandbox-id <id> -m "Implement a REST API" --stream
```
