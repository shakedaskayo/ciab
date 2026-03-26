# CIAB Rust Library API & Cloud Provisioners

**Date:** 2026-03-26
**Status:** Approved

## Overview

Three additions to CIAB:

1. **`ciab` facade crate** — Native Rust library API for embedding CIAB in any Rust application
2. **Cloud provisioners** — AWS EC2 runtime + HashiCorp Packer image builder
3. **Config improvements** — Embedded default config, remote config fetch, resolution chain

## 1. `ciab` Facade Crate (Rust Library API)

### Purpose

Provide a single `ciab` crate that Rust applications depend on to programmatically create, manage, and interact with agent sandboxes — without going through HTTP.

### API Surface

```rust
use ciab::{CiabEngine, SandboxSpec, StreamEvent};

// Full-featured builder
let engine = CiabEngine::builder()
    .config_from_file("config.toml")       // or .config_from_url(), .config_default()
    .runtime("local", LocalProcessRuntime::new(workdir))
    .runtime("ec2", Ec2Runtime::new(ec2_config))
    .agent("claude-code", ClaudeCodeProvider::new())
    .database(Database::sqlite("ciab.db").await?)
    .build()
    .await?;

// Zero-config local-only engine
let engine = CiabEngine::builder().build().await?;

// High-level operations
let sandbox = engine.create_sandbox(&SandboxSpec { ... }).await?;
let mut stream = engine.send_message(sandbox.id, session_id, "Fix the bug").await?;
while let Some(event) = stream.recv().await {
    println!("{:?}", event);
}

// Low-level access
let runtime = engine.runtime("ec2")?;
runtime.exec(&sandbox.id, &ExecRequest { ... }).await?;
```

### Design Decisions

- **`CiabEngine`** wraps current `AppState` internals behind a clean, documented API
- **Builder pattern** with sensible defaults — `.build()` with zero config creates a local-only engine
- **Cargo features** gate heavy dependencies: `features = ["ec2", "packer", "kubernetes"]`
- **Re-exports** `ciab-core` types (`SandboxSpec`, `SandboxInfo`, `StreamEvent`, etc.) so embedders don't need direct `ciab-core` dependency
- **Consumers:** `ciab-api` and `ciab-cli` become consumers of this crate, replacing manual wiring

### Cargo Features

```toml
[features]
default = ["local"]
local = ["ciab-sandbox"]
ec2 = ["ciab-sandbox-ec2"]
kubernetes = ["ciab-sandbox-k8s"]
packer = ["ciab-packer"]
full = ["local", "ec2", "kubernetes", "packer"]
```

### `CiabEngine` Methods

**Sandbox lifecycle:**
- `create_sandbox(&SandboxSpec) -> CiabResult<SandboxInfo>`
- `get_sandbox(Uuid) -> CiabResult<SandboxInfo>`
- `list_sandboxes(Option<&str>) -> CiabResult<Vec<SandboxInfo>>`
- `start_sandbox(Uuid) -> CiabResult<()>`
- `stop_sandbox(Uuid) -> CiabResult<()>`
- `terminate_sandbox(Uuid) -> CiabResult<()>`

**Sessions & messaging:**
- `create_session(Uuid) -> CiabResult<SessionInfo>`
- `send_message(Uuid, Uuid, &str) -> CiabResult<Receiver<StreamEvent>>`

**Execution:**
- `exec(Uuid, &ExecRequest) -> CiabResult<ExecResult>`
- `exec_streaming(Uuid, &ExecRequest) -> CiabResult<Receiver<String>>`

**Files:**
- `read_file(Uuid, &str) -> CiabResult<Vec<u8>>`
- `write_file(Uuid, &str, &[u8]) -> CiabResult<()>`
- `list_files(Uuid, &str) -> CiabResult<Vec<FileInfo>>`

**Image building (feature-gated):**
- `build_image(&ImageBuildRequest) -> CiabResult<ImageBuildResult>`
- `list_images() -> CiabResult<Vec<BuiltImage>>`

