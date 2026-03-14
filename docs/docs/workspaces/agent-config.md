# Agent Configuration

Configure the primary coding agent for the workspace: provider, model, system prompt, tools, and MCP servers.

## Configuration

```toml
[workspace.agent]
provider = "claude-code"
model = "claude-sonnet-4-20250514"
tools_enabled = true
system_prompt = """
You are a senior developer working on this project.
Follow the coding standards and write tests for all changes.
"""

[[workspace.agent.mcp_servers]]
name = "filesystem"
command = "npx"
args = ["-y", "@anthropic-ai/mcp-server-filesystem", "/workspace"]
```

## Fields

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `provider` | string | Yes | — | Agent provider: `claude-code`, `codex`, `gemini`, `cursor` |
| `model` | string | No | provider default | Model identifier |
| `system_prompt` | string | No | — | System prompt prepended to all sessions |
| `max_tokens` | integer | No | — | Max tokens per response |
| `temperature` | float | No | — | Sampling temperature |
| `tools_enabled` | boolean | No | `true` | Whether the agent can use tools |
| `mcp_servers` | array | No | `[]` | MCP server configurations |
| `allowed_tools` | string[] | No | `[]` | Tool allowlist (empty = all) |
| `denied_tools` | string[] | No | `[]` | Tool denylist |
| `extra` | object | No | `{}` | Provider-specific settings |

## Providers

| Provider | Description |
|----------|-------------|
| `claude-code` | Anthropic's Claude Code CLI |
| `codex` | OpenAI Codex agent |
| `gemini` | Google Gemini CLI |
| `cursor` | Cursor CLI agent |

## System Prompts

System prompts give the agent context about the workspace:

```toml
[workspace.agent]
provider = "claude-code"
system_prompt = """
Project: E-commerce Platform
Stack: React frontend (/workspace/frontend), Rust backend (/workspace/backend)
Database: PostgreSQL
Standards: Use TypeScript strict mode, write unit tests, follow REST conventions.
"""
```

## MCP Servers

Connect MCP (Model Context Protocol) servers for extended capabilities:

```toml
[[workspace.agent.mcp_servers]]
name = "filesystem"
command = "npx"
args = ["-y", "@anthropic-ai/mcp-server-filesystem", "/workspace"]

[[workspace.agent.mcp_servers]]
name = "postgres"
command = "npx"
args = ["-y", "@anthropic-ai/mcp-server-postgres", "postgresql://localhost/mydb"]
```
