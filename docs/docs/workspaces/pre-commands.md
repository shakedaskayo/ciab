# Pre-commands

Pre-commands are shell commands executed during sandbox provisioning, before the agent starts. Use them for dependency installation, database setup, build steps, and environment preparation.

## Configuration

```toml
[[workspace.pre_commands]]
name = "Install dependencies"
command = "npm"
args = ["install"]
workdir = "/workspace/frontend"
fail_on_error = true
timeout_secs = 120
env = { NODE_ENV = "development" }
```

## Fields

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `name` | string | No | — | Human-readable step name |
| `command` | string | Yes | — | Command to execute |
| `args` | string[] | No | `[]` | Command arguments |
| `workdir` | string | No | — | Working directory |
| `env` | object | No | `{}` | Additional environment variables |
| `fail_on_error` | boolean | No | `true` | Stop provisioning on failure |
| `timeout_secs` | integer | No | — | Command timeout |

## Execution Order

Pre-commands run in the order they appear in the TOML file, after:

1. Skills are installed
2. Binaries are installed

And before:

3. The agent process starts

## Examples

```toml
# Install dependencies
[[workspace.pre_commands]]
name = "Frontend deps"
command = "npm"
args = ["ci"]
workdir = "/workspace/frontend"
timeout_secs = 120

# Run database migrations
[[workspace.pre_commands]]
name = "DB setup"
command = "bash"
args = ["-c", "cargo sqlx database setup"]
workdir = "/workspace/backend"

# Non-critical setup (won't fail provisioning)
[[workspace.pre_commands]]
name = "Download sample data"
command = "curl"
args = ["-fsSL", "-o", "/workspace/data/sample.json", "https://example.com/data.json"]
fail_on_error = false
```
