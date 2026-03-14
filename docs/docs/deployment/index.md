# Deployment

CIAB can be deployed as a standalone binary or via Docker.

## Deployment Options

| Method | Best For |
|--------|----------|
| [Docker](docker.md) | Production deployments, CI/CD |
| [Binary](../getting-started/installation.md) | Development, simple setups |

## Requirements

- **OpenSandbox** — A running OpenSandbox instance (see [OpenSandbox Setup](opensandbox.md))
- **SQLite** — For credential storage and session persistence
- **Network** — CIAB server needs network access to OpenSandbox API
- **API Keys** — At least one agent provider's API key

## Architecture

```
┌──────────────┐     ┌──────────────┐     ┌──────────────────┐
│  CIAB Server │────▶│  OpenSandbox │────▶│  Agent Containers │
│  (port 8080) │     │  (port 8000) │     │  (dynamic ports)  │
└──────────────┘     └──────────────┘     └──────────────────┘
```

For production hardening, see [Production](production.md).
