# Contributing

## Development Setup

### Prerequisites

- **Rust** (stable, latest) — [rustup.rs](https://rustup.rs)
- **Node.js 22+** — for the desktop app
- **Python 3.12+** — for the docs site (optional)

### Quick Start

```bash
git clone https://github.com/shakedaskayo/ciab.git
cd ciab

# Build everything
make build

# Run tests
make test

# Start the server locally
cp config.example.toml config.toml
make server

# Run the desktop app (in another terminal)
make desktop-install
make desktop
```

### Running the Docs Site Locally

```bash
make docs-install
make docs
# Opens at http://127.0.0.1:8000
```

## Project Structure

```
ciab/
  crates/              # Rust workspace crates
    ciab-core/         # Types, traits, errors
    ciab-db/           # SQLite persistence
    ciab-streaming/    # SSE broker, WebSocket
    ciab-sandbox/      # Runtime backends
    ciab-agent-*/      # Agent provider implementations
    ciab-credentials/  # Encrypted credential store
    ciab-provisioning/ # Sandbox provisioning pipeline
    ciab-gateway/      # Remote access tunnels
    ciab-channels/     # External messaging
    ciab-api/          # Axum REST API
    ciab-cli/          # CLI binary
  desktop/             # Tauri v2 + React desktop app
  docs/                # MkDocs Material documentation
  tests/integration/   # Integration tests
```

## Code Style

- Run `cargo fmt --all` before committing
- Run `cargo clippy --workspace` and fix all warnings
- Write tests for new functionality
- Keep crate dependencies minimal

## Pull Request Guidelines

1. Fork the repo and create a feature branch from `main`
2. Write clear commit messages
3. Add tests for new functionality
4. Ensure CI passes: `make lint && make test`
5. Update documentation if adding new API endpoints or CLI commands
6. Open a PR with a description of what changed and why

## Adding a New Agent Provider

See [Custom Provider](../agent-providers/custom-provider.md) for a step-by-step guide.

## Reporting Issues

Use [GitHub Issues](https://github.com/shakedaskayo/ciab/issues) for bug reports and feature requests. Include your OS, Rust version, and steps to reproduce.
