# Quickstart

Create your first coding agent sandbox in 5 minutes.

## 1. Start the Server

```bash
ciab server start --config config.toml --port 8080
```

The API is now available at `http://localhost:8080`.

!!! tip "Health Check"
    Verify the server is running: `curl http://localhost:8080/health`

## 2. Create a Sandbox

```bash
ciab sandbox create \
  --provider claude-code \
  --name my-first-sandbox \
  --env ANTHROPIC_API_KEY=$ANTHROPIC_API_KEY
```

This will:

1. Validate the sandbox specification
2. Pull the Claude Code container image
3. Resolve and inject credentials
4. Start the sandbox and the agent process

You'll see provisioning progress streamed to your terminal.

## 3. Chat with the Agent

```bash
# Single message
ciab agent chat --sandbox-id <id> --message "Explain the project structure" --stream

# Interactive mode
ciab agent chat --sandbox-id <id> --interactive --stream
```

The `--stream` flag shows the agent's response as it's generated, including tool use.

## 4. Execute Commands

```bash
# Run a command in the sandbox
ciab sandbox exec <id> -- ls -la /workspace

# Check installed tools
ciab sandbox exec <id> -- node --version
```

## 5. Browse Files

```bash
# List files
ciab files list <id> --path /workspace

# Download a file
ciab files download <id> --path /workspace/README.md --output ./README.md

# Upload a file
ciab files upload <id> --path /workspace/data.json --input ./data.json
```

## 6. Monitor Resources

```bash
ciab sandbox stats <id>
```

Output shows CPU usage, memory, disk, and network statistics.

## 7. Clean Up

```bash
# Stop the sandbox
ciab sandbox stop <id>

# Delete it entirely
ciab sandbox delete <id>
```

## Using the REST API

All CLI operations are available via the REST API:

```bash
# Create a sandbox
curl -X POST http://localhost:8080/api/v1/sandboxes \
  -H "Content-Type: application/json" \
  -d '{
    "agent_provider": "claude-code",
    "name": "api-sandbox",
    "env_vars": {
      "ANTHROPIC_API_KEY": "sk-ant-..."
    }
  }'

# Send a message
curl -X POST http://localhost:8080/api/v1/sessions/<sid>/messages \
  -H "Content-Type: application/json" \
  -d '{"role": "user", "content": [{"type": "text", "text": "Hello!"}]}'

# Stream events (SSE)
curl -N http://localhost:8080/api/v1/sandboxes/<id>/stream
```

## Next Steps

- [Architecture](../architecture/index.md) — Understand how CIAB works
- [API Reference](../api-reference/index.md) — Full endpoint documentation
- [CLI Reference](../cli-reference/index.md) — All CLI commands
- [Configuration](../configuration/index.md) — Customize your setup
