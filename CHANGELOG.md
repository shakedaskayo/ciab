# Changelog

All notable changes to CIAB are documented here.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

---

## [Unreleased]

## [0.2.0] - 2026-03-16

### Added
- Workspace system: reusable TOML-configurable environment definitions
- Workspace CLI commands: `ciab workspace import/launch/list/get/delete/export`
- Workspace REST API routes (CRUD + launch + import/export)
- Workspace desktop pages: list, detail, create dialog
- Desktop gateway page with QR code for mobile access
- Skills catalog integration with skills.sh open registry
- Remote tunneling support: bore, Cloudflare Tunnel, ngrok, frp
- Channels: Slack, WhatsApp, and webhook adapters for agent conversations
- 9-step sandbox provisioning pipeline with SSE progress streaming
- AES-256-GCM encrypted credential vault with OAuth2 support
- OpenSandbox runtime backend
- Kubernetes runtime backend with Kata Containers support (`ciab-sandbox-k8s`)
- Helm chart for deploying CIAB on Kubernetes (`helm/ciab/`)
- LLM provider management: configure and switch between API providers from the desktop app
- Session management API routes
- Launch override dialog for workspaces
- New session dialog in chat
- LLM provider icons and model picker in desktop app
- Community section in README, CONTRIBUTING guide, SECURITY policy, issue/PR templates
- MkDocs Material documentation site

## [0.1.0] - 2026-03-01

### Added
- Initial release
- Multi-agent support: Claude Code, Codex, Gemini CLI, Cursor
- Local process and Docker runtime backends
- Axum REST API server with SSE streaming
- SQLite persistence via sqlx
- CLI (`ciab`) with sandbox, agent, session, files, credential, config, and server commands
- Tauri v2 + React desktop app
- Web UI served from the API server
- LAN/mDNS discovery (`ciab.local`)
- One-liner install script
- Pre-built binaries for macOS (arm64/x64) and Linux (x86_64/arm64)
- macOS DMG desktop installer
