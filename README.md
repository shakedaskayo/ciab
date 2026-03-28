<p align="center">
  <img src="https://raw.githubusercontent.com/shakedaskayo/ciab/main/docs/docs/assets/logo.png" alt="CIAB" width="220">
</p>

<h3 align="center">Run, orchestrate, and stream coding agents — from a single control plane.</h3>

<p align="center">
  <a href="https://github.com/shakedaskayo/ciab/actions/workflows/ci.yml"><img src="https://github.com/shakedaskayo/ciab/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://github.com/shakedaskayo/ciab/releases/latest"><img src="https://img.shields.io/github/v/release/shakedaskayo/ciab?include_prereleases&label=release&color=C4693D" alt="Release"></a>
  <a href="https://crates.io/crates/ciab"><img src="https://img.shields.io/crates/v/ciab?color=C4693D" alt="crates.io"></a>
  <a href="https://github.com/shakedaskayo/ciab/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-5B8CA8.svg" alt="License"></a>
  <a href="https://shakedaskayo.github.io/ciab"><img src="https://img.shields.io/badge/docs-shakedaskayo.github.io%2Fciab-C4693D" alt="Docs"></a>
  <a href="https://github.com/shakedaskayo/ciab/stargazers"><img src="https://img.shields.io/github/stars/shakedaskayo/ciab?style=social" alt="GitHub Stars"></a>
</p>

<br>

CIAB (**Claude-In-A-Box**) is an open-source control plane for coding agents. Spin up **Claude Code**, **Codex**, **Gemini CLI**, or **Cursor** in isolated sandboxes — as local processes, containers, Kubernetes pods, or EC2 instances — and manage them through a unified **REST API**, **CLI**, **desktop app**, or **any web browser**.

Every sandbox gets its own workspace, credentials, repos, and real-time streaming output. Access them from your laptop, phone, tablet, or CI pipeline — from the same network or anywhere in the world via built-in tunneling.

```bash
curl -fsSL https://raw.githubusercontent.com/shakedaskayo/ciab/main/install.sh | bash
```

<br>

## Quick Start

```bash
# Install
curl -fsSL https://raw.githubusercontent.com/shakedaskayo/ciab/main/install.sh | bash

# Start the server (zero config — embedded defaults just work)
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

## Screenshots

### Dashboard

Manage all your sandboxes from a single view. Quick-launch any agent provider, monitor running/paused/failed states, and create new sandboxes in seconds.

<p align="center">
  <img src="docs/docs/assets/screenshots/dashboard.png" alt="CIAB Dashboard" width="100%">
</p>

### Chat — Real-time Streaming

Full conversation interface with live SSE streaming. Watch tool calls execute, file results appear, and code stream in — all in real time. Permission modes (Auto, Safe, Strict, Plan) control what the agent can do.

<p align="center">
  <img src="docs/docs/assets/screenshots/chat.png" alt="CIAB Chat with Streaming" width="100%">
</p>

### Access From Anywhere — Phone, Tablet, Any Browser

CIAB's built-in **web gateway** makes every sandbox accessible from **any device**. Open the Gateway page, scan the QR code with your phone, and you're chatting with your agent — with full streaming. No app install needed.

<p align="center">
  <img src="docs/docs/assets/screenshots/gateway.png" alt="CIAB Gateway — Mobile Access" width="100%">
</p>

### Skills Catalog

Browse and install agent skills from the [skills.sh](https://skills.sh) open registry. Add skills to workspaces to give agents specialized knowledge — React best practices, security guidelines, API patterns, and more.

<p align="center">
  <img src="docs/docs/assets/screenshots/skills.png" alt="CIAB Skills Catalog" width="100%">
</p>

<br>

## Features

| | |
|---|---|
| **Multi-agent** | Run Claude Code, Codex, Gemini CLI, and Cursor side-by-side. Switch providers with one config change. |
| **Isolated sandboxes** | Each agent gets its own workspace, env vars, credentials, and mounted repos. Local processes, containers, Kubernetes pods, or EC2 instances. |
| **Real-time streaming** | Watch agent output as it happens — text deltas, tool use, provisioning steps, logs — all over SSE. |
| **Access anywhere** | Open sandboxes from your phone, tablet, laptop, or CI pipeline. Built-in web gateway with QR codes, mDNS, and tunneling. |
| **Workspaces** | Reusable, TOML-configurable environment definitions. Bundle repos, skills, pre-commands, binaries, and agent config. |
| **Skills catalog** | Install agent skills from the skills.sh open registry — React, security, Docker, and more. |
| **Encrypted credentials** | AES-256-GCM vault with OAuth2 support. Credentials injected at provisioning time, never stored in plaintext. |
| **Remote tunnels** | One-click expose via bore, Cloudflare Tunnel, ngrok, or frp. Token-scoped auth with LAN discovery. |
| **Channels** | Pipe agent conversations through Slack, WhatsApp, or webhooks. Per-sender session tracking. |
| **Rust library** | Embed CIAB in any Rust app with `CiabEngine::builder().build()`. Feature-gated crate — pick only the runtimes you need. |
| **Cloud provisioning** | Build AMIs with Packer, launch EC2 instances, manage remote sandboxes. Templates from git repos, URLs, or built-in defaults. |
| **Desktop + Web** | Tauri + React desktop app and responsive web UI — same interface on any device. |

<br>

## How It Works

<p align="center">
  <img src="docs/docs/assets/architecture.svg" alt="CIAB Architecture" width="100%">
</p>

**Clients** (CLI, REST API, desktop app, mobile browser, Slack/WhatsApp) connect to the **CIAB control plane** — an Axum-based server that handles auth, streaming, and orchestration. The control plane provisions sandboxes through an **11-step pipeline** (validate → prepare image → resolve credentials → create → start → mount local dirs → inject credentials → clone repos → setup agent filesystem → run scripts → start agent), streaming every step over **SSE** in real time.

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

| Method | How | Use case |
|--------|-----|----------|
| **LAN / mDNS** | `http://ciab.local.local:9090` or scan QR code | Same WiFi — phone, tablet, another laptop |
| **Tunnel** (bore, Cloudflare, ngrok, frp) | One config line, auto-installs | Access from anywhere in the world |
| **REST API** | `curl`, scripts, CI/CD pipelines | Programmatic access, automation |

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

