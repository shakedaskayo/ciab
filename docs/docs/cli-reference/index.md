# CLI Reference

The `ciab` CLI provides interactive access to all CIAB functionality.

## Global Options

| Flag | Env Var | Default | Description |
|------|---------|---------|-------------|
| `--server-url` | `CIAB_SERVER_URL` | `http://localhost:8080` | CIAB server URL |
| `--api-key` | `CIAB_API_KEY` | — | API key for authentication |
| `--output` | — | `text` | Output format: `text`, `json` |
| `--verbose` | — | — | Enable verbose output |

## Commands

| Command | Description |
|---------|-------------|
| [`sandbox`](sandbox.md) | Sandbox lifecycle management |
| [`agent`](agent.md) | Agent interaction (chat, attach) |
| [`session`](session.md) | Session management |
| [`files`](files.md) | File operations |
| [`credential`](credential.md) | Credential management |
| [`oauth`](oauth.md) | OAuth flows |
| [`config`](config.md) | Configuration management |
| [`server`](server.md) | API server commands |

## Examples

```bash
# Create and chat with a sandbox
ciab sandbox create --provider claude-code -e ANTHROPIC_API_KEY=$ANTHROPIC_API_KEY
ciab agent chat --sandbox-id <id> --interactive --stream

# Quick command execution
ciab sandbox exec <id> -- cargo test

# File management
ciab files list <id> --path /workspace
ciab files download <id> --path /workspace/output.txt
```
