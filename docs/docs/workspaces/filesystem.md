# Filesystem Settings

Configure the sandbox filesystem: working directory, copy-on-write isolation, file size limits, and exclusion patterns.

## Configuration

```toml
[workspace.filesystem]
workdir = "/workspace"
cow_isolation = false
persist_changes = true
max_file_size_bytes = 10485760
readonly_paths = ["/etc", "/usr"]
writable_paths = ["/workspace", "/tmp"]
exclude_patterns = ["**/node_modules/**", "**/target/**", "**/.git/**"]
```

## Fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `workdir` | string | `"/workspace"` | Agent working directory |
| `cow_isolation` | boolean | `false` | Enable copy-on-write isolation |
| `persist_changes` | boolean | `false` | Persist filesystem changes across restarts |
| `max_file_size_bytes` | integer | — | Maximum file size the agent can create |
| `tmp_size_mb` | integer | — | Temp directory size limit |
| `readonly_paths` | string[] | `[]` | Paths mounted read-only |
| `writable_paths` | string[] | `[]` | Explicitly writable paths (with CoW) |
| `exclude_patterns` | string[] | `[]` | Glob patterns to exclude from agent access |

## Copy-on-Write Isolation

When `cow_isolation = true`, the sandbox filesystem operates with copy-on-write semantics (inspired by [AgentFS](https://docs.turso.tech/agentfs/introduction)). The agent can modify files freely, but changes are isolated from the source. This is useful for:

- **Safe experimentation** — Changes don't affect the original codebase
- **Reproducible runs** — Each launch starts from a clean state
- **Audit trail** — All file operations are tracked
