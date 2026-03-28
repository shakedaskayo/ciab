# Environment Variables

CIAB uses environment variables for secrets and CLI configuration.

## CLI Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `CIAB_SERVER_URL` | CIAB server URL | `http://localhost:9090` |
| `CIAB_API_KEY` | API key for authentication | — |

## Server Variables

| Variable | Description |
|----------|-------------|
| `CIAB_ENCRYPTION_KEY` | AES-256 key for credential encryption (required) |
| `OPENSANDBOX_API_KEY` | OpenSandbox API key (optional) |

## Agent Provider API Keys

| Variable | Provider | Description |
|----------|----------|-------------|
| `ANTHROPIC_API_KEY` | Claude Code | Anthropic API key |
| `OPENAI_API_KEY` | Codex | OpenAI API key |
| `GOOGLE_API_KEY` | Gemini | Google AI API key |
| `CURSOR_API_KEY` | Cursor | Cursor API key |

## OAuth Variables

| Variable | Description |
|----------|-------------|
| `GITHUB_CLIENT_ID` | GitHub OAuth app client ID |
| `GITHUB_CLIENT_SECRET` | GitHub OAuth app client secret |

## Generating an Encryption Key

```bash
# Generate a random 32-byte key (base64 encoded)
openssl rand -base64 32

# Set it
export CIAB_ENCRYPTION_KEY="$(openssl rand -base64 32)"
```
