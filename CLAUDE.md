# CIAB — Claude-In-A-Box

Rust workspace managing coding agent instances (Claude Code, Codex, Gemini CLI, Cursor CLI) as local processes or inside containers.

## Build

```bash
cargo build --workspace
cargo test --workspace
cd desktop && npm install && npm run tauri dev   # Desktop app
cd docs && pip install -r requirements.txt && mkdocs serve  # Docs site
```

## Architecture

- `ciab-core` — Types, traits, errors (foundation)
- `ciab-db` — SQLite persistence (sqlx)
- `ciab-streaming` — SSE broker, event buffer, WebSocket
- `ciab-sandbox` — Runtime backends: LocalProcessRuntime + OpenSandboxRuntime
- `ciab-agent-{claude,codex,gemini,cursor}` — Agent provider implementations
- `ciab-credentials` — Encrypted credential store, OAuth2
- `ciab-provisioning` — 9-step sandbox provisioning pipeline
- `ciab-api` — axum REST API server
- `ciab-cli` — CLI binary (`ciab`)
- `desktop/` — Tauri v2 + React desktop app
- `docs/` — MkDocs Material documentation site

## Runtime Backends

CIAB supports multiple runtime backends configured in `config.toml`:

- **`local`** (default) — Run agents as local processes, no Docker needed
- **`docker`** — Run agents in Docker containers
- **`opensandbox`** — Run agents in OpenSandbox containers

## Workspaces

Workspaces are reusable environment definitions that bundle repos, skills, pre-commands, binaries, filesystem settings, agent config, subagents, and credentials. Fully TOML-configurable for CI.

```bash
# Import workspace from TOML
ciab workspace import workspace.toml

# Launch a sandbox from workspace
ciab workspace launch <workspace-id>
```

See `workspace.example.toml` for a complete example.

## Running

```bash
# Start server (local runtime, no Docker needed)
ciab server start --config config.toml

# Create a sandbox directly
ciab sandbox create --provider claude-code --env ANTHROPIC_API_KEY=$ANTHROPIC_API_KEY

# Or use a workspace
ciab workspace create --name my-project --provider claude-code
ciab workspace launch <workspace-id>

# Chat with agent
ciab agent chat <sandbox-id> --prompt "Explain the codebase" --stream
```

## Config

Copy `config.example.toml` to `config.toml` and adjust settings.
