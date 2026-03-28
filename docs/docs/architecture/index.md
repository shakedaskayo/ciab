# Architecture Overview

CIAB is a layered Rust workspace for managing multiple coding agents in isolated sandboxes — local processes or containers — through a unified control plane.

## System Diagram

<p align="center">
  <img src="../assets/architecture.svg" alt="CIAB Architecture" width="100%">
</p>

## How It Fits Together

**Clients** (CLI, REST API, desktop app, Slack/WhatsApp channels) connect to the **CIAB control plane** — an Axum-based server that handles authentication, request routing, and event streaming.

When a sandbox is created, the control plane runs an **11-step provisioning pipeline**: validate the spec, prepare the runtime image, resolve credentials, create and start the sandbox, mount local directories, inject credentials, clone repositories, set up the agent filesystem, run setup scripts, and start the agent. Every step is streamed over **SSE** in real time.

Each sandbox runs an isolated coding agent (Claude Code, Codex, Gemini CLI, or Cursor) in its own workspace with its own env vars, credentials, and repos. The control plane communicates with sandboxes through the **runtime backend** (local process, Docker, or OpenSandbox).

## Layered Architecture

```
┌─────────────────────────────────────────────────────────┐
│  ciab-cli          ciab-api         desktop              │  User-facing
├─────────────────────────────────────────────────────────┤
│  ciab-provisioning  ciab-gateway    ciab-channels        │  Orchestration
├─────────────────────────────────────────────────────────┤
│  ciab-agent-*       ciab-credentials                     │  Providers & Security
├─────────────────────────────────────────────────────────┤
│  ciab-sandbox       ciab-streaming  ciab-db              │  Infrastructure
├─────────────────────────────────────────────────────────┤
│  ciab-core                                               │  Foundation
└─────────────────────────────────────────────────────────┘
```

Each layer depends only on layers below it. `ciab-core` has no internal dependencies and defines all shared types, traits, and errors.

## Request Flow

1. Client sends HTTP request to the API server
2. Auth middleware validates API key / token (if configured)
3. Request handler deserializes and routes the request
4. For sandbox creation: the **provisioning pipeline** executes 11 steps, streaming each over SSE
5. For agent chat: the message is forwarded to the **agent provider** via the sandbox runtime
6. Events are published to the **SSE broker** for real-time streaming back to clients
7. State changes are persisted to **SQLite**
8. Response is returned to the client

## Key Design Decisions

- **Agent-agnostic**: The `AgentProvider` trait abstracts away agent-specific behavior. Adding a new agent requires implementing one trait.
- **Runtime-flexible**: Local process is the default — no Docker needed. OpenSandbox, Kubernetes, and EC2 are pluggable backends for stronger isolation.
- **Streaming-first**: All long-running operations emit `StreamEvent`s via SSE, not just polling endpoints.
- **Encrypted credentials**: API keys and OAuth tokens are encrypted at rest using AES-256-GCM.
- **Remote access**: The gateway layer supports tunneling (bore, Cloudflare, ngrok, frp) and LAN discovery (mDNS) for remote sandbox access.
- **Channel integration**: Slack, WhatsApp, and webhook adapters route conversations to sandboxes with per-sender session tracking.
