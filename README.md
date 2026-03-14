<p align="center">
  <img src="docs/docs/assets/logo.png" alt="CIAB" width="220">
</p>

<p align="center">
  <b>Run, orchestrate, and stream coding agents — from a single control plane.</b>
</p>

<p align="center">
  <a href="https://github.com/shakedaskayo/ciab/actions/workflows/ci.yml"><img src="https://github.com/shakedaskayo/ciab/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://github.com/shakedaskayo/ciab/releases/latest"><img src="https://img.shields.io/github/v/release/shakedaskayo/ciab?label=release&color=C4693D" alt="Release"></a>
  <a href="https://github.com/shakedaskayo/ciab/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-5B8CA8.svg" alt="License"></a>
  <a href="https://shakedaskayo.github.io/ciab"><img src="https://img.shields.io/badge/docs-shakedaskayo.github.io%2Fciab-C4693D" alt="Docs"></a>
</p>

<br>

CIAB (**Claude-In-A-Box**) is an open-source control plane for coding agents. Spin up **Claude Code**, **Codex**, **Gemini CLI**, or **Cursor** in isolated sandboxes — as local processes or containers — and manage them through a unified **REST API**, **CLI**, **desktop app**, or **any web browser**.

Every sandbox gets its own workspace, credentials, repos, and real-time streaming output. Access them from your laptop, phone, tablet, or CI pipeline — from the same network or anywhere in the world via built-in tunneling.

```bash
curl -fsSL https://raw.githubusercontent.com/shakedaskayo/ciab/main/install.sh | bash
```

<br>

## Dashboard

Manage all your sandboxes from a single view. Quick-launch any agent provider, monitor running/paused/failed states, and create new sandboxes in seconds.

<p align="center">
  <img src="docs/docs/assets/screenshots/dashboard.png" alt="CIAB Dashboard" width="100%">
</p>

<br>

## Chat with Agents — Real-time Streaming

Full conversation interface with live SSE streaming. Watch tool calls execute, file system results appear, and code stream in — all in real time. Permission modes (Auto, Safe, Strict, Plan) control what the agent can do.

<p align="center">
  <img src="docs/docs/assets/screenshots/chat.png" alt="CIAB Chat with Streaming" width="100%">
</p>

<br>

## Access From Anywhere — Phone, Tablet, Any Browser

CIAB's built-in **web gateway** makes every sandbox accessible from **any device**. Open the Gateway page, scan the QR code with your iPhone or iPad, and you're chatting with your agent from your phone — with full streaming support. No app install needed.

<p align="center">
  <img src="docs/docs/assets/screenshots/gateway.png" alt="CIAB Gateway — Mobile Access" width="100%">
</p>

**Three ways to access your sandboxes remotely:**

| Method | How | Use case |
|--------|-----|----------|
| **LAN / mDNS** | Open `http://ciab.local.local:9090` or scan QR code | Same WiFi — phone, tablet, another laptop |
| **Tunnel** (bore, Cloudflare, ngrok, frp) | One config line, auto-installs | Access from anywhere in the world |
| **REST API** | `curl`, scripts, CI/CD pipelines | Programmatic access, automation |

Every access method gets the same full experience: real-time streaming chat, tool use visualization, file browsing, terminal access, and session management.

<br>

## Skills Catalog

