# Subagents

Subagents are additional agent instances that run alongside the primary agent, each with their own provider, model, and system prompt. Use them for specialized tasks like code review, testing, or documentation.

## Configuration

```toml
[[workspace.subagents]]
name = "reviewer"
provider = "claude-code"
model = "claude-sonnet-4-20250514"
activation = "on_demand"
system_prompt = "You are a code reviewer. Focus on correctness, security, and style."

[[workspace.subagents]]
name = "test-runner"
provider = "claude-code"
activation = "on_demand"
system_prompt = "Run tests and report results. Focus on coverage and failure analysis."
```

## Fields

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `name` | string | Yes | — | Unique name within workspace |
| `provider` | string | Yes | — | Agent provider |
| `model` | string | No | provider default | Model identifier |
| `system_prompt` | string | No | — | System prompt for the subagent |
| `activation` | string | No | `"on_demand"` | When the subagent starts |
| `allowed_tools` | string[] | No | `[]` | Tool allowlist |
| `mcp_servers` | array | No | `[]` | MCP servers for this subagent |

## Activation Modes

| Mode | Description |
|------|-------------|
| `always` | Starts with the sandbox, always running |
| `on_demand` | Started when the primary agent requests it |
| `on_event` | Triggered by specific event types |

## Example: Multi-Agent Workflow

```toml
# Primary agent: developer
[workspace.agent]
provider = "claude-code"
system_prompt = "You are the lead developer. Delegate reviews to @reviewer and tests to @test-runner."

# Subagent: code reviewer
[[workspace.subagents]]
name = "reviewer"
provider = "claude-code"
activation = "on_demand"
system_prompt = "Review code changes for bugs, security issues, and style violations."

# Subagent: test runner
[[workspace.subagents]]
name = "test-runner"
provider = "claude-code"
activation = "on_demand"
system_prompt = "Run the test suite and report results with coverage analysis."

# Subagent: documentation writer
[[workspace.subagents]]
name = "docs-writer"
provider = "claude-code"
activation = "on_demand"
system_prompt = "Generate and update documentation for code changes."
```
