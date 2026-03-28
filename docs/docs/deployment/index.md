# Deployment

CIAB can be deployed as a standalone binary or via Docker.

## Deployment Options

| Method | Best For |
|--------|----------|
| [Binary](../getting-started/installation.md) | Development, simple setups |
| [Docker](docker.md) | Containerized deployments, CI/CD |
| [Kubernetes](kubernetes.md) | Production clusters, Kata Containers microVM isolation |
| [OpenSandbox](opensandbox.md) | Managed sandbox containers |

## Requirements

- **OpenSandbox** — A running OpenSandbox instance (see [OpenSandbox Setup](opensandbox.md))
- **SQLite** — For credential storage and session persistence
- **Network** — CIAB server needs network access to OpenSandbox API
- **API Keys** — At least one agent provider's API key

## Architecture

```
┌──────────────┐     ┌──────────────┐     ┌──────────────────┐
│  CIAB Server │────▶│  OpenSandbox │────▶│  Agent Containers │
│  (port 9090) │     │  (port 8000) │     │  (dynamic ports)  │
└──────────────┘     └──────────────┘     └──────────────────┘
```

For production hardening, see [Production](production.md).
