# Show HN Draft

## Title
Show HN: CIAB – Run Claude Code, Codex, Gemini CLI, and Cursor from one control plane

## Body

Hi HN,

I built CIAB (Claude-In-A-Box) — an open-source control plane for coding agents.

**The problem:** I kept spinning up Claude Code, Codex, and Gemini CLI manually in separate terminals, juggling credentials, context, and repos across each. There was no unified way to manage them, stream their output, or reach them from my phone while away from my desk.

**What CIAB does:**
- Runs any supported agent (Claude Code, Codex, Gemini, Cursor) in an isolated sandbox — as a local process or Docker container
- Unified REST API + CLI + desktop app (Tauri) + responsive web UI
- Real-time SSE streaming: watch tool calls, file changes, and text deltas as they happen
- Built-in web gateway with QR codes + optional tunneling (bore, Cloudflare, ngrok) so you can chat with your agent from your phone
- Workspaces: TOML-configurable environment definitions bundling repos, skills, pre-commands, credentials, and agent config — reproducible and CI-friendly
- Encrypted credential vault (AES-256-GCM) with OAuth2 support
- Skills catalog via skills.sh — install prompt libraries into sandboxes at launch time

**Why Rust?** The core is latency-sensitive (streaming, provisioning pipeline, SSE broker) and I wanted a single binary with no runtime. The async story (tokio + axum) felt right for this workload.

**No Docker required** — the default runtime runs agents as local processes. Docker and OpenSandbox are opt-in.

Repo: https://github.com/shakedaskayo/ciab
Docs: https://shakedaskayo.github.io/ciab

Happy to answer questions about the architecture, the streaming pipeline, or the multi-agent sandboxing approach.

---

## Submission checklist
- [ ] Post on a weekday between 9–11am ET (peak HN traffic)
- [ ] Have the GitHub repo and docs polished before posting
- [ ] Be ready to respond to comments within the first 2 hours (critical for ranking)
- [ ] Cross-post to r/rust and r/LocalLLaMA the same day
