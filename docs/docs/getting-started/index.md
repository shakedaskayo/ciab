# Getting Started

CIAB (Claude In A Box) is a platform for managing coding agent instances inside secure, sandboxed containers. It provides a REST API and CLI for creating sandboxes, chatting with agents, executing commands, and managing files — all with real-time streaming output.

## What You Can Do

- **Create sandboxes** — Spin up isolated containers with a coding agent (Claude Code, Codex, Gemini CLI, or Cursor) pre-installed
- **Chat with agents** — Send prompts and receive streaming responses, including tool use visualization
- **Execute commands** — Run shell commands inside sandboxes with streaming stdout/stderr
- **Manage files** — Upload, download, list, and delete files in any sandbox
- **Monitor resources** — Track CPU, memory, disk, and network usage per sandbox
- **Manage credentials** — Store encrypted API keys and OAuth tokens for agent access

## Prerequisites

- **Rust 1.75+** — For building the CIAB server and CLI
- **SQLite** — For the credential store and session persistence
- **OpenSandbox** — A running OpenSandbox instance for container management
- **Agent API keys** — At least one of: `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, `GOOGLE_API_KEY`, or `CURSOR_API_KEY`

## Next Steps

1. [Install CIAB](installation.md) — Build from source
2. [Quickstart](quickstart.md) — Create your first sandbox in 5 minutes
