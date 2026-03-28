# Configuration Reference

CIAB is configured via a TOML file (default: `config.toml`).

Generate a default config with `ciab config init`.

## Full Reference

```toml
[server]
host = "0.0.0.0"              # Bind address
port = 9090                    # HTTP port
workers = 4                    # Worker threads (default: CPU count)
request_timeout_secs = 300     # Request timeout
cors_origins = ["*"]           # CORS allowed origins

[runtime]
backend = "local"                          # "local", "docker", "opensandbox", "kubernetes", "ec2"
# opensandbox_url = "http://localhost:8000"   # OpenSandbox API URL
# opensandbox_api_key = "${OPENSANDBOX_API_KEY}"  # Optional API key

# [runtime.kubernetes]
# namespace = "ciab-agents"
# agent_image = "ghcr.io/shakedaskayo/ciab-claude:latest"
# runtime_class = "kata-containers"         # Optional: Kata Containers
# storage_class = "standard"
# workspace_pvc_size = "10Gi"

# [runtime.ec2]
# region = "us-east-1"
# default_ami = "ami-0abcdef1234567890"
# instance_type = "t3.medium"
# subnet_id = "subnet-0123456789abcdef0"
# security_group_ids = ["sg-0123456789abcdef0"]
# ssh_user = "ubuntu"
# root_volume_size_gb = 30

# [packer]
# template_source = "builtin://default-ec2"
# default_region = "us-east-1"
# build_instance_type = "t3.medium"
# auto_install = true

[agents]
default_provider = "claude-code"   # Default agent provider

[agents.providers.claude-code]
enabled = true
image = "ghcr.io/shakedaskayo/ciab-claude:latest"
default_model = "claude-sonnet-4-20250514"
api_key_env = "ANTHROPIC_API_KEY"

[agents.providers.codex]
enabled = true
image = "ghcr.io/shakedaskayo/ciab-codex:latest"
api_key_env = "OPENAI_API_KEY"

[agents.providers.gemini]
enabled = false
image = "ghcr.io/shakedaskayo/ciab-gemini:latest"
api_key_env = "GOOGLE_API_KEY"

[agents.providers.cursor]
enabled = false
image = "ghcr.io/shakedaskayo/ciab-cursor:latest"
api_key_env = "CURSOR_API_KEY"

[credentials]
backend = "sqlite"                     # Storage backend
encryption_key_env = "CIAB_ENCRYPTION_KEY"  # Env var with AES key

[provisioning]
timeout_secs = 300                     # Max provisioning time
max_script_size_bytes = 1048576        # Max script size (1MB)

[streaming]
buffer_size = 500                      # Events buffered per sandbox
keepalive_interval_secs = 15           # SSE heartbeat interval
max_stream_duration_secs = 3600        # Max SSE connection duration

[security]
api_keys = []                          # API keys (empty = auth disabled)
drop_capabilities = ["NET_RAW", "SYS_ADMIN"]  # Linux capabilities to drop

[logging]
level = "info"                         # Log level: trace, debug, info, warn, error
format = "json"                        # Log format: json, pretty

# Optional OAuth configuration
# [oauth.providers.github]
# client_id = "${GITHUB_CLIENT_ID}"
# client_secret_env = "GITHUB_CLIENT_SECRET"
# auth_url = "https://github.com/login/oauth/authorize"
# token_url = "https://github.com/login/oauth/access_token"
# scopes = ["repo", "read:org"]
# redirect_uri = "http://localhost:9090/api/v1/oauth/github/callback"
```

## Section Details

### `[server]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `host` | string | `0.0.0.0` | Bind address |
| `port` | u16 | `9090` | HTTP port |
| `workers` | u16 | CPU count | Worker threads |
| `request_timeout_secs` | u32 | `300` | Global request timeout |
| `cors_origins` | string[] | `["*"]` | CORS allowed origins |

### `[runtime]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `backend` | string | `local` | Runtime backend: `local`, `docker`, `opensandbox`, `kubernetes`, `ec2` |
| `opensandbox_url` | string | — | OpenSandbox API base URL (opensandbox backend) |
| `opensandbox_api_key` | string | — | OpenSandbox API key |

