# Getting Started

CIAB (Claude In A Box) is an open-source control plane for coding agents. Manage **Claude Code**, **Codex**, **Gemini CLI**, and **Cursor** as isolated sandboxes — via CLI, REST API, desktop app, or any web browser.

## Install in 10 Seconds

```bash
curl -fsSL https://raw.githubusercontent.com/shakedaskayo/ciab/main/install.sh | bash
```

This downloads the pre-built binary for your platform (macOS or Linux) and installs it to `/usr/local/bin`. See [Installation](installation.md) for other options.

## What You Can Do

- **Create sandboxes** — Spin up isolated agents (Claude Code, Codex, Gemini CLI, or Cursor) as local processes or containers
- **Chat with agents** — Send prompts and receive streaming responses, including tool use visualization
- **Access from any device** — Open sandboxes from your phone, tablet, or any browser via the built-in web gateway
- **Execute commands** — Run shell commands inside sandboxes with streaming stdout/stderr
- **Manage files** — Upload, download, list, and delete files in any sandbox
- **Manage credentials** — Store encrypted API keys and OAuth tokens
- **Remote access** — Expose sandboxes via bore, Cloudflare Tunnel, ngrok, or frp

## Prerequisites

- **macOS or Linux** — For running the CIAB server
- **Agent API key** — At least one of: `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, or `GOOGLE_API_KEY`

That's it. No Docker, no Rust toolchain, no containers required — CIAB runs agents as local processes by default.

!!! tip "Want containers?"
    For Docker or OpenSandbox backends, see [Deployment](../deployment/index.md).

## Next Steps

1. [Installation](installation.md) — Install via script, binary download, or build from source
2. [Quickstart](quickstart.md) — Create your first sandbox in 5 minutes
3. [Mobile Access](../deployment/mobile-access.md) — Access sandboxes from your phone