**Low-level access:**
- `runtime(&str) -> CiabResult<Arc<dyn SandboxRuntime>>`
- `agent(&str) -> CiabResult<Arc<dyn AgentProvider>>`
- `config() -> &AppConfig`

## 2. `ciab-sandbox-ec2` (AWS EC2 Runtime)

### Purpose

`SandboxRuntime` implementation that manages ephemeral EC2 instances — one instance per sandbox, commands executed over SSH.

### Lifecycle Mapping

| SandboxRuntime method | EC2 action |
|---|---|
| `create_sandbox` | `RunInstances` (launch from AMI, tag with sandbox ID) + wait for SSH |
| `start_sandbox` | `StartInstances` |
| `stop_sandbox` | `StopInstances` |
| `terminate_sandbox` | `TerminateInstances` |
| `pause/resume` | Returns `CiabError::Unsupported` |
| `exec` | SSH command execution via `russh` |
| `exec_streaming` | SSH exec with streamed stdout/stderr |
| `read_file/write_file` | SFTP over SSH (`russh-sftp`) |
| `list_files` | SSH `ls -la` parsed |
| `get_stats` | `DescribeInstances` for instance metadata + SSH for system stats |
| `stream_logs` | SSH `tail -f` on agent log path |

### SSH Management

- On `create_sandbox`, generate an ephemeral Ed25519 keypair
- Public key injected via EC2 user-data (cloud-init `authorized_keys`)
- Private key held in memory per sandbox (never persisted to disk)
- Connection pooling: reuse SSH sessions across exec calls per sandbox
- Configurable SSH user (default: `ubuntu`), port (default: 22)

### Instance Tagging

Every instance gets:
- `ciab-sandbox-id = <uuid>`
- `ciab-managed = true`
- User-defined tags from config

`list_sandboxes` uses `DescribeInstances` with tag filter `ciab-managed=true`. Safe for shared AWS accounts.

### Configuration

```toml
[runtime.ec2]
region = "us-east-1"
# Credentials: standard AWS credential chain (env vars, ~/.aws, IAM role)
default_ami = "ami-xxxxx"           # Base AMI or Packer-built AMI
instance_type = "t3.medium"
subnet_id = "subnet-xxxxx"          # Optional — uses default VPC if omitted
security_group_ids = ["sg-xxxxx"]   # Must allow SSH inbound from CIAB host
ssh_user = "ubuntu"
ssh_port = 22
iam_instance_profile = ""           # Optional IAM role for the instance
root_volume_size_gb = 20
instance_ready_timeout_secs = 180   # Wait for instance + SSH reachable

[runtime.ec2.tags]
Environment = "ciab"
ManagedBy = "ciab"
```

### Dependencies

- `aws-sdk-ec2` — Instance lifecycle
- `aws-config` — Credential resolution
- `russh` + `russh-sftp` — Async SSH/SFTP
- `ciab-core` — Traits and types

## 3. `ciab-packer` (Image Builder)

### Purpose

Build machine images using HashiCorp Packer. Not a `SandboxRuntime` — it's an `ImageBuilder` that produces AMIs (or other image formats) consumed by EC2 or other runtimes.

### New Trait (`ciab-core`)

```rust
#[async_trait]
pub trait ImageBuilder: Send + Sync {
    /// Start an image build. Returns immediately with a build ID.
    async fn build_image(&self, request: &ImageBuildRequest) -> CiabResult<ImageBuildResult>;

    /// List previously built images.
    async fn list_images(&self) -> CiabResult<Vec<BuiltImage>>;

    /// Delete a built image (e.g., deregister AMI).
    async fn delete_image(&self, image_id: &str) -> CiabResult<()>;

    /// Check status of an in-progress build.
    async fn build_status(&self, build_id: &Uuid) -> CiabResult<ImageBuildStatus>;
}
```

### New Types (`ciab-core`)

