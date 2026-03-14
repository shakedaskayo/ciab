# Provisioning Pipeline

The provisioning pipeline is the core orchestration component of CIAB. It takes a `SandboxSpec` and executes 9 sequential steps to produce a running sandbox with a ready agent.

## Pipeline Steps

```mermaid
graph LR
    V[1. Validate] --> P[2. PrepareImage]
    P --> R[3. ResolveCredentials]
    R --> C[4. CreateSandbox]
    C --> S[5. StartSandbox]
    S --> I[6. InjectCredentials]
    I --> G[7. CloneRepositories]
    G --> X[8. RunScripts]
    X --> A[9. StartAgent]
```

### Step 1: Validate

Validates the `SandboxSpec` and agent configuration:

- Agent provider exists and is enabled
- Resource limits are within allowed ranges
- Required environment variables are present or credential-backed
- Git repo URLs are valid

### Step 2: PrepareImage

Resolves the container image:

- Uses the provider's `base_image()` or a custom `spec.image`
- Ensures the image is available in the OpenSandbox registry

### Step 3: ResolveCredentials

Fetches and decrypts credentials from the credential store:

- Looks up each credential ID in `spec.credentials`
- Decrypts secrets using AES-GCM
- Builds the final environment variable map

### Step 4: CreateSandbox

Creates the container via the OpenSandbox API:

- Sends the container spec with resource limits, network policy, and volumes
- Receives a sandbox ID and initial state (`Pending`)

### Step 5: StartSandbox

Starts the container:

- Calls the OpenSandbox start API
- Waits for the container to reach `Running` state
- Applies port mappings and network configuration

### Step 6: InjectCredentials

Injects credentials into the running sandbox:

- Sets environment variables via the execd API
- Writes file-based credentials (SSH keys, config files) to the filesystem

### Step 7: CloneRepositories

Clones Git repositories into the sandbox:

- Iterates over `spec.git_repos`
- Clones each repo to its `dest_path` using the specified branch and depth
- Uses credential-backed Git tokens for authentication

### Step 8: RunScripts

Executes provisioning scripts:

- Runs each script in `spec.provisioning_scripts` sequentially
- Scripts run as the sandbox user with a configurable timeout
- Script size is limited by `provisioning.max_script_size_bytes`

### Step 9: StartAgent

Starts the coding agent process:

- Calls the provider's `build_start_command()` to get the agent command
- Executes the command via the execd API
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