## Rust Library

Embed CIAB natively in any Rust application. The `ciab` crate provides `CiabEngine` — a single entry point that owns the database, runtimes, streaming broker, and credential store.

```rust
use ciab::{CiabEngine, SandboxSpec};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Zero config — just works with local runtime
    let engine = CiabEngine::builder().build().await?;

    // Create a sandbox
    let spec: SandboxSpec = serde_json::from_value(serde_json::json!({
        "agent_provider": "claude-code",
        "env_vars": {
            "ANTHROPIC_API_KEY": std::env::var("ANTHROPIC_API_KEY")?
        }
    }))?;
    let sandbox = engine.create_sandbox(&spec).await?;

    // Execute a command
    let req: ciab::ExecRequest = serde_json::from_value(serde_json::json!({
        "command": ["cargo", "test"]
    }))?;
    let result = engine.exec(&sandbox.id, &req).await?;
    println!("exit={} stdout={}", result.exit_code, result.stdout);

    // File operations
    let files = engine.list_files(&sandbox.id, "/workspace").await?;
    engine.write_file(&sandbox.id, "/workspace/hello.txt", b"world").await?;

    // Clean up
    engine.terminate_sandbox(&sandbox.id).await?;
    Ok(())
}
```

```toml
# Cargo.toml — pick only what you need
[dependencies]
ciab = "0.1"                                          # local runtime (default)
ciab = { version = "0.1", features = ["ec2"] }        # + AWS EC2
ciab = { version = "0.1", features = ["kubernetes"] } # + Kubernetes
ciab = { version = "0.1", features = ["full"] }       # everything
```

See the full [Rust Library reference](https://shakedaskayo.github.io/ciab/architecture/rust-library/) for the builder API, config resolution chain, feature flags, and re-exports.

<br>

## Cloud Provisioning

Build machine images and run sandboxes on AWS EC2:

```bash
# Build an AMI with Claude Code pre-installed
ciab image build --agent claude-code \
  --var region=us-east-1 \
  --var base_ami=ami-0abcdef1234567890

# Or use a custom Packer template from a git repo
ciab image build \
  --template "git::https://github.com/org/templates.git//agent.pkr.hcl?ref=main" \
  --var region=eu-west-1

# Configure EC2 as the default runtime
# config.toml:
# [runtime]
# backend = "ec2"
# [runtime.ec2]
# region = "us-east-1"
# default_ami = "ami-your-built-image"
```

<br>

## Architecture

```
crates/
  ciab                  Rust library facade (CiabEngine, feature-gated re-exports)
  ciab-core             Types, traits, errors (foundation)
  ciab-api              Axum REST API — 15 route groups
  ciab-sandbox          Runtime backends: local process, Docker, OpenSandbox
  ciab-sandbox-k8s      Kubernetes runtime backend (Kata Containers support)
  ciab-sandbox-ec2      AWS EC2 runtime backend (ephemeral instances, SSH exec)
  ciab-streaming        SSE broker with event buffer and replay
  ciab-provisioning     11-step sandbox provisioning pipeline
  ciab-credentials      AES-256-GCM encrypted vault, OAuth2
  ciab-packer           HashiCorp Packer image builder (AMI builds)
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

**Pre-built binaries:**

| Platform | Download |
|----------|----------|
| macOS (Apple Silicon) | [`ciab-darwin-arm64.tar.gz`](https://github.com/shakedaskayo/ciab/releases/latest) |
| macOS (Intel) | [`ciab-darwin-x64.tar.gz`](https://github.com/shakedaskayo/ciab/releases/latest) |
| Linux (x86_64) | [`ciab-linux-x64.tar.gz`](https://github.com/shakedaskayo/ciab/releases/latest) |
| Linux (ARM64) | [`ciab-linux-arm64.tar.gz`](https://github.com/shakedaskayo/ciab/releases/latest) |
| Desktop (macOS) | [`CIAB.dmg`](https://github.com/shakedaskayo/ciab/releases/latest) |

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

## Community

- [GitHub Discussions](https://github.com/shakedaskayo/ciab/discussions) — questions, ideas, show & tell
- [Issues](https://github.com/shakedaskayo/ciab/issues) — bugs and feature requests
- [Contributing](CONTRIBUTING.md) — how to contribute

If CIAB is useful to you, a star on GitHub helps others find it.

<br>

## License

[MIT](LICENSE)
