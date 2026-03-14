# OpenSandbox Setup

CIAB uses OpenSandbox as its container runtime for managing agent sandboxes.

## What is OpenSandbox?

OpenSandbox provides a REST API for creating, managing, and executing commands in isolated containers. CIAB uses it to:

- Create containers with specific images and resource limits
- Start/stop/pause/resume containers
- Execute commands via the execd API
- Manage files within containers
- Monitor resource usage

## Installation

Refer to the OpenSandbox documentation for installation. CIAB requires OpenSandbox to be accessible via HTTP.

## Configuration

Set the OpenSandbox URL in `config.toml`:

```toml
[runtime]
opensandbox_url = "http://localhost:8000"
# opensandbox_api_key = "your-api-key"  # If authentication is enabled
```

## Container Images

CIAB provides pre-built container images for each agent provider:

| Image | Agent |
|-------|-------|
| `ghcr.io/shakedaskayo/ciab-claude:latest` | Claude Code |
| `ghcr.io/shakedaskayo/ciab-codex:latest` | Codex |
| `ghcr.io/shakedaskayo/ciab-gemini:latest` | Gemini CLI |
| `ghcr.io/shakedaskayo/ciab-cursor:latest` | Cursor |

Build them from the `images/` directory:

```bash
docker build -t ciab-claude:latest images/claude-sandbox/
docker build -t ciab-codex:latest images/codex-sandbox/
docker build -t ciab-gemini:latest images/gemini-sandbox/
docker build -t ciab-cursor:latest images/cursor-sandbox/
```

## Network Requirements

- CIAB server must reach OpenSandbox API (default port 8000)
- OpenSandbox must have access to a container runtime (Docker socket or containerd)
- Agent containers need internet access for API calls (configurable via network policies)
