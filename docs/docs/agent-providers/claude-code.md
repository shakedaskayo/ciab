# Claude Code

The Claude Code provider runs [Claude Code](https://docs.anthropic.com/en/docs/claude-code) inside a sandboxed container.

## Configuration

```toml
[agents.providers.claude-code]
enabled = true
image = "ghcr.io/shakedaskayo/ciab-claude:latest"
default_model = "claude-sonnet-4-20250514"
api_key_env = "ANTHROPIC_API_KEY"
```

## Required Environment

| Variable | Description |
|----------|-------------|
| `ANTHROPIC_API_KEY` | Anthropic API key |

## Container Image

The Claude Code container image is based on `node:22-slim` and includes:

- Node.js 22 LTS
- `@anthropic-ai/claude-code@latest` (installed globally via npm)
- Git, curl, ca-certificates, openssh-client

## Agent Start Command

Claude Code runs in headless mode with JSON streaming output:

```bash
claude-code --headless --output-format stream-json
```

## Models

Claude Code supports all Claude models. Configure via `agent_config.model`:

- `claude-sonnet-4-20250514` (default)
- `claude-opus-4-20250514`
- `claude-haiku-4-5-20251001`

## Example

```bash
ciab sandbox create \
  --provider claude-code \
  --name my-project \
  --env ANTHROPIC_API_KEY=$ANTHROPIC_API_KEY \
  --git-repo https://github.com/user/repo.git

ciab agent chat --sandbox-id <id> -m "Review the auth module" --stream
```
