# Workspaces

Workspaces are **reusable, composable environment definitions** that bundle everything a coding agent needs into a single configuration: repositories, skills, pre-commands, binaries, filesystem settings, agent configuration, subagents, credentials, and environment variables.

## Why Workspaces?

Without workspaces, every sandbox requires manually specifying repos, credentials, tools, and agent settings. Workspaces solve this by:

- **Reusability** — Define once, launch many sandboxes from the same workspace
- **Composability** — Mix and match repos, skills, and agent configurations
- **Portability** — Export as TOML for CI pipelines, share across teams
- **Reproducibility** — Same configuration produces identical environments

## Quick Example

Create a workspace via TOML:

```toml
[workspace]
name = "my-project"

[workspace.agent]
provider = "claude-code"
model = "claude-sonnet-4-20250514"
system_prompt = "You are a senior developer working on this project."

[[workspace.repositories]]
url = "https://github.com/shakedaskayo/repo.git"
branch = "main"

[[workspace.skills]]
source = "vercel-labs/ai-sdk-best-practices"

[[workspace.pre_commands]]
name = "Install deps"
command = "npm"
args = ["install"]

[[workspace.credentials]]
name = "api-key"
env_var = "ANTHROPIC_API_KEY"
```

Import and launch:

```bash
ciab workspace import workspace.toml
ciab workspace launch <workspace-id>
```

Or via API:

```bash
# Import TOML
curl -X POST http://localhost:8080/api/v1/workspaces/import \
  -H "Content-Type: text/plain" \
  --data-binary @workspace.toml

# Launch a sandbox from the workspace
curl -X POST http://localhost:8080/api/v1/workspaces/{id}/launch
```

## Workspace Components

| Component | Description |
|-----------|-------------|
| [Repositories](repositories.md) | Git repos to clone with branch/tag/commit pinning |
| [Skills](skills.md) | Reusable agent capabilities (skills.sh compatible) |
| [Pre-commands](pre-commands.md) | Setup commands run before the agent starts |
| [Binaries](binaries.md) | Additional tools to install (apt, cargo, npm, pip) |
| [Filesystem](filesystem.md) | Working directory, CoW isolation, exclusions |
| [Agent](agent-config.md) | Provider, model, system prompt, MCP servers |
| [Subagents](subagents.md) | Additional agent instances for specialized tasks |
| [Credentials](credentials.md) | Secret references from vault providers |

## TOML Configuration

Workspaces can be fully defined as TOML files, making them perfect for:

- **CI/CD pipelines** — Commit workspace TOML alongside your code
- **Team sharing** — Version-controlled environment definitions
- **Single-run usage** — `ciab workspace import workspace.toml && ciab workspace launch <id>`

See [workspace.example.toml](https://github.com/shakedaskayo/ciab/blob/main/workspace.example.toml) for a complete example.

## API vs CLI vs Desktop

Workspaces are a first-class concept across all CIAB interfaces:

- **CLI**: `ciab workspace create|list|get|update|delete|launch|export|import`
- **API**: Full CRUD at `/api/v1/workspaces` plus launch, export, and import endpoints
- **Desktop**: Visual workspace editor with tab-based configuration
