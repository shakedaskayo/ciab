# Workspace Credentials

Workspace credentials are references to secrets stored in CIAB's credential vault or external vault providers. They are resolved and injected during provisioning.

## Configuration

```toml
[[workspace.credentials]]
name = "anthropic-key"
vault_provider = "local"
env_var = "ANTHROPIC_API_KEY"

[[workspace.credentials]]
name = "github-token"
vault_provider = "local"
env_var = "GITHUB_TOKEN"
file_path = "/workspace/.github-token"
```

## Fields

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `id` | string | No* | — | Credential ID in CIAB vault |
| `name` | string | No* | — | Credential name (looked up by name) |
| `vault_provider` | string | No | `"local"` | Vault backend |
| `vault_path` | string | No | — | Path in external vault |
| `env_var` | string | No | — | Inject as environment variable |
| `file_path` | string | No | — | Write to file path |

*Either `id` or `name` must be provided.

## Vault Providers

| Provider | Description |
|----------|-------------|
| `local` | CIAB's built-in encrypted credential store |
| `aws-secrets-manager` | AWS Secrets Manager |
| `hashicorp-vault` | HashiCorp Vault |
| `1password` | 1Password CLI |

## Injection Methods

Credentials can be injected as:

1. **Environment variables** — Set via `env_var` field
2. **Files** — Written to `file_path` in the sandbox
3. **Both** — Set both fields to inject in both ways

## Example: Multi-Provider Setup

```toml
# Local CIAB vault
[[workspace.credentials]]
name = "anthropic-key"
vault_provider = "local"
env_var = "ANTHROPIC_API_KEY"

# AWS Secrets Manager
[[workspace.credentials]]
name = "database-url"
vault_provider = "aws-secrets-manager"
vault_path = "prod/myapp/database-url"
env_var = "DATABASE_URL"

# SSH key as file
[[workspace.credentials]]
name = "deploy-key"
vault_provider = "local"
file_path = "/root/.ssh/id_ed25519"
```
