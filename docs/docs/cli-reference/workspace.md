# ciab workspace

Manage workspaces — reusable environment definitions.

## Commands

### `ciab workspace create`

Create a new workspace.

```bash
ciab workspace create --name "my-project" --provider claude-code
ciab workspace create --name "imported" --from-toml workspace.toml
```

| Flag | Description |
|------|-------------|
| `--name` | Workspace name (required) |
| `--description` | Optional description |
| `--provider` | Default agent provider |
| `--from-toml` | Create from TOML file |

### `ciab workspace list`

List all workspaces.

```bash
ciab workspace list
ciab workspace list --name "my-project"
```

### `ciab workspace get <id>`

Get workspace details.

### `ciab workspace update <id>`

Update workspace fields.

```bash
ciab workspace update <id> --name "new-name" --description "updated"
```

### `ciab workspace delete <id>`

Delete a workspace.

### `ciab workspace launch <id>`

Create and provision a sandbox from the workspace definition.

```bash
ciab workspace launch <id>
```

### `ciab workspace sandboxes <id>`

List sandboxes created from this workspace.

### `ciab workspace export <id>`

Export workspace as TOML.

```bash
ciab workspace export <id>                    # Print to stdout
ciab workspace export <id> -o workspace.toml  # Save to file
```

### `ciab workspace import <file>`

Import workspace from a TOML file.

```bash
ciab workspace import workspace.toml
```