```rust
pub struct ImageBuildRequest {
    pub template: TemplateSource,           // Where to load the HCL template
    pub variables: HashMap<String, String>, // Packer variables
    pub agent_provider: Option<String>,     // Auto-set agent-specific vars
    pub tags: HashMap<String, String>,
}

pub enum TemplateSource {
    Inline(String),                         // Raw HCL content
    FilePath(PathBuf),                      // Local file
    Url(String),                            // HTTP(S) URL
    Git { url: String, path: String, ref_: Option<String> }, // Git repo
}

pub struct ImageBuildResult {
    pub build_id: Uuid,
    pub status: ImageBuildStatus,
    pub image_id: Option<String>,           // e.g., "ami-xxxxx" when complete
    pub logs: Vec<String>,
}

pub enum ImageBuildStatus {
    Queued,
    Running,
    Succeeded,
    Failed(String),
}

pub struct BuiltImage {
    pub image_id: String,
    pub provider: String,                   // "amazon-ebs", "googlecompute", etc.
    pub region: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub tags: HashMap<String, String>,
}
```

### Template Resolution

Templates can come from three sources, using a shared resolution utility:

```
file:///path/to/template.pkr.hcl   → read from disk
https://example.com/template.pkr.hcl → HTTP GET
git::https://github.com/org/repo.git//path/to/template.pkr.hcl?ref=v1.0 → shallow clone
```

Git resolution: shallow clone into temp dir, extract the file at the subpath. Cached locally with configurable TTL (default: 1 hour).

### Default Template

CIAB ships `templates/packer/default-ec2.pkr.hcl`:

- **Variables:** `region`, `base_ami`, `instance_type`, `agent_provider`, `agent_binary_url`, `ssh_user`
- **Provisioners:**
  - Install agent CLI binary (curl + install)
  - Configure SSH access (non-root user, authorized_keys setup)
  - Set up working directory structure (`/home/ubuntu/workspace`)
  - Install common dependencies (git, curl, build-essential)
  - Harden image (disable root SSH, minimal packages)
- **Builder:** `amazon-ebs` (produces an AMI)

### Packer Invocation

- Runs `packer build` as a child process via `tokio::process::Command`
- Variables passed via `-var key=value` flags
- Stdout/stderr streamed back as `StreamEvent`s for real-time progress
- Parses Packer's machine-readable output (`-machine-readable`) for image ID extraction
- Packer binary location configurable; `auto_install = true` downloads it if missing

### Configuration

```toml
[packer]
binary = "packer"                   # Path to packer binary
auto_install = true                 # Download packer if not found
template_cache_dir = "/tmp/ciab-packer-cache"
template_cache_ttl_secs = 3600     # Re-fetch git/URL templates after 1 hour

# Default template — used when ImageBuildRequest.template is not specified
default_template = "builtin://default-ec2"  # Ships with CIAB
# Or point to your own:
# default_template = "git::https://github.com/org/templates.git//agent.pkr.hcl?ref=main"

[packer.variables]
region = "us-east-1"
base_ami = "ami-0abcdef1234567890"  # Ubuntu 22.04 base
instance_type = "t3.medium"
# Any additional key-value pairs passed as -var to packer build
```

### Integration with EC2 Runtime

Two flows:

1. **Pre-built AMI:** User runs Packer separately (or via API), gets an AMI ID, sets `runtime.ec2.default_ami = "ami-xxx"` in config.
2. **On-demand build:** `Ec2RuntimeConfig` has `packer_config: Option<PackerConfig>`. If `default_ami` is not set but Packer is configured, `create_sandbox` triggers a Packer build first, caches the resulting AMI, then launches the instance.

### Dependencies

- `tokio::process` — Run packer binary
- `git2` — Git clone for template resolution
- `reqwest` — HTTP template fetch
- `ciab-core` — `ImageBuilder` trait and types

## 4. Config Improvements

### Embedded Default Config

- New file: `config.default.toml` at repo root
- Embedded in binary via `include_str!()` in `ciab-core`
- Contains working defaults: local runtime, claude-code agent, SQLite credentials, sensible timeouts
- Not an example — a production-ready starting point

### Config Resolution Chain

Priority order (highest first):