Browse and install agent skills from the [skills.sh](https://skills.sh) open registry. Add skills to workspaces to give agents specialized knowledge — React best practices, security guidelines, API patterns, and more.

<p align="center">
  <img src="docs/docs/assets/screenshots/skills.png" alt="CIAB Skills Catalog" width="100%">
</p>

<br>

## How It Works

<p align="center">
  <img src="docs/docs/assets/architecture.svg" alt="CIAB Architecture" width="100%">
</p>

**Clients** (CLI, REST API, desktop app, mobile browser, Slack/WhatsApp) connect to the **CIAB control plane** — an Axum-based server that handles auth, streaming, and orchestration. The control plane provisions sandboxes through a **9-step pipeline** (validate → prepare → create → start → mount → inject credentials → clone repos → run scripts → launch agent), streaming every step over **SSE** in real time.

<br>

## Quick Start

```bash
# Install
curl -fsSL https://raw.githubusercontent.com/shakedaskayo/ciab/main/install.sh | bash

# Initialize and start
ciab config init
ciab server start

# Create a sandbox with Claude Code
ciab sandbox create --provider claude-code \
  --env ANTHROPIC_API_KEY=$ANTHROPIC_API_KEY

# Stream a conversation
ciab agent chat <sandbox-id> --message "Refactor the auth module" --stream

# Access from your phone — open the Gateway page and scan the QR code
# Or navigate to http://<your-ip>:9090 from any browser
```

<br>

## Why CIAB?

| | What you get |
|---|---|
| **Multi-agent** | Run Claude Code, Codex, Gemini CLI, and Cursor side-by-side. Switch providers with one config change. |
| **Isolated sandboxes** | Each agent gets its own workspace, env vars, credentials, and mounted repos. Local processes or containers. |
| **Real-time streaming** | Watch agent output as it happens — text deltas, tool use, provisioning steps, logs — all over SSE. |
| **Access anywhere** | Open sandboxes from your phone, tablet, laptop, or CI pipeline. Built-in web gateway with QR codes, mDNS, and tunneling. |
| **Workspaces** | Reusable, TOML-configurable environment definitions. Bundle repos, skills, pre-commands, binaries, and agent config. |
| **Skills catalog** | Install agent skills from the skills.sh open registry — React, security, Docker, and more. |
| **Encrypted credentials** | AES-256-GCM vault with OAuth2 support. Credentials injected at provisioning time, never stored in plaintext. |
| **Remote tunnels** | One-click expose via bore, Cloudflare Tunnel, ngrok, or frp. Token-scoped auth with LAN discovery. |
| **Channels** | Pipe agent conversations through Slack, WhatsApp, or webhooks. Per-sender session tracking. |
| **Desktop + Web** | Tauri + React desktop app and responsive web UI — same interface on any device. |

<br>

## Every Way to Access Your Sandboxes

```bash
# CLI — create and chat
ciab sandbox create --provider claude-code
ciab agent chat <id> --message "Fix the bug" --stream

# REST API — programmatic control
curl -X POST http://localhost:9090/api/v1/sandboxes \
  -H "Content-Type: application/json" \
  -d '{"agent_provider": "claude-code", "env_vars": {"ANTHROPIC_API_KEY": "..."}}'

# SSE streaming — real-time events
curl -N http://localhost:9090/api/v1/sandboxes/<id>/stream

# Web browser — from any device on the network
open http://localhost:9090          # desktop
open http://ciab.local.local:9090   # mDNS (Apple devices auto-discover)
# or scan QR code from Gateway page  # phone / tablet

# Desktop app
make desktop
```

<br>

## Workspaces

Define reusable environments in TOML — repos, skills, credentials, agent config — and launch them with one command:

```toml
[workspace]
name = "my-project"
provider = "claude-code"

[[repositories]]
url = "https://github.com/org/repo.git"
branch = "main"

[[skills]]
source = "vercel-labs/ai-sdk-best-practices"

[[pre_commands]]
command = "npm install"

[agent_config]
model = "claude-sonnet-4-20250514"
system_prompt = "You are a senior engineer working on this project."

[[credentials]]
env_var = "ANTHROPIC_API_KEY"
vault_path = "anthropic/api-key"
```

```bash
ciab workspace import workspace.toml
ciab workspace launch <workspace-id>
```

<br>

## Architecture

```
crates/
  ciab-core             Types, traits, errors (foundation)
  ciab-api              Axum REST API — 15 route groups
  ciab-sandbox          Runtime backends: local process, Docker, OpenSandbox
  ciab-streaming        SSE broker with event buffer and replay
  ciab-provisioning     9-step sandbox provisioning pipeline
  ciab-credentials      AES-256-GCM encrypted vault, OAuth2
  ciab-gateway          Web gateway + tunneling (bore, Cloudflare, ngrok, frp) + LAN/mDNS
  ciab-channels         Slack, WhatsApp, webhook adapters
  ciab-agent-claude     Claude Code provider
  ciab-agent-codex      Codex provider
  ciab-agent-gemini     Gemini CLI provider
  ciab-agent-cursor     Cursor provider
  ciab-db               SQLite persistence (sqlx)
desktop/                Tauri v2 + React desktop app
docs/                   MkDocs Material documentation
```

<br>

## Install

**One-liner** (macOS & Linux):

```bash
curl -fsSL https://raw.githubusercontent.com/shakedaskayo/ciab/main/install.sh | bash
```

**From releases** — download pre-built binaries:

| Platform | Download |
|----------|----------|
| macOS (Apple Silicon) | [`ciab-darwin-arm64.tar.gz`](https://github.com/shakedaskayo/ciab/releases/latest) |
| macOS (Intel) | [`ciab-darwin-x64.tar.gz`](https://github.com/shakedaskayo/ciab/releases/latest) |
| Linux (x86_64) | [`ciab-linux-x64.tar.gz`](https://github.com/shakedaskayo/ciab/releases/latest) |
| Linux (ARM64) | [`ciab-linux-arm64.tar.gz`](https://github.com/shakedaskayo/ciab/releases/latest) |

**From source:**

```bash
git clone https://github.com/shakedaskayo/ciab.git && cd ciab
cargo build --release
sudo cp target/release/ciab /usr/local/bin/
```

<br>

## Development

```bash
make build      # Build all crates
make test       # Run tests
make lint       # Clippy + warnings-as-errors
make server     # Start API server on :9090
make desktop    # Launch desktop app
make docs       # Serve docs at localhost:8000
make dev        # Server + desktop together
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for full setup.

<br>

## License

[MIT](LICENSE)
