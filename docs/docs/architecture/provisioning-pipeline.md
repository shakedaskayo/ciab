# Provisioning Pipeline

The provisioning pipeline is the core orchestration component of CIAB. It takes a `SandboxSpec` and executes 11 sequential steps to produce a running sandbox with a ready agent.

## Pipeline Steps

```mermaid
graph LR
    V[1. Validate] --> P[2. PrepareImage]
    P --> R[3. ResolveCredentials]
    R --> C[4. CreateSandbox]
    C --> S[5. StartSandbox]
    S --> M[6. MountLocalDirs]
    M --> I[7. InjectCredentials]
    I --> G[8. CloneRepositories]
    G --> F[9. SetupAgentFs]
    F --> X[10. RunScripts]
    X --> A[11. StartAgent]
```

### Step 1: Validate

Validates the `SandboxSpec` and agent configuration:

- Agent provider exists and is enabled
- Resource limits are within allowed ranges
- Required environment variables are present or credential-backed
- Git repo URLs are valid

### Step 2: PrepareImage

Resolves the container or runtime image:

- Uses the provider's `base_image()` or a custom `spec.image`
- For container backends, ensures the image is available in the registry

### Step 3: ResolveCredentials

Fetches and decrypts credentials from the credential store:

- Looks up each credential ID in `spec.credentials`
- Decrypts secrets using AES-GCM
- Builds the final environment variable map

### Step 4: CreateSandbox

Creates the sandbox via the runtime backend:

- Sends the sandbox spec with resource limits, network policy, and volumes
- Receives a sandbox ID and initial state (`Pending`)

### Step 5: StartSandbox

Starts the sandbox:

- Calls the runtime backend's start API
- Waits for the sandbox to reach `Running` state
- Applies port mappings and network configuration

### Step 6: MountLocalDirs

Mounts local directories into the sandbox:

- Iterates over `spec.local_mounts`
- Syncs each source directory to the sandbox with configured exclude patterns
- Supports writeback mode for bidirectional syncing

### Step 7: InjectCredentials

Injects credentials into the running sandbox:

- Sets environment variables via the exec API
- Writes file-based credentials (SSH keys, config files) to the filesystem

### Step 8: CloneRepositories

Clones Git repositories into the sandbox:

- Iterates over `spec.git_repos`
- Clones each repo to its `dest_path` using the specified branch and depth
- Uses credential-backed Git tokens for authentication

### Step 9: SetupAgentFs

Sets up the agent-specific filesystem layout:

- Creates required directories for the agent provider
- Writes agent configuration files (e.g., settings, CLAUDE.md)
- Installs agent skills from the workspace spec

### Step 10: RunScripts

Executes provisioning scripts:

- Runs each script in `spec.provisioning_scripts` sequentially
- Scripts run as the sandbox user with a configurable timeout
- Script size is limited by `provisioning.max_script_size_bytes`

### Step 11: StartAgent

Starts the coding agent process:

- Calls the provider's `build_start_command()` to get the agent command
- Executes the command via the exec API
- Runs a health check via the provider's `health_check()` method

## Streaming Events

Each step emits a `ProvisioningStep` event via SSE:

```json
{
  "event_type": "provisioning_step",
  "data": {
    "step": "CloneRepositories",
    "status": "in_progress",
    "message": "Cloning https://github.com/user/repo.git..."
  }
}
```

On completion: `ProvisioningComplete`. On failure: `ProvisioningFailed`.

## Error Handling

If any step fails:

1. The error is emitted as a `ProvisioningFailed` event
2. The sandbox is cleaned up (stopped and deleted)
3. A `CiabError::ProvisioningFailed` is returned to the caller

The entire pipeline has a configurable timeout (`provisioning.timeout_secs`, default 300s).