1. `--config <path-or-url>` explicit CLI flag
2. `CIAB_CONFIG` environment variable
3. `./config.toml` in current working directory
4. `~/.config/ciab/config.toml` user-level config
5. Embedded `config.default.toml` (compiled into binary)

### Remote Config Support

Same resolution syntax as Packer templates:

```bash
# HTTP URL
ciab server start --config https://raw.githubusercontent.com/org/infra/main/ciab.toml

# Git repo
ciab server start --config git::https://github.com/org/infra.git//ciab/config.toml?ref=prod
```

Fetched at startup, validated, then used. No caching — always fresh on restart.

### `CiabEngine` Builder Integration

```rust
// From file
CiabEngine::builder().config_from_file("config.toml").build().await?;

// From URL
CiabEngine::builder().config_from_url("https://example.com/ciab.toml").build().await?;

// Embedded default
CiabEngine::builder().config_default().build().await?;

// Zero-arg (uses resolution chain)
CiabEngine::builder().build().await?;
```

## 5. Shared Utilities: Resource Resolution

A common module (in `ciab-core` or a thin `ciab-resolve` module) for fetching resources from multiple source types. Used by both Packer template resolution and remote config fetch.

```rust
pub enum ResourceSource {
    FilePath(PathBuf),
    Url(String),
    Git { url: String, path: String, ref_: Option<String> },
    Builtin(&'static str),  // include_str!() content
}

pub async fn resolve_resource(source: &ResourceSource) -> CiabResult<String> { ... }
pub fn parse_source_string(s: &str) -> ResourceSource { ... }
```

**Parsing rules:**
- Starts with `git::` → `Git` variant, parse `//subpath` and `?ref=`
- Starts with `http://` or `https://` → `Url`
- Starts with `builtin://` → `Builtin`
- Everything else → `FilePath`

## 6. New Crate Structure

| Crate | Type | Purpose |
|---|---|---|
| `crates/ciab/` | Library (facade) | `CiabEngine`, re-exports, feature gates |
| `crates/ciab-sandbox-ec2/` | Library | EC2 `SandboxRuntime` implementation |
| `crates/ciab-packer/` | Library | Packer `ImageBuilder` implementation |

### Changes to Existing Crates

- **`ciab-core`**: Add `ImageBuilder` trait, `ImageBuildRequest`/`Result`/`Status` types, `ResourceSource` resolver, `Ec2RuntimeConfig`, `PackerConfig` to config types, embed `config.default.toml`
- **`ciab-cli`**: Depend on `ciab` facade instead of manual wiring. Server startup becomes `CiabEngine::builder().config_from_file(path).build().await?`. Add `ciab image build` / `ciab image list` CLI commands.
- **`ciab-api`**: Depend on `ciab` facade. Add image builder API routes (`POST /api/v1/images/build`, `GET /api/v1/images`). `AppState` wraps or delegates to `CiabEngine`.

### New Files in Repo

- `config.default.toml` — Embedded production-ready default config
- `templates/packer/default-ec2.pkr.hcl` — Default Packer template for EC2 AMIs

## 7. Error Handling

New error variants in `CiabError`:

- `Ec2Error(String)` — AWS SDK errors
- `SshError(String)` — SSH connection/exec failures
- `PackerError(String)` — Packer build failures
- `ImageBuildError(String)` — General image building errors
- `ResourceResolutionError(String)` — Failed to fetch config/template from URL/git
- `Unsupported(String)` — For operations a runtime doesn't support (e.g., EC2 pause)

## 8. Testing Strategy

- **`ciab` facade**: Unit tests for builder logic, config resolution chain, feature gate compilation tests
- **`ciab-sandbox-ec2`**: Mock AWS SDK responses for unit tests. Integration tests behind `#[cfg(feature = "ec2-integration")]` that require real AWS credentials.
- **`ciab-packer`**: Unit tests for template resolution, variable merging, output parsing. Integration tests behind feature flag requiring Packer binary.
- **Config resolution**: Unit tests for each resolution step, mock HTTP server for URL fetch tests.
