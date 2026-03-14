# Repositories

Workspaces can include one or more Git repositories that are cloned into the sandbox during provisioning.

## Configuration

```toml
[[workspace.repositories]]
url = "https://github.com/shakedaskayo/repo.git"
branch = "main"                    # Branch to checkout
# tag = "v1.0.0"                  # Or pin to a tag
# commit = "abc123"               # Or pin to a specific commit
dest_path = "/workspace/repo"      # Clone destination
depth = 1                          # Shallow clone (faster)
credential_id = "github-token"     # For private repos
sparse_paths = ["src/", "tests/"]  # Sparse checkout
submodules = true                  # Init submodules
```

## Fields

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `url` | string | Yes | — | Git clone URL (HTTPS or SSH) |
| `branch` | string | No | `main` | Branch to checkout |
| `tag` | string | No | — | Tag to checkout (overrides branch) |
| `commit` | string | No | — | Specific commit hash |
| `dest_path` | string | No | `/workspace/<repo-name>` | Clone destination path |
| `depth` | integer | No | — | Shallow clone depth |
| `credential_id` | string | No | — | Credential ID for authentication |
| `sparse_paths` | string[] | No | `[]` | Paths for sparse checkout |
| `submodules` | boolean | No | `false` | Initialize and update submodules |

## Authentication

For private repositories, reference a credential by ID:

```toml
[[workspace.credentials]]
name = "github-token"
vault_provider = "local"
env_var = "GITHUB_TOKEN"

[[workspace.repositories]]
url = "https://github.com/shakedaskayo/private-repo.git"
credential_id = "github-token"
```

## Multiple Repositories

```toml
[[workspace.repositories]]
url = "https://github.com/org/frontend.git"
dest_path = "/workspace/frontend"

[[workspace.repositories]]
url = "https://github.com/org/backend.git"
dest_path = "/workspace/backend"

[[workspace.repositories]]
url = "https://github.com/org/shared-libs.git"
dest_path = "/workspace/libs"
branch = "stable"
```