### `[runtime.kubernetes]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `namespace` | string | `ciab-agents` | Namespace for agent Pods |
| `agent_image` | string | — | Default container image for agent Pods |
| `runtime_class` | string | — | RuntimeClass name (e.g. `kata-containers` for microVM isolation) |
| `storage_class` | string | `standard` | StorageClass for workspace PVCs |
| `workspace_pvc_size` | string | `10Gi` | PVC size per workspace |
| `create_network_policy` | bool | `true` | Create NetworkPolicy to isolate agent Pods |
| `run_as_non_root` | bool | `true` | Run agent containers as non-root |
| `drop_all_capabilities` | bool | `true` | Drop all Linux capabilities |
| `default_cpu_request` | string | `500m` | Default CPU request |
| `default_cpu_limit` | string | `2` | Default CPU limit |
| `default_memory_request` | string | `256Mi` | Default memory request |
| `default_memory_limit` | string | `2Gi` | Default memory limit |
| `kubeconfig` | string | — | Path to kubeconfig (omit for in-cluster) |
| `context` | string | — | Kubeconfig context |

See [Kubernetes Deployment](../deployment/kubernetes.md) for full setup instructions.

### `[runtime.ec2]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `region` | string | `us-east-1` | AWS region for instances |
| `default_ami` | string | -- | Default AMI for agent instances |
| `instance_type` | string | `t3.medium` | EC2 instance type |
| `subnet_id` | string | -- | VPC subnet ID |
| `security_group_ids` | string[] | `[]` | Security group IDs |
| `ssh_user` | string | `ubuntu` | SSH user on the AMI |
| `ssh_port` | u16 | `22` | SSH port |
| `ssh_timeout_secs` | u32 | `120` | Timeout waiting for SSH readiness |
| `key_pair_name` | string | -- | AWS key pair name (empty = ephemeral keys) |
| `iam_instance_profile` | string | -- | IAM instance profile ARN |
| `root_volume_size_gb` | u32 | `30` | Root EBS volume size in GB |
| `root_volume_type` | string | `gp3` | EBS volume type |
| `terminate_on_delete` | bool | `true` | Terminate instance on sandbox deletion |
| `stop_on_pause` | bool | `true` | Stop instance on sandbox pause |
| `instance_ready_timeout_secs` | u64 | `180` | Max time to wait for instance startup |

See [AWS EC2 Deployment](../deployment/ec2.md) for full setup instructions.

### `[packer]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `binary` | string | `packer` | Path to the Packer binary |
| `auto_install` | bool | `true` | Auto-install Packer if not found |
| `work_dir` | string | `/tmp/ciab-packer` | Working directory for builds |
| `default_region` | string | `us-east-1` | Default AWS region for AMI builds |
| `build_instance_type` | string | `t3.medium` | Instance type for build instances |
| `build_subnet_id` | string | -- | VPC subnet for build instances |
| `build_timeout_secs` | u32 | `1800` | Build process timeout |
| `template_source` | string | `builtin://default-ec2` | Packer template source |

See [Packer Image Builder](../deployment/packer.md) for full documentation.

### `[agents]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `default_provider` | string | `claude-code` | Default provider for new sandboxes |

### `[agents.providers.<name>]`

| Key | Type | Description |
|-----|------|-------------|
| `enabled` | bool | Whether this provider is available |
| `image` | string | Container image |
| `default_model` | string | Default AI model |
| `api_key_env` | string | Env var for the API key |

### `[credentials]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `backend` | string | `sqlite` | Storage backend |
| `encryption_key_env` | string | `CIAB_ENCRYPTION_KEY` | Env var with the AES encryption key |

### `[provisioning]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `timeout_secs` | u32 | `300` | Max provisioning duration |
| `max_script_size_bytes` | u64 | `1048576` | Max provisioning script size |

### `[streaming]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `buffer_size` | u32 | `500` | Events buffered per sandbox for replay |
| `keepalive_interval_secs` | u32 | `15` | SSE heartbeat interval |
| `max_stream_duration_secs` | u32 | `3600` | Max SSE connection lifetime |

### `[security]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `api_keys` | string[] | `[]` | Valid API keys (empty = auth disabled) |
| `drop_capabilities` | string[] | `["NET_RAW", "SYS_ADMIN"]` | Linux capabilities to drop from containers |

### `[logging]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `level` | string | `info` | Log level |
| `format` | string | `json` | Output format: `json` or `pretty` |

## Config Resolution Chain

CIAB supports zero-config startup. When no explicit config file is provided, configuration is resolved through a 5-step chain where each step overrides values from the previous:

1. **Built-in defaults** -- Sensible defaults for all fields (local runtime, port 9090, etc.)
2. **`./config.toml`** -- Config file in the current working directory
3. **`~/.config/ciab/config.toml`** -- User-level config file
4. **Environment variables** -- `CIAB_PORT`, `CIAB_RUNTIME_BACKEND`, etc. (see [Environment Variables](environment-variables.md))
5. **CLI flags / builder overrides** -- Values set via CLI flags or `CiabEngineBuilder`

This means running `ciab server start` with no arguments works out of the box using built-in defaults.

!!! tip
    Use `ciab config show` to see the resolved configuration after all sources are merged.
