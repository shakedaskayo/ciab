# Rust Library API & Cloud Provisioners Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a native Rust library API (`CiabEngine`), AWS EC2 runtime, HashiCorp Packer image builder, and config improvements with embedded defaults and remote fetch.

**Architecture:** Four phases built bottom-up: (1) core types + config + resource resolver, (2) Packer image builder crate, (3) EC2 runtime crate, (4) `ciab` facade crate. Each phase produces working, testable code. The facade crate re-exports everything and provides `CiabEngine` builder. EC2 and Packer are isolated crates following the existing `ciab-sandbox-k8s` pattern.

**Tech Stack:** Rust, `aws-sdk-ec2`, `aws-config`, `russh`/`russh-sftp` (async SSH), `git2` (libgit2), `tokio::process` (Packer invocation), `reqwest` (HTTP fetch), existing workspace deps.

---

## Phase 1: Core Types, Config, Resource Resolver

### Task 1: Add `ImageBuilder` trait to `ciab-core`

**Files:**
- Create: `crates/ciab-core/src/traits/image_builder.rs`
- Modify: `crates/ciab-core/src/traits/mod.rs`
- Create: `crates/ciab-core/src/types/image.rs`
- Modify: `crates/ciab-core/src/types/mod.rs`

- [ ] **Step 1: Add image types module**

Create `crates/ciab-core/src/types/image.rs`:

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

/// Where to load a Packer template from.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TemplateSource {
    /// Raw HCL content inline.
    Inline { content: String },
    /// Local filesystem path.
    FilePath { path: PathBuf },
    /// HTTP(S) URL.
    Url { url: String },
    /// Git repository with optional subpath and ref.
    Git {
        url: String,
        path: String,
        #[serde(rename = "ref")]
        ref_: Option<String>,
    },
    /// Built-in template shipped with CIAB.
    Builtin { name: String },
}

/// Request to build a machine image.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageBuildRequest {
    /// Where to load the HCL template.
    pub template: Option<TemplateSource>,
    /// Packer variables passed as `-var key=value`.
    #[serde(default)]
    pub variables: HashMap<String, String>,
    /// If set, auto-populates agent-specific variables.
    pub agent_provider: Option<String>,
    /// Tags applied to the resulting image.
    #[serde(default)]
    pub tags: HashMap<String, String>,
}

/// Result of an image build.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageBuildResult {
    pub build_id: Uuid,
    pub status: ImageBuildStatus,
    /// Image ID when build is complete (e.g., "ami-xxxxx").
    pub image_id: Option<String>,
    /// Build log lines.
    #[serde(default)]
    pub logs: Vec<String>,
}

/// Status of an image build.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ImageBuildStatus {
    Queued,
    Running,
    Succeeded,
    Failed(String),
}

/// A previously built image.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuiltImage {
    /// Image identifier (e.g., "ami-xxxxx").
    pub image_id: String,
    /// Packer builder type (e.g., "amazon-ebs").
    pub provider: String,
    pub region: Option<String>,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub tags: HashMap<String, String>,
}
```

- [ ] **Step 2: Register the image types module**

In `crates/ciab-core/src/types/mod.rs`, add after the last existing module:

```rust
pub mod image;
```

- [ ] **Step 3: Add the ImageBuilder trait**

Create `crates/ciab-core/src/traits/image_builder.rs`:

```rust
use async_trait::async_trait;
use uuid::Uuid;

use crate::error::CiabResult;
use crate::types::image::{BuiltImage, ImageBuildRequest, ImageBuildResult, ImageBuildStatus};

/// Trait for building machine images (e.g., AMIs via Packer).
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

- [ ] **Step 4: Register the trait module**

In `crates/ciab-core/src/traits/mod.rs`, add:

```rust
pub mod image_builder;
```

So the file becomes:

```rust
pub mod agent;
pub mod channel;
pub mod image_builder;
pub mod runtime;
pub mod stream;
```

- [ ] **Step 5: Verify it compiles**

Run: `cargo check -p ciab-core`
Expected: Compiles with no errors.

- [ ] **Step 6: Commit**

```bash
git add crates/ciab-core/src/traits/image_builder.rs crates/ciab-core/src/traits/mod.rs crates/ciab-core/src/types/image.rs crates/ciab-core/src/types/mod.rs
git commit -m "feat(core): add ImageBuilder trait and image types"
```

---

### Task 2: Add EC2 and Packer config types

**Files:**
- Modify: `crates/ciab-core/src/types/config.rs`

- [ ] **Step 1: Add Ec2Config struct**

In `crates/ciab-core/src/types/config.rs`, add before the `RuntimeConfig` struct (around line 145):

```rust
/// Configuration for AWS EC2 runtime backend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ec2Config {
    /// AWS region (e.g., "us-east-1").
    #[serde(default = "default_ec2_region")]
    pub region: String,
    /// Default AMI to launch instances from. If unset, Packer builds one.
    pub default_ami: Option<String>,
    /// EC2 instance type.
    #[serde(default = "default_ec2_instance_type")]
    pub instance_type: String,
    /// VPC subnet ID. Uses default VPC if omitted.
    pub subnet_id: Option<String>,
    /// Security group IDs. Must allow SSH inbound.
    #[serde(default)]
    pub security_group_ids: Vec<String>,
    /// SSH username on the AMI.
    #[serde(default = "default_ec2_ssh_user")]
    pub ssh_user: String,
    /// SSH port.
    #[serde(default = "default_ec2_ssh_port")]
    pub ssh_port: u16,
    /// IAM instance profile ARN or name.
    pub iam_instance_profile: Option<String>,
    /// Root EBS volume size in GB.
    #[serde(default = "default_ec2_root_volume_gb")]
    pub root_volume_size_gb: u32,
    /// Timeout waiting for instance + SSH to become reachable.
    #[serde(default = "default_ec2_ready_timeout")]
    pub instance_ready_timeout_secs: u64,
    /// Additional tags applied to every instance.
    #[serde(default)]
    pub tags: HashMap<String, String>,
}

fn default_ec2_region() -> String {
    "us-east-1".to_string()
}
fn default_ec2_instance_type() -> String {
    "t3.medium".to_string()
}
fn default_ec2_ssh_user() -> String {
    "ubuntu".to_string()
}
fn default_ec2_ssh_port() -> u16 {
    22
}
fn default_ec2_root_volume_gb() -> u32 {
    20
}
fn default_ec2_ready_timeout() -> u64 {
    180
}
```

- [ ] **Step 2: Add PackerConfig struct**

In the same file, add after the Ec2Config:

```rust
/// Configuration for HashiCorp Packer image builder.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackerConfig {
    /// Path to the packer binary.
    #[serde(default = "default_packer_binary")]
    pub binary: String,
    /// Automatically install packer if not found on PATH.
    #[serde(default)]
    pub auto_install: bool,
    /// Directory for caching git/URL templates.
    #[serde(default = "default_packer_cache_dir")]
    pub template_cache_dir: String,
    /// TTL for cached templates in seconds.
    #[serde(default = "default_packer_cache_ttl")]
    pub template_cache_ttl_secs: u64,
    /// Default template used when ImageBuildRequest.template is None.
    /// Supports: "builtin://default-ec2", file paths, URLs, git:: prefixed URIs.
    #[serde(default = "default_packer_template")]
    pub default_template: String,
    /// Default variables passed to every packer build.
    #[serde(default)]
    pub variables: HashMap<String, String>,
}

fn default_packer_binary() -> String {
    "packer".to_string()
}
fn default_packer_cache_dir() -> String {
    "/tmp/ciab-packer-cache".to_string()
}
fn default_packer_cache_ttl() -> u64 {
    3600
}
fn default_packer_template() -> String {
    "builtin://default-ec2".to_string()
}
```

- [ ] **Step 3: Add fields to RuntimeConfig**

Add `ec2` and `packer` fields to the existing `RuntimeConfig` struct. Find the struct (around line 145) and add these fields after the existing `kubernetes` field:

```rust
    /// AWS EC2 runtime configuration.
    pub ec2: Option<Ec2Config>,

    /// HashiCorp Packer image builder configuration.
    pub packer: Option<PackerConfig>,
```

- [ ] **Step 4: Add the HashMap import if not already present**

Check the imports at the top of `config.rs`. If `HashMap` is not imported, add:

```rust
use std::collections::HashMap;
```

(It's likely already imported via the existing structs that use HashMap.)

- [ ] **Step 5: Verify it compiles**

Run: `cargo check -p ciab-core`
Expected: Compiles with no errors.

- [ ] **Step 6: Commit**

```bash
git add crates/ciab-core/src/types/config.rs
git commit -m "feat(core): add Ec2Config and PackerConfig to RuntimeConfig"
```

---

### Task 3: Add new error variants

**Files:**
- Modify: `crates/ciab-core/src/error.rs`

- [ ] **Step 1: Add error variants**

In `crates/ciab-core/src/error.rs`, find the `CiabError` enum. Add these variants after the existing runtime errors section (after `KubernetesPodNotFound`, around line 67):

```rust
    // EC2 errors
    #[error("EC2 error: {0}")]
    Ec2Error(String),

    #[error("SSH error: {0}")]
    SshError(String),

    // Image builder errors
    #[error("Packer error: {0}")]
    PackerError(String),

    #[error("Image build error: {0}")]
    ImageBuildError(String),

    // Resource resolution errors
    #[error("Resource resolution error: {0}")]
    ResourceResolutionError(String),

    // Unsupported operation
    #[error("Unsupported operation: {0}")]
    Unsupported(String),
```

- [ ] **Step 2: Add status_code mappings**

In the `status_code()` method, add these mappings in the appropriate section:

```rust
            Self::Ec2Error(_) => StatusCode::BAD_GATEWAY,
            Self::SshError(_) => StatusCode::BAD_GATEWAY,
            Self::PackerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::ImageBuildError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::ResourceResolutionError(_) => StatusCode::BAD_REQUEST,
            Self::Unsupported(_) => StatusCode::NOT_IMPLEMENTED,
```

- [ ] **Step 3: Add error_code mappings**

In the `error_code()` method, add:

```rust
            Self::Ec2Error(_) => "EC2_ERROR",
            Self::SshError(_) => "SSH_ERROR",
            Self::PackerError(_) => "PACKER_ERROR",
            Self::ImageBuildError(_) => "IMAGE_BUILD_ERROR",
            Self::ResourceResolutionError(_) => "RESOURCE_RESOLUTION_ERROR",
            Self::Unsupported(_) => "UNSUPPORTED",
```

- [ ] **Step 4: Verify it compiles**

Run: `cargo check -p ciab-core`
Expected: Compiles with no errors.

- [ ] **Step 5: Commit**

```bash
git add crates/ciab-core/src/error.rs
git commit -m "feat(core): add EC2, SSH, Packer, and resource resolution error variants"
```

---

### Task 4: Resource resolver module in `ciab-core`

**Files:**
- Create: `crates/ciab-core/src/resolve.rs`
- Modify: `crates/ciab-core/src/lib.rs`
- Modify: `crates/ciab-core/Cargo.toml`

- [ ] **Step 1: Add dependencies to ciab-core**

In `crates/ciab-core/Cargo.toml`, add to `[dependencies]`:

```toml
reqwest = { workspace = true }
git2 = "0.19"
tempfile = "3"
```

- [ ] **Step 2: Create the resolver module**

Create `crates/ciab-core/src/resolve.rs`:

```rust
use std::path::{Path, PathBuf};

use tracing::info;

use crate::error::{CiabError, CiabResult};

/// A resolved resource source. Used for config files, Packer templates, etc.
#[derive(Debug, Clone)]
pub enum ResourceSource {
    /// Local filesystem path.
    FilePath(PathBuf),
    /// HTTP(S) URL.
    Url(String),
    /// Git repository.
    Git {
        url: String,
        path: String,
        ref_: Option<String>,
    },
    /// Built-in content compiled into the binary.
    Builtin(String),
}

/// Parse a source string into a ResourceSource.
///
/// Rules:
/// - Starts with `git::` → Git { url, path, ref_ }
/// - Starts with `http://` or `https://` → Url
/// - Starts with `builtin://` → Builtin
/// - Everything else → FilePath
pub fn parse_source_string(s: &str) -> ResourceSource {
    if let Some(rest) = s.strip_prefix("git::") {
        parse_git_source(rest)
    } else if s.starts_with("http://") || s.starts_with("https://") {
        ResourceSource::Url(s.to_string())
    } else if let Some(name) = s.strip_prefix("builtin://") {
        ResourceSource::Builtin(name.to_string())
    } else {
        ResourceSource::FilePath(PathBuf::from(s))
    }
}

/// Parse a git source string like `https://github.com/org/repo.git//path/to/file?ref=main`
fn parse_git_source(s: &str) -> ResourceSource {
    let (url_and_path, ref_) = if let Some(idx) = s.find("?ref=") {
        (&s[..idx], Some(s[idx + 5..].to_string()))
    } else {
        (s, None)
    };

    let (url, path) = if let Some(idx) = url_and_path.find("//") {
        (
            url_and_path[..idx].to_string(),
            url_and_path[idx + 2..].to_string(),
        )
    } else {
        (url_and_path.to_string(), String::new())
    };

    ResourceSource::Git { url, path, ref_ }
}

/// Resolve a resource source to its string content.
pub async fn resolve_resource(source: &ResourceSource) -> CiabResult<String> {
    match source {
        ResourceSource::FilePath(path) => resolve_file(path).await,
        ResourceSource::Url(url) => resolve_url(url).await,
        ResourceSource::Git { url, path, ref_ } => {
            resolve_git(url, path, ref_.as_deref()).await
        }
        ResourceSource::Builtin(name) => resolve_builtin(name),
    }
}

async fn resolve_file(path: &Path) -> CiabResult<String> {
    tokio::fs::read_to_string(path).await.map_err(|e| {
        CiabError::ResourceResolutionError(format!(
            "Failed to read file {}: {}",
            path.display(),
            e
        ))
    })
}

async fn resolve_url(url: &str) -> CiabResult<String> {
    info!(url = url, "Fetching resource from URL");
    let response = reqwest::get(url).await.map_err(|e| {
        CiabError::ResourceResolutionError(format!("Failed to fetch {}: {}", url, e))
    })?;

    if !response.status().is_success() {
        return Err(CiabError::ResourceResolutionError(format!(
            "HTTP {} fetching {}",
            response.status(),
            url
        )));
    }

    response.text().await.map_err(|e| {
        CiabError::ResourceResolutionError(format!("Failed to read response from {}: {}", url, e))
    })
}

async fn resolve_git(url: &str, subpath: &str, ref_: Option<&str>) -> CiabResult<String> {
    let url = url.to_string();
    let subpath = subpath.to_string();
    let ref_ = ref_.map(|s| s.to_string());

    // Git operations are blocking, run in spawn_blocking
    tokio::task::spawn_blocking(move || {
        let tmp = tempfile::tempdir().map_err(|e| {
            CiabError::ResourceResolutionError(format!("Failed to create temp dir: {}", e))
        })?;

        info!(url = %url, subpath = %subpath, ref_ = ?ref_, "Cloning git resource");

        let mut builder = git2::build::RepoBuilder::new();

        // Shallow clone (depth 1)
        let mut fetch_opts = git2::FetchOptions::new();
        fetch_opts.depth(1);
        builder.fetch_options(fetch_opts);

        if let Some(ref branch) = ref_ {
            builder.branch(branch);
        }

        let repo = builder.clone(&url, tmp.path()).map_err(|e| {
            CiabError::ResourceResolutionError(format!("Git clone failed for {}: {}", url, e))
        })?;

        let file_path = if subpath.is_empty() {
            repo.workdir()
                .ok_or_else(|| {
                    CiabError::ResourceResolutionError("Bare repository".to_string())
                })?
                .to_path_buf()
        } else {
            repo.workdir()
                .ok_or_else(|| {
                    CiabError::ResourceResolutionError("Bare repository".to_string())
                })?
                .join(&subpath)
        };

        std::fs::read_to_string(&file_path).map_err(|e| {
            CiabError::ResourceResolutionError(format!(
                "Failed to read {} from cloned repo: {}",
                file_path.display(),
                e
            ))
        })
    })
    .await
    .map_err(|e| CiabError::ResourceResolutionError(format!("Git task panicked: {}", e)))?
}

fn resolve_builtin(name: &str) -> CiabResult<String> {
    match name {
        "default-ec2" => Ok(include_str!("../../../templates/packer/default-ec2.pkr.hcl").to_string()),
        "default-config" => Ok(include_str!("../../../config.default.toml").to_string()),
        _ => Err(CiabError::ResourceResolutionError(format!(
            "Unknown builtin resource: {}",
            name
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_file_path() {
        let source = parse_source_string("/path/to/file.toml");
        assert!(matches!(source, ResourceSource::FilePath(p) if p == PathBuf::from("/path/to/file.toml")));
    }

    #[test]
    fn test_parse_url() {
        let source = parse_source_string("https://example.com/config.toml");
        assert!(matches!(source, ResourceSource::Url(u) if u == "https://example.com/config.toml"));
    }

    #[test]
    fn test_parse_builtin() {
        let source = parse_source_string("builtin://default-ec2");
        assert!(matches!(source, ResourceSource::Builtin(n) if n == "default-ec2"));
    }

    #[test]
    fn test_parse_git_full() {
        let source =
            parse_source_string("git::https://github.com/org/repo.git//path/to/file.hcl?ref=main");
        match source {
            ResourceSource::Git { url, path, ref_ } => {
                assert_eq!(url, "https://github.com/org/repo.git");
                assert_eq!(path, "path/to/file.hcl");
                assert_eq!(ref_, Some("main".to_string()));
            }
            _ => panic!("Expected Git source"),
        }
    }

    #[test]
    fn test_parse_git_no_ref() {
        let source = parse_source_string("git::https://github.com/org/repo.git//template.hcl");
        match source {
            ResourceSource::Git { url, path, ref_ } => {
                assert_eq!(url, "https://github.com/org/repo.git");
                assert_eq!(path, "template.hcl");
                assert_eq!(ref_, None);
            }
            _ => panic!("Expected Git source"),
        }
    }

    #[test]
    fn test_parse_git_no_subpath() {
        let source = parse_source_string("git::https://github.com/org/repo.git?ref=v1.0");
        match source {
            ResourceSource::Git { url, path, ref_ } => {
                assert_eq!(url, "https://github.com/org/repo.git");
                assert_eq!(path, "");
                assert_eq!(ref_, Some("v1.0".to_string()));
            }
            _ => panic!("Expected Git source"),
        }
    }
}
```

- [ ] **Step 3: Register the module**

In `crates/ciab-core/src/lib.rs`, add:

```rust
pub mod resolve;
```

So the file becomes:

```rust
pub mod error;
pub mod resolve;
pub mod traits;
pub mod types;
```

- [ ] **Step 4: Run the unit tests**

Run: `cargo test -p ciab-core -- resolve`
Expected: All 5 tests pass (test_parse_file_path, test_parse_url, test_parse_builtin, test_parse_git_full, test_parse_git_no_ref, test_parse_git_no_subpath).

- [ ] **Step 5: Verify full workspace compiles**

Run: `cargo check --workspace`
Expected: Compiles with no errors (the `include_str!` for builtin templates will fail until we create those files — that's Task 6).

- [ ] **Step 6: Commit**

```bash
git add crates/ciab-core/src/resolve.rs crates/ciab-core/src/lib.rs crates/ciab-core/Cargo.toml
git commit -m "feat(core): add resource resolver for files, URLs, and git repos"
```

---

### Task 5: Embedded default config

**Files:**
- Create: `config.default.toml` (repo root)
- Modify: `crates/ciab-core/src/types/config.rs` (add load_default_config function)

- [ ] **Step 1: Create config.default.toml**

Create `config.default.toml` at the repo root:

```toml
# CIAB Default Configuration
# This config is embedded in the binary and used when no config file is specified.
# Copy to config.toml and customize for your environment.

[server]
host = "0.0.0.0"
port = 9090
workers = 4
request_timeout_secs = 300
cors_origins = ["*"]

[runtime]
backend = "local"
local_workdir = "/tmp/ciab-sandboxes"
local_max_processes = 10

[agents]
default_provider = "claude-code"

[agents.providers.claude-code]
enabled = true
binary = "claude"
default_model = "claude-sonnet-4-20250514"
api_key_env = "ANTHROPIC_API_KEY"

[agents.providers.codex]
enabled = true
binary = "codex"
api_key_env = "OPENAI_API_KEY"

[agents.providers.gemini]
enabled = false
binary = "gemini"
api_key_env = "GOOGLE_API_KEY"

[agents.providers.cursor]
enabled = false
binary = "cursor"
api_key_env = "CURSOR_API_KEY"

[credentials]
backend = "sqlite"
encryption_key_env = "CIAB_ENCRYPTION_KEY"

[provisioning]
timeout_secs = 300
max_script_size_bytes = 1048576

[streaming]
buffer_size = 500
keepalive_interval_secs = 15
max_stream_duration_secs = 3600

[security]
api_keys = []
drop_capabilities = ["NET_RAW", "SYS_ADMIN"]

[logging]
level = "info"
format = "json"

[llm_providers]
auto_detect_ollama = true
```

- [ ] **Step 2: Add config loading helpers to config.rs**

At the bottom of `crates/ciab-core/src/types/config.rs`, add:

```rust
impl AppConfig {
    /// Load the embedded default configuration.
    pub fn load_default() -> Result<Self, toml::de::Error> {
        let content = include_str!("../../../config.default.toml");
        toml::from_str(content)
    }

    /// Load configuration from a string.
    pub fn from_str(content: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(content)
    }

    /// Load configuration using the resolution chain:
    /// 1. Explicit path/URL (if provided)
    /// 2. CIAB_CONFIG env var
    /// 3. ./config.toml
    /// 4. ~/.config/ciab/config.toml
    /// 5. Embedded default
    pub async fn load(explicit_source: Option<&str>) -> crate::error::CiabResult<Self> {
        use crate::resolve::{parse_source_string, resolve_resource, ResourceSource};

        // 1. Explicit source
        if let Some(source) = explicit_source {
            let src = parse_source_string(source);
            let content = resolve_resource(&src).await?;
            return toml::from_str(&content).map_err(|e| {
                crate::error::CiabError::ConfigError(format!("Failed to parse config: {}", e))
            });
        }

        // 2. CIAB_CONFIG env var
        if let Ok(env_source) = std::env::var("CIAB_CONFIG") {
            let src = parse_source_string(&env_source);
            let content = resolve_resource(&src).await?;
            return toml::from_str(&content).map_err(|e| {
                crate::error::CiabError::ConfigError(format!("Failed to parse config: {}", e))
            });
        }

        // 3. ./config.toml
        let local_config = std::path::Path::new("config.toml");
        if local_config.exists() {
            let content = tokio::fs::read_to_string(local_config).await.map_err(|e| {
                crate::error::CiabError::ConfigError(format!("Failed to read config.toml: {}", e))
            })?;
            return toml::from_str(&content).map_err(|e| {
                crate::error::CiabError::ConfigError(format!("Failed to parse config.toml: {}", e))
            });
        }

        // 4. ~/.config/ciab/config.toml
        if let Some(home) = dirs_next::home_dir() {
            let user_config = home.join(".config").join("ciab").join("config.toml");
            if user_config.exists() {
                let content = tokio::fs::read_to_string(&user_config).await.map_err(|e| {
                    crate::error::CiabError::ConfigError(format!(
                        "Failed to read {}: {}",
                        user_config.display(),
                        e
                    ))
                })?;
                return toml::from_str(&content).map_err(|e| {
                    crate::error::CiabError::ConfigError(format!(
                        "Failed to parse {}: {}",
                        user_config.display(),
                        e
                    ))
                });
            }
        }

        // 5. Embedded default
        Self::load_default().map_err(|e| {
            crate::error::CiabError::ConfigError(format!(
                "Failed to parse embedded default config: {}",
                e
            ))
        })
    }
}
```

- [ ] **Step 3: Add dirs-next dependency**

In `crates/ciab-core/Cargo.toml`, add:

```toml
dirs-next = "2"
```

- [ ] **Step 4: Verify it compiles**

Run: `cargo check -p ciab-core`
Expected: Compiles with no errors.

- [ ] **Step 5: Commit**

```bash
git add config.default.toml crates/ciab-core/src/types/config.rs crates/ciab-core/Cargo.toml
git commit -m "feat(core): add embedded default config and config resolution chain"
```

---

### Task 6: Default Packer template

**Files:**
- Create: `templates/packer/default-ec2.pkr.hcl`

- [ ] **Step 1: Create the templates directory and default template**

Create `templates/packer/default-ec2.pkr.hcl`:

```hcl
# CIAB Default EC2 AMI Template
# Builds an Ubuntu-based AMI with a coding agent pre-installed.
#
# Variables are injected by CIAB or passed via -var flags.

packer {
  required_plugins {
    amazon = {
      source  = "github.com/hashicorp/amazon"
      version = ">= 1.3.0"
    }
  }
}

variable "region" {
  type    = string
  default = "us-east-1"
}

variable "base_ami" {
  type        = string
  description = "Base AMI ID (Ubuntu 22.04 recommended)"
}

variable "instance_type" {
  type    = string
  default = "t3.medium"
}

variable "agent_provider" {
  type        = string
  default     = "claude-code"
  description = "Agent CLI to install: claude-code, codex, gemini, cursor"
}

variable "ssh_user" {
  type    = string
  default = "ubuntu"
}

variable "ami_name_prefix" {
  type    = string
  default = "ciab-agent"
}

variable "volume_size" {
  type    = number
  default = 20
}

source "amazon-ebs" "agent" {
  region        = var.region
  source_ami    = var.base_ami
  instance_type = var.instance_type
  ssh_username  = var.ssh_user
  ami_name      = "${var.ami_name_prefix}-${var.agent_provider}-{{timestamp}}"

  launch_block_device_mappings {
    device_name           = "/dev/sda1"
    volume_size           = var.volume_size
    volume_type           = "gp3"
    delete_on_termination = true
  }

  tags = {
    Name       = "${var.ami_name_prefix}-${var.agent_provider}"
    ManagedBy  = "ciab-packer"
    Agent      = var.agent_provider
    BaseAMI    = var.base_ami
    BuiltAt    = "{{timestamp}}"
  }
}

build {
  sources = ["source.amazon-ebs.agent"]

  # System updates and base packages
  provisioner "shell" {
    inline = [
      "sudo apt-get update -y",
      "sudo apt-get install -y git curl wget build-essential unzip jq",
      "sudo apt-get clean",
    ]
  }

  # Install Node.js (required by claude-code, codex, cursor)
  provisioner "shell" {
    inline = [
      "curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -",
      "sudo apt-get install -y nodejs",
    ]
  }

  # Install agent CLI based on provider
  provisioner "shell" {
    inline = [
      "case '${var.agent_provider}' in",
      "  claude-code)",
      "    sudo npm install -g @anthropic-ai/claude-code",
      "    ;;",
      "  codex)",
      "    sudo npm install -g @openai/codex",
      "    ;;",
      "  gemini)",
      "    sudo npm install -g @google/gemini-cli",
      "    ;;",
      "  cursor)",
      "    echo 'Cursor CLI requires manual installation'",
      "    ;;",
      "  *)",
      "    echo 'Unknown agent provider: ${var.agent_provider}'",
      "    exit 1",
      "    ;;",
      "esac",
    ]
  }

  # Create workspace directory
  provisioner "shell" {
    inline = [
      "sudo mkdir -p /home/${var.ssh_user}/workspace",
      "sudo chown -R ${var.ssh_user}:${var.ssh_user} /home/${var.ssh_user}/workspace",
    ]
  }

  # Security hardening
  provisioner "shell" {
    inline = [
      "# Disable root SSH login",
      "sudo sed -i 's/^PermitRootLogin.*/PermitRootLogin no/' /etc/ssh/sshd_config",
      "# Disable password authentication",
      "sudo sed -i 's/^#PasswordAuthentication.*/PasswordAuthentication no/' /etc/ssh/sshd_config",
      "sudo sed -i 's/^PasswordAuthentication.*/PasswordAuthentication no/' /etc/ssh/sshd_config",
    ]
  }

  # Create CIAB marker file with metadata
  provisioner "shell" {
    inline = [
      "echo '{\"agent_provider\":\"${var.agent_provider}\",\"built_at\":\"'$(date -u +%Y-%m-%dT%H:%M:%SZ)'\"}' | sudo tee /etc/ciab-image.json > /dev/null",
    ]
  }
}
```

- [ ] **Step 2: Verify ciab-core compiles with the builtin includes**

Run: `cargo check -p ciab-core`
Expected: Compiles — the `include_str!` in `resolve.rs` now finds both `templates/packer/default-ec2.pkr.hcl` and `config.default.toml`.

- [ ] **Step 3: Commit**

```bash
git add templates/packer/default-ec2.pkr.hcl
git commit -m "feat: add default Packer EC2 template for agent AMI builds"
```

---

## Phase 2: `ciab-packer` Image Builder

### Task 7: Scaffold `ciab-packer` crate

**Files:**
- Create: `crates/ciab-packer/Cargo.toml`
- Create: `crates/ciab-packer/src/lib.rs`
- Modify: `Cargo.toml` (workspace root)

- [ ] **Step 1: Create Cargo.toml**

Create `crates/ciab-packer/Cargo.toml`:

```toml
[package]
name = "ciab-packer"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "HashiCorp Packer image builder for CIAB"

[dependencies]
ciab-core = { workspace = true }
tokio = { workspace = true, features = ["full"] }
async-trait = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
uuid = { workspace = true }
chrono = { workspace = true }
tracing = { workspace = true }
thiserror = { workspace = true }
dashmap = { workspace = true }
```

- [ ] **Step 2: Create lib.rs with module stubs**

Create `crates/ciab-packer/src/lib.rs`:

```rust
pub mod builder;
pub mod template;

pub use builder::PackerImageBuilder;
```

- [ ] **Step 3: Create template module**

Create `crates/ciab-packer/src/template.rs`:

```rust
use ciab_core::error::{CiabError, CiabResult};
use ciab_core::resolve::{parse_source_string, resolve_resource};
use ciab_core::types::config::PackerConfig;
use ciab_core::types::image::TemplateSource;
use std::path::PathBuf;

/// Resolve a template source to its HCL content string.
pub async fn resolve_template(
    source: &Option<TemplateSource>,
    config: &PackerConfig,
) -> CiabResult<String> {
    match source {
        Some(TemplateSource::Inline { content }) => Ok(content.clone()),
        Some(TemplateSource::FilePath { path }) => {
            tokio::fs::read_to_string(path).await.map_err(|e| {
                CiabError::PackerError(format!(
                    "Failed to read template {}: {}",
                    path.display(),
                    e
                ))
            })
        }
        Some(TemplateSource::Url { url }) => {
            let src = parse_source_string(url);
            resolve_resource(&src).await
        }
        Some(TemplateSource::Git { url, path, ref_ }) => {
            let git_uri = format!(
                "git::{}//{}{}",
                url,
                path,
                ref_
                    .as_ref()
                    .map(|r| format!("?ref={}", r))
                    .unwrap_or_default()
            );
            let src = parse_source_string(&git_uri);
            resolve_resource(&src).await
        }
        Some(TemplateSource::Builtin { name }) => {
            let src = parse_source_string(&format!("builtin://{}", name));
            resolve_resource(&src).await
        }
        None => {
            // Use default template from config
            let src = parse_source_string(&config.default_template);
            resolve_resource(&src).await
        }
    }
}

/// Write template content to a temporary file for packer to consume.
pub async fn write_temp_template(content: &str) -> CiabResult<PathBuf> {
    let dir = tempfile::tempdir().map_err(|e| {
        CiabError::PackerError(format!("Failed to create temp dir: {}", e))
    })?;
    let path = dir.into_path().join("template.pkr.hcl");
    tokio::fs::write(&path, content).await.map_err(|e| {
        CiabError::PackerError(format!("Failed to write temp template: {}", e))
    })?;
    Ok(path)
}
```

- [ ] **Step 4: Add to workspace**

In the root `Cargo.toml`, add `"crates/ciab-packer"` to `[workspace.members]` and add to `[workspace.dependencies]`:

```toml
ciab-packer = { path = "crates/ciab-packer" }
```

- [ ] **Step 5: Verify it compiles**

Run: `cargo check -p ciab-packer`
Expected: Compile error because `builder` module doesn't exist yet — that's Task 8.

- [ ] **Step 6: Commit**

```bash
git add crates/ciab-packer/ Cargo.toml
git commit -m "feat(packer): scaffold ciab-packer crate with template resolver"
```

---

### Task 8: Implement `PackerImageBuilder`

**Files:**
- Create: `crates/ciab-packer/src/builder.rs`

- [ ] **Step 1: Implement the builder**

Create `crates/ciab-packer/src/builder.rs`:

```rust
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use dashmap::DashMap;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tracing::{error, info, warn};
use uuid::Uuid;

use ciab_core::error::{CiabError, CiabResult};
use ciab_core::traits::image_builder::ImageBuilder;
use ciab_core::types::config::PackerConfig;
use ciab_core::types::image::{
    BuiltImage, ImageBuildRequest, ImageBuildResult, ImageBuildStatus,
};

use crate::template;

/// Tracks an in-progress or completed build.
struct BuildState {
    status: ImageBuildStatus,
    image_id: Option<String>,
    logs: Vec<String>,
}

/// HashiCorp Packer image builder.
pub struct PackerImageBuilder {
    config: PackerConfig,
    builds: Arc<DashMap<Uuid, BuildState>>,
    images: Arc<DashMap<String, BuiltImage>>,
}

impl PackerImageBuilder {
    pub fn new(config: PackerConfig) -> Self {
        Self {
            config,
            builds: Arc::new(DashMap::new()),
            images: Arc::new(DashMap::new()),
        }
    }

    /// Find the packer binary, installing if configured.
    async fn packer_binary(&self) -> CiabResult<String> {
        // Check if packer is available
        let check = Command::new("which")
            .arg(&self.config.binary)
            .output()
            .await;

        if let Ok(output) = check {
            if output.status.success() {
                return Ok(self.config.binary.clone());
            }
        }

        if self.config.auto_install {
            info!("Packer not found, attempting auto-install");
            self.install_packer().await?;
            Ok(self.config.binary.clone())
        } else {
            Err(CiabError::PackerError(format!(
                "Packer binary '{}' not found on PATH. Set auto_install = true to install automatically.",
                self.config.binary
            )))
        }
    }

    async fn install_packer(&self) -> CiabResult<()> {
        // Install via package manager or direct download
        let output = Command::new("sh")
            .arg("-c")
            .arg(
                "curl -fsSL https://releases.hashicorp.com/packer/1.11.2/packer_1.11.2_linux_amd64.zip -o /tmp/packer.zip \
                 && unzip -o /tmp/packer.zip -d /usr/local/bin/ \
                 && rm /tmp/packer.zip"
            )
            .output()
            .await
            .map_err(|e| CiabError::PackerError(format!("Failed to install packer: {}", e)))?;

        if !output.status.success() {
            return Err(CiabError::PackerError(format!(
                "Packer install failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        info!("Packer installed successfully");
        Ok(())
    }

    /// Merge request variables with config default variables.
    fn merge_variables(
        &self,
        request: &ImageBuildRequest,
    ) -> HashMap<String, String> {
        let mut vars = self.config.variables.clone();
        // Request variables override config defaults
        vars.extend(request.variables.clone());
        vars
    }

    /// Build the packer command with all -var flags.
    fn build_command_args(
        &self,
        binary: &str,
        template_path: &std::path::Path,
        variables: &HashMap<String, String>,
    ) -> Command {
        let mut cmd = Command::new(binary);
        cmd.arg("build");
        cmd.arg("-machine-readable");

        for (key, value) in variables {
            cmd.arg("-var");
            cmd.arg(format!("{}={}", key, value));
        }

        cmd.arg(template_path);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd
    }

    /// Parse machine-readable output to extract artifact ID.
    /// Format: timestamp,target,type,data...
    /// We look for: ...,artifact,0,id,<region>:<ami-id>
    fn parse_artifact_id(line: &str) -> Option<String> {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() >= 5 && parts[2] == "artifact" && parts[4] == "id" {
            // Format is usually "region:ami-id"
            let id_part = parts.get(5).unwrap_or(&"");
            if let Some((_region, ami)) = id_part.split_once(':') {
                return Some(ami.to_string());
            }
            return Some(id_part.to_string());
        }
        None
    }
}

#[async_trait]
impl ImageBuilder for PackerImageBuilder {
    async fn build_image(&self, request: &ImageBuildRequest) -> CiabResult<ImageBuildResult> {
        let build_id = Uuid::new_v4();
        info!(build_id = %build_id, "Starting Packer image build");

        // Initialize build state
        self.builds.insert(
            build_id,
            BuildState {
                status: ImageBuildStatus::Running,
                image_id: None,
                logs: Vec::new(),
            },
        );

        // Resolve template
        let template_content =
            template::resolve_template(&request.template, &self.config).await?;
        let template_path = template::write_temp_template(&template_content).await?;

        // Find packer binary
        let binary = self.packer_binary().await?;

        // Merge variables
        let variables = self.merge_variables(request);

        // Run packer build
        let mut cmd = self.build_command_args(&binary, &template_path, &variables);
        let mut child = cmd.spawn().map_err(|e| {
            CiabError::PackerError(format!("Failed to spawn packer: {}", e))
        })?;

        let stdout = child.stdout.take().ok_or_else(|| {
            CiabError::PackerError("Failed to capture packer stdout".to_string())
        })?;

        let builds = self.builds.clone();
        let images = self.images.clone();
        let build_id_clone = build_id;
        let tags = request.tags.clone();

        // Stream output in background
        tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            let mut artifact_id: Option<String> = None;

            while let Ok(Some(line)) = lines.next_line().await {
                // Try to extract artifact ID
                if let Some(id) = Self::parse_artifact_id(&line) {
                    artifact_id = Some(id);
                }

                // Store log line
                if let Some(mut build) = builds.get_mut(&build_id_clone) {
                    build.logs.push(line);
                }
            }

            // Wait for process to finish
            let status = child.wait().await;
            let success = status.map(|s| s.success()).unwrap_or(false);

            if let Some(mut build) = builds.get_mut(&build_id_clone) {
                if success {
                    build.status = ImageBuildStatus::Succeeded;
                    build.image_id = artifact_id.clone();

                    // Register the built image
                    if let Some(ref image_id) = artifact_id {
                        images.insert(
                            image_id.clone(),
                            BuiltImage {
                                image_id: image_id.clone(),
                                provider: "amazon-ebs".to_string(),
                                region: None,
                                created_at: Utc::now(),
                                tags: tags.clone(),
                            },
                        );
                    }
                } else {
                    let err_msg = build
                        .logs
                        .last()
                        .cloned()
                        .unwrap_or_else(|| "Unknown error".to_string());
                    build.status = ImageBuildStatus::Failed(err_msg);
                }
            }
        });

        // Return immediately with build ID
        Ok(ImageBuildResult {
            build_id,
            status: ImageBuildStatus::Running,
            image_id: None,
            logs: Vec::new(),
        })
    }

    async fn list_images(&self) -> CiabResult<Vec<BuiltImage>> {
        Ok(self.images.iter().map(|r| r.value().clone()).collect())
    }

    async fn delete_image(&self, image_id: &str) -> CiabResult<()> {
        self.images.remove(image_id);
        info!(image_id = image_id, "Removed image from local registry");
        Ok(())
    }

    async fn build_status(&self, build_id: &Uuid) -> CiabResult<ImageBuildStatus> {
        self.builds
            .get(build_id)
            .map(|b| b.status.clone())
            .ok_or_else(|| {
                CiabError::ImageBuildError(format!("Build {} not found", build_id))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_artifact_id_valid() {
        let line = "1234567890,amazon-ebs.agent,artifact,0,id,us-east-1:ami-0123456789abcdef0";
        let result = PackerImageBuilder::parse_artifact_id(line);
        assert_eq!(result, Some("ami-0123456789abcdef0".to_string()));
    }

    #[test]
    fn test_parse_artifact_id_no_match() {
        let line = "1234567890,amazon-ebs.agent,ui,message,Building AMI...";
        let result = PackerImageBuilder::parse_artifact_id(line);
        assert_eq!(result, None);
    }

    #[test]
    fn test_merge_variables() {
        let config = PackerConfig {
            binary: "packer".to_string(),
            auto_install: false,
            template_cache_dir: "/tmp".to_string(),
            template_cache_ttl_secs: 3600,
            default_template: "builtin://default-ec2".to_string(),
            variables: HashMap::from([
                ("region".to_string(), "us-east-1".to_string()),
                ("instance_type".to_string(), "t3.small".to_string()),
            ]),
        };
        let builder = PackerImageBuilder::new(config);

        let request = ImageBuildRequest {
            template: None,
            variables: HashMap::from([
                ("instance_type".to_string(), "t3.large".to_string()),
                ("base_ami".to_string(), "ami-123".to_string()),
            ]),
            agent_provider: None,
            tags: HashMap::new(),
        };

        let merged = builder.merge_variables(&request);
        assert_eq!(merged.get("region"), Some(&"us-east-1".to_string()));
        assert_eq!(merged.get("instance_type"), Some(&"t3.large".to_string())); // overridden
        assert_eq!(merged.get("base_ami"), Some(&"ami-123".to_string()));
    }
}
```

- [ ] **Step 2: Run the tests**

Run: `cargo test -p ciab-packer`
Expected: All 3 tests pass.

- [ ] **Step 3: Verify workspace compiles**

Run: `cargo check --workspace`
Expected: Compiles with no errors.

- [ ] **Step 4: Commit**

```bash
git add crates/ciab-packer/src/builder.rs
git commit -m "feat(packer): implement PackerImageBuilder with template resolution and build streaming"
```

---

## Phase 3: `ciab-sandbox-ec2` AWS EC2 Runtime

### Task 9: Scaffold `ciab-sandbox-ec2` crate

**Files:**
- Create: `crates/ciab-sandbox-ec2/Cargo.toml`
- Create: `crates/ciab-sandbox-ec2/src/lib.rs`
- Modify: `Cargo.toml` (workspace root)

- [ ] **Step 1: Create Cargo.toml**

Create `crates/ciab-sandbox-ec2/Cargo.toml`:

```toml
[package]
name = "ciab-sandbox-ec2"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "AWS EC2 runtime backend for CIAB"

[dependencies]
ciab-core = { workspace = true }
tokio = { workspace = true, features = ["full"] }
async-trait = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
uuid = { workspace = true }
chrono = { workspace = true }
tracing = { workspace = true }
thiserror = { workspace = true }
dashmap = { workspace = true }
aws-config = "1"
aws-sdk-ec2 = "1"
russh = "0.46"
russh-sftp = "2"
russh-keys = "0.46"
ssh-key = { version = "0.6", features = ["ed25519", "rand_core"] }
rand = "0.8"
```

- [ ] **Step 2: Create lib.rs**

Create `crates/ciab-sandbox-ec2/src/lib.rs`:

```rust
pub mod runtime;
pub mod ssh;

pub use runtime::Ec2Runtime;
```

- [ ] **Step 3: Add to workspace**

In the root `Cargo.toml`, add `"crates/ciab-sandbox-ec2"` to `[workspace.members]` and:

```toml
ciab-sandbox-ec2 = { path = "crates/ciab-sandbox-ec2" }
```

- [ ] **Step 4: Commit**

```bash
git add crates/ciab-sandbox-ec2/Cargo.toml crates/ciab-sandbox-ec2/src/lib.rs Cargo.toml
git commit -m "feat(ec2): scaffold ciab-sandbox-ec2 crate"
```

---

### Task 10: Implement SSH client module

**Files:**
- Create: `crates/ciab-sandbox-ec2/src/ssh.rs`

- [ ] **Step 1: Implement the SSH client**

Create `crates/ciab-sandbox-ec2/src/ssh.rs`:

```rust
use std::sync::Arc;

use russh::client::{self, Handler};
use russh::Channel;
use russh_keys::key::KeyPair;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;
use tracing::{debug, error, info};

use ciab_core::error::{CiabError, CiabResult};

/// SSH client handler (minimal — we don't need interactive auth).
#[derive(Clone)]
pub struct SshHandler;

#[async_trait::async_trait]
impl Handler for SshHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &russh_keys::key::PublicKey,
    ) -> Result<bool, Self::Error> {
        // Accept all host keys (sandboxes are ephemeral)
        Ok(true)
    }
}

/// Manages an SSH connection to an EC2 instance.
pub struct SshSession {
    session: client::Handle<SshHandler>,
}

impl SshSession {
    /// Connect to a remote host via SSH using a key pair.
    pub async fn connect(
        host: &str,
        port: u16,
        user: &str,
        key: &KeyPair,
    ) -> CiabResult<Self> {
        info!(host = host, port = port, user = user, "Connecting via SSH");

        let config = Arc::new(client::Config {
            inactivity_timeout: Some(std::time::Duration::from_secs(300)),
            ..Default::default()
        });

        let handler = SshHandler;
        let mut session =
            client::connect(config, (host, port), handler)
                .await
                .map_err(|e| CiabError::SshError(format!("SSH connect failed: {}", e)))?;

        let auth_result = session
            .authenticate_publickey(user, Arc::new(key.clone()))
            .await
            .map_err(|e| CiabError::SshError(format!("SSH auth failed: {}", e)))?;

        if !auth_result {
            return Err(CiabError::SshError("SSH authentication rejected".to_string()));
        }

        debug!("SSH connection established to {}:{}", host, port);
        Ok(Self { session })
    }

    /// Execute a command and return stdout + stderr + exit code.
    pub async fn exec(&self, command: &str) -> CiabResult<(String, String, u32)> {
        let mut channel = self
            .session
            .channel_open_session()
            .await
            .map_err(|e| CiabError::SshError(format!("Failed to open channel: {}", e)))?;

        channel
            .exec(true, command)
            .await
            .map_err(|e| CiabError::SshError(format!("Failed to exec: {}", e)))?;

        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let mut exit_code: u32 = 1;

        loop {
            let msg = channel.wait().await;
            match msg {
                Some(russh::ChannelMsg::Data { data }) => {
                    stdout.extend_from_slice(&data);
                }
                Some(russh::ChannelMsg::ExtendedData { data, ext }) => {
                    if ext == 1 {
                        stderr.extend_from_slice(&data);
                    }
                }
                Some(russh::ChannelMsg::ExitStatus { exit_status }) => {
                    exit_code = exit_status;
                }
                Some(russh::ChannelMsg::Eof) | None => break,
                _ => {}
            }
        }

        Ok((
            String::from_utf8_lossy(&stdout).to_string(),
            String::from_utf8_lossy(&stderr).to_string(),
            exit_code,
        ))
    }

    /// Execute a command and stream stdout lines via a channel.
    pub async fn exec_streaming(
        &self,
        command: &str,
    ) -> CiabResult<(mpsc::Receiver<String>, tokio::task::JoinHandle<CiabResult<u32>>)> {
        let mut channel = self
            .session
            .channel_open_session()
            .await
            .map_err(|e| CiabError::SshError(format!("Failed to open channel: {}", e)))?;

        channel
            .exec(true, command)
            .await
            .map_err(|e| CiabError::SshError(format!("Failed to exec: {}", e)))?;

        let (tx, rx) = mpsc::channel(256);

        let handle = tokio::spawn(async move {
            let mut exit_code: u32 = 1;
            loop {
                let msg = channel.wait().await;
                match msg {
                    Some(russh::ChannelMsg::Data { data }) => {
                        let text = String::from_utf8_lossy(&data).to_string();
                        for line in text.lines() {
                            let _ = tx.send(line.to_string()).await;
                        }
                    }
                    Some(russh::ChannelMsg::ExtendedData { data, ext: 1 }) => {
                        let text = String::from_utf8_lossy(&data).to_string();
                        for line in text.lines() {
                            let _ = tx.send(line.to_string()).await;
                        }
                    }
                    Some(russh::ChannelMsg::ExitStatus { exit_status }) => {
                        exit_code = exit_status;
                    }
                    Some(russh::ChannelMsg::Eof) | None => break,
                    _ => {}
                }
            }
            Ok(exit_code)
        });

        Ok((rx, handle))
    }

    /// Upload file content to the remote host via SFTP.
    pub async fn write_file(&self, path: &str, content: &[u8]) -> CiabResult<()> {
        // Use a shell command to write file content as SFTP requires additional setup
        let encoded = base64::engine::general_purpose::STANDARD.encode(content);
        let cmd = format!(
            "echo '{}' | base64 -d > '{}'",
            encoded,
            path.replace('\'', "'\\''")
        );
        let (_, stderr, code) = self.exec(&cmd).await?;
        if code != 0 {
            return Err(CiabError::SshError(format!(
                "Failed to write file {}: {}",
                path, stderr
            )));
        }
        Ok(())
    }

    /// Read file content from the remote host.
    pub async fn read_file(&self, path: &str) -> CiabResult<Vec<u8>> {
        let cmd = format!("base64 '{}'", path.replace('\'', "'\\''"));
        let (stdout, stderr, code) = self.exec(&cmd).await?;
        if code != 0 {
            return Err(CiabError::SshError(format!(
                "Failed to read file {}: {}",
                path, stderr
            )));
        }
        base64::engine::general_purpose::STANDARD
            .decode(stdout.trim())
            .map_err(|e| CiabError::SshError(format!("Failed to decode file content: {}", e)))
    }

    /// List files at a path.
    pub async fn list_files(&self, path: &str) -> CiabResult<String> {
        let cmd = format!("ls -la '{}'", path.replace('\'', "'\\''"));
        let (stdout, stderr, code) = self.exec(&cmd).await?;
        if code != 0 {
            return Err(CiabError::SshError(format!(
                "Failed to list files at {}: {}",
                path, stderr
            )));
        }
        Ok(stdout)
    }
}

/// Generate an ephemeral Ed25519 SSH key pair.
pub fn generate_keypair() -> CiabResult<(KeyPair, String)> {
    use ssh_key::private::Ed25519Keypair;
    use ssh_key::PrivateKey;

    let ed25519_keypair = Ed25519Keypair::random(&mut rand::thread_rng());
    let private_key = PrivateKey::from(ed25519_keypair);

    let public_key_openssh = private_key
        .public_key()
        .to_openssh()
        .map_err(|e| CiabError::SshError(format!("Failed to encode public key: {}", e)))?;

    let russh_keypair = russh_keys::key::KeyPair::Ed25519(
        russh_keys::key::ed25519::SecretKey::from_bytes(
            private_key
                .key_data()
                .ed25519()
                .ok_or_else(|| CiabError::SshError("Not an Ed25519 key".to_string()))?
                .private
                .as_ref(),
        ),
    );

    Ok((russh_keypair, public_key_openssh))
}
```

Note: Add `base64 = "0.22"` to `crates/ciab-sandbox-ec2/Cargo.toml` dependencies.

- [ ] **Step 2: Add base64 dependency**

In `crates/ciab-sandbox-ec2/Cargo.toml`, add to `[dependencies]`:

```toml
base64 = "0.22"
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo check -p ciab-sandbox-ec2`
Expected: Compile error since `runtime.rs` doesn't exist yet — that's Task 11.

- [ ] **Step 4: Commit**

```bash
git add crates/ciab-sandbox-ec2/src/ssh.rs crates/ciab-sandbox-ec2/Cargo.toml
git commit -m "feat(ec2): implement async SSH client with key generation"
```

---

### Task 11: Implement `Ec2Runtime`

**Files:**
- Create: `crates/ciab-sandbox-ec2/src/runtime.rs`

- [ ] **Step 1: Implement the runtime**

Create `crates/ciab-sandbox-ec2/src/runtime.rs`:

```rust
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;
use aws_sdk_ec2::types::{
    BlockDeviceMapping, EbsBlockDevice, Filter, InstanceStateName, InstanceType, ResourceType,
    Tag, TagSpecification, VolumeType,
};
use aws_sdk_ec2::Client as Ec2Client;
use chrono::Utc;
use dashmap::DashMap;
use russh_keys::key::KeyPair;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::{error, info, warn};
use uuid::Uuid;

use ciab_core::error::{CiabError, CiabResult};
use ciab_core::traits::runtime::SandboxRuntime;
use ciab_core::types::config::Ec2Config;
use ciab_core::types::sandbox::*;

use crate::ssh::{self, SshSession};

/// Per-sandbox state held in memory.
struct InstanceState {
    instance_id: String,
    public_ip: Option<String>,
    keypair: KeyPair,
    ssh_session: Option<SshSession>,
    sandbox_info: SandboxInfo,
}

/// AWS EC2 runtime backend. One EC2 instance per sandbox.
pub struct Ec2Runtime {
    client: Ec2Client,
    config: Ec2Config,
    instances: Arc<DashMap<Uuid, InstanceState>>,
}

impl Ec2Runtime {
    /// Create a new EC2 runtime from config.
    pub async fn new(config: Ec2Config) -> CiabResult<Self> {
        let region = config.region.clone();
        let aws_config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(aws_config::Region::new(region))
            .load()
            .await;

        let client = Ec2Client::new(&aws_config);

        Ok(Self {
            client,
            config,
            instances: Arc::new(DashMap::new()),
        })
    }

    /// Build the user-data script that injects the SSH public key.
    fn build_user_data(&self, public_key: &str) -> String {
        let user = &self.config.ssh_user;
        format!(
            r#"#!/bin/bash
mkdir -p /home/{user}/.ssh
echo '{public_key}' >> /home/{user}/.ssh/authorized_keys
chmod 700 /home/{user}/.ssh
chmod 600 /home/{user}/.ssh/authorized_keys
chown -R {user}:{user} /home/{user}/.ssh
"#
        )
    }

    /// Build EC2 tags from config + sandbox ID.
    fn build_tags(&self, sandbox_id: &Uuid, name: &str) -> Vec<Tag> {
        let mut tags = vec![
            Tag::builder()
                .key("ciab-sandbox-id")
                .value(sandbox_id.to_string())
                .build(),
            Tag::builder()
                .key("ciab-managed")
                .value("true")
                .build(),
            Tag::builder()
                .key("Name")
                .value(format!("ciab-{}", name))
                .build(),
        ];

        for (k, v) in &self.config.tags {
            tags.push(Tag::builder().key(k).value(v).build());
        }

        tags
    }

    /// Wait for instance to be running and SSH to be reachable.
    async fn wait_for_instance(
        &self,
        instance_id: &str,
        sandbox_id: &Uuid,
    ) -> CiabResult<String> {
        let deadline =
            Instant::now() + std::time::Duration::from_secs(self.config.instance_ready_timeout_secs);

        // Wait for running state
        loop {
            if Instant::now() > deadline {
                return Err(CiabError::SandboxTimeout(format!(
                    "Instance {} did not become running within timeout",
                    instance_id
                )));
            }

            let desc = self
                .client
                .describe_instances()
                .instance_ids(instance_id)
                .send()
                .await
                .map_err(|e| CiabError::Ec2Error(format!("DescribeInstances failed: {}", e)))?;

            if let Some(reservation) = desc.reservations().first() {
                if let Some(instance) = reservation.instances().first() {
                    if let Some(state) = instance.state() {
                        if state.name() == Some(&InstanceStateName::Running) {
                            if let Some(ip) = instance.public_ip_address() {
                                return Ok(ip.to_string());
                            }
                        }
                    }
                }
            }

            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    }

    /// Get or create an SSH session for a sandbox.
    async fn get_ssh_session(&self, sandbox_id: &Uuid) -> CiabResult<SshSession> {
        let state = self.instances.get(sandbox_id).ok_or_else(|| {
            CiabError::SandboxNotFound(sandbox_id.to_string())
        })?;

        let ip = state.public_ip.as_ref().ok_or_else(|| {
            CiabError::SshError("Instance has no public IP".to_string())
        })?;

        SshSession::connect(
            ip,
            self.config.ssh_port,
            &self.config.ssh_user,
            &state.keypair,
        )
        .await
    }
}

#[async_trait]
impl SandboxRuntime for Ec2Runtime {
    async fn create_sandbox(&self, spec: &SandboxSpec) -> CiabResult<SandboxInfo> {
        let sandbox_id = Uuid::new_v4();
        let name = spec
            .name
            .clone()
            .unwrap_or_else(|| format!("sandbox-{}", &sandbox_id.to_string()[..8]));

        info!(sandbox_id = %sandbox_id, name = %name, "Creating EC2 sandbox");

        // Generate ephemeral SSH keypair
        let (keypair, public_key) = ssh::generate_keypair()?;

        // Build user-data for SSH key injection
        let user_data = self.build_user_data(&public_key);
        let user_data_b64 =
            base64::engine::general_purpose::STANDARD.encode(user_data.as_bytes());

        // Determine AMI
        let ami = self
            .config
            .default_ami
            .as_ref()
            .ok_or_else(|| {
                CiabError::Ec2Error(
                    "No default_ami configured. Set runtime.ec2.default_ami or build an image with Packer first."
                        .to_string(),
                )
            })?
            .clone();

        // Build tags
        let tags = self.build_tags(&sandbox_id, &name);

        // Launch instance
        let mut run_req = self
            .client
            .run_instances()
            .image_id(&ami)
            .instance_type(InstanceType::from(self.config.instance_type.as_str()))
            .min_count(1)
            .max_count(1)
            .user_data(&user_data_b64)
            .tag_specifications(
                TagSpecification::builder()
                    .resource_type(ResourceType::Instance)
                    .set_tags(Some(tags))
                    .build(),
            )
            .block_device_mappings(
                BlockDeviceMapping::builder()
                    .device_name("/dev/sda1")
                    .ebs(
                        EbsBlockDevice::builder()
                            .volume_size(self.config.root_volume_size_gb as i32)
                            .volume_type(VolumeType::Gp3)
                            .delete_on_termination(true)
                            .build(),
                    )
                    .build(),
            );

        if let Some(ref subnet) = self.config.subnet_id {
            run_req = run_req.subnet_id(subnet);
        }

        for sg in &self.config.security_group_ids {
            run_req = run_req.security_group_ids(sg);
        }

        if let Some(ref profile) = self.config.iam_instance_profile {
            run_req = run_req.iam_instance_profile(
                aws_sdk_ec2::types::IamInstanceProfileSpecification::builder()
                    .name(profile)
                    .build(),
            );
        }

        let result = run_req
            .send()
            .await
            .map_err(|e| CiabError::Ec2Error(format!("RunInstances failed: {}", e)))?;

        let instance = result
            .instances()
            .first()
            .ok_or_else(|| CiabError::Ec2Error("No instance returned".to_string()))?;

        let instance_id = instance
            .instance_id()
            .ok_or_else(|| CiabError::Ec2Error("No instance ID".to_string()))?
            .to_string();

        info!(instance_id = %instance_id, sandbox_id = %sandbox_id, "EC2 instance launched");

        // Wait for instance to be running and get public IP
        let public_ip = self.wait_for_instance(&instance_id, &sandbox_id).await?;

        // Wait a bit for SSH to be ready
        let deadline =
            Instant::now() + std::time::Duration::from_secs(60);
        loop {
            if Instant::now() > deadline {
                warn!("SSH readiness timeout, proceeding anyway");
                break;
            }
            match SshSession::connect(
                &public_ip,
                self.config.ssh_port,
                &self.config.ssh_user,
                &keypair,
            )
            .await
            {
                Ok(_) => {
                    info!("SSH is reachable on {}", public_ip);
                    break;
                }
                Err(_) => {
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
            }
        }

        let now = Utc::now();
        let sandbox_info = SandboxInfo {
            id: sandbox_id,
            name: name.clone(),
            state: SandboxState::Running,
            persistence: spec.persistence.clone(),
            agent_provider: spec.agent_provider.clone(),
            endpoint_url: Some(format!("ssh://{}@{}:{}", self.config.ssh_user, public_ip, self.config.ssh_port)),
            resource_stats: None,
            labels: spec.labels.clone(),
            created_at: now,
            updated_at: now,
            spec: Some(spec.clone()),
        };

        self.instances.insert(
            sandbox_id,
            InstanceState {
                instance_id,
                public_ip: Some(public_ip),
                keypair,
                ssh_session: None,
                sandbox_info: sandbox_info.clone(),
            },
        );

        Ok(sandbox_info)
    }

    async fn get_sandbox(&self, id: &Uuid) -> CiabResult<SandboxInfo> {
        self.instances
            .get(id)
            .map(|s| s.sandbox_info.clone())
            .ok_or_else(|| CiabError::SandboxNotFound(id.to_string()))
    }

    async fn list_sandboxes(
        &self,
        _filters: Option<&SandboxFilters>,
    ) -> CiabResult<Vec<SandboxInfo>> {
        Ok(self
            .instances
            .iter()
            .map(|r| r.value().sandbox_info.clone())
            .collect())
    }

    async fn start_sandbox(&self, id: &Uuid) -> CiabResult<()> {
        let state = self.instances.get(id).ok_or_else(|| {
            CiabError::SandboxNotFound(id.to_string())
        })?;

        self.client
            .start_instances()
            .instance_ids(&state.instance_id)
            .send()
            .await
            .map_err(|e| CiabError::Ec2Error(format!("StartInstances failed: {}", e)))?;

        drop(state);

        if let Some(mut s) = self.instances.get_mut(id) {
            s.sandbox_info.state = SandboxState::Running;
            s.sandbox_info.updated_at = Utc::now();
        }

        Ok(())
    }

    async fn stop_sandbox(&self, id: &Uuid) -> CiabResult<()> {
        let state = self.instances.get(id).ok_or_else(|| {
            CiabError::SandboxNotFound(id.to_string())
        })?;

        self.client
            .stop_instances()
            .instance_ids(&state.instance_id)
            .send()
            .await
            .map_err(|e| CiabError::Ec2Error(format!("StopInstances failed: {}", e)))?;

        drop(state);

        if let Some(mut s) = self.instances.get_mut(id) {
            s.sandbox_info.state = SandboxState::Stopped;
            s.sandbox_info.updated_at = Utc::now();
        }

        Ok(())
    }

    async fn pause_sandbox(&self, _id: &Uuid) -> CiabResult<()> {
        Err(CiabError::Unsupported(
            "EC2 instances do not support pause/resume".to_string(),
        ))
    }

    async fn resume_sandbox(&self, _id: &Uuid) -> CiabResult<()> {
        Err(CiabError::Unsupported(
            "EC2 instances do not support pause/resume".to_string(),
        ))
    }

    async fn terminate_sandbox(&self, id: &Uuid) -> CiabResult<()> {
        let state = self.instances.get(id).ok_or_else(|| {
            CiabError::SandboxNotFound(id.to_string())
        })?;

        info!(instance_id = %state.instance_id, sandbox_id = %id, "Terminating EC2 instance");

        self.client
            .terminate_instances()
            .instance_ids(&state.instance_id)
            .send()
            .await
            .map_err(|e| CiabError::Ec2Error(format!("TerminateInstances failed: {}", e)))?;

        drop(state);
        self.instances.remove(id);

        Ok(())
    }

    async fn exec(&self, id: &Uuid, request: &ExecRequest) -> CiabResult<ExecResult> {
        let session = self.get_ssh_session(id).await?;
        let start = Instant::now();

        // Build command string
        let mut cmd = String::new();
        if let Some(ref workdir) = request.workdir {
            cmd.push_str(&format!("cd '{}' && ", workdir.replace('\'', "'\\''")));
        }
        for (k, v) in &request.env {
            cmd.push_str(&format!("export {}='{}' && ", k, v.replace('\'', "'\\''")));
        }
        cmd.push_str(&request.command);
        if !request.args.is_empty() {
            for arg in &request.args {
                cmd.push_str(&format!(" '{}'", arg.replace('\'', "'\\''")));
            }
        }

        let (stdout, stderr, exit_code) = session.exec(&cmd).await?;

        Ok(ExecResult {
            exit_code: exit_code as i32,
            stdout,
            stderr,
            duration_ms: Some(start.elapsed().as_millis() as u64),
        })
    }

    async fn read_file(&self, id: &Uuid, path: &str) -> CiabResult<Vec<u8>> {
        let session = self.get_ssh_session(id).await?;
        session.read_file(path).await
    }

    async fn write_file(&self, id: &Uuid, path: &str, content: &[u8]) -> CiabResult<()> {
        let session = self.get_ssh_session(id).await?;
        session.write_file(path, content).await
    }

    async fn list_files(&self, id: &Uuid, path: &str) -> CiabResult<Vec<FileInfo>> {
        let session = self.get_ssh_session(id).await?;
        let output = session.list_files(path).await?;

        // Parse ls -la output into FileInfo structs
        let mut files = Vec::new();
        for line in output.lines().skip(1) {
            // Skip "total N" line
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 9 {
                let is_dir = parts[0].starts_with('d');
                let size = parts[4].parse::<u64>().unwrap_or(0);
                let name = parts[8..].join(" ");
                files.push(FileInfo {
                    path: if path.ends_with('/') {
                        format!("{}{}", path, name)
                    } else {
                        format!("{}/{}", path, name)
                    },
                    size,
                    is_dir,
                    mode: Some(parts[0].to_string()),
                    modified_at: None,
                });
            }
        }

        Ok(files)
    }

    async fn get_stats(&self, id: &Uuid) -> CiabResult<ResourceStats> {
        let session = self.get_ssh_session(id).await?;

        // Get CPU and memory stats via SSH
        let (stats_output, _, _) = session
            .exec("echo \"CPU:$(top -bn1 | grep 'Cpu(s)' | awk '{print $2}')\nMEM:$(free -m | awk '/Mem:/{printf \"%s/%s\", $3, $2}')\"")
            .await?;

        let mut cpu_usage = 0.0;
        let mut mem_used = 0;
        let mut mem_total = 0;

        for line in stats_output.lines() {
            if let Some(cpu) = line.strip_prefix("CPU:") {
                cpu_usage = cpu.trim().parse().unwrap_or(0.0);
            }
            if let Some(mem) = line.strip_prefix("MEM:") {
                let parts: Vec<&str> = mem.split('/').collect();
                if parts.len() == 2 {
                    mem_used = parts[0].trim().parse().unwrap_or(0);
                    mem_total = parts[1].trim().parse().unwrap_or(0);
                }
            }
        }

        Ok(ResourceStats {
            cpu_usage_percent: Some(cpu_usage),
            memory_used_mb: Some(mem_used),
            memory_limit_mb: Some(mem_total),
            disk_used_mb: None,
            disk_limit_mb: None,
            network_rx_bytes: None,
            network_tx_bytes: None,
        })
    }

    async fn stream_logs(
        &self,
        id: &Uuid,
        options: &LogOptions,
    ) -> CiabResult<mpsc::Receiver<String>> {
        let session = self.get_ssh_session(id).await?;

        let mut cmd = "tail".to_string();
        if options.follow {
            cmd.push_str(" -f");
        }
        if let Some(n) = options.tail {
            cmd.push_str(&format!(" -n {}", n));
        }
        cmd.push_str(" /var/log/syslog");

        let (rx, _handle) = session.exec_streaming(&cmd).await?;
        Ok(rx)
    }

    async fn kill_exec(&self, id: &Uuid) -> CiabResult<()> {
        // Kill any running commands by sending SIGTERM to all user processes
        let session = self.get_ssh_session(id).await?;
        let _ = session.exec("pkill -u $(whoami) -f ciab || true").await;
        Ok(())
    }
}
```

- [ ] **Step 2: Add base64 import in runtime.rs**

Add to the use statements at the top of `runtime.rs`:

```rust
use base64::Engine as _;
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo check -p ciab-sandbox-ec2`
Expected: Compiles (may have warnings about unused imports — fix any).

Note: The exact `russh` API may need adjustment depending on the version. If compilation fails on specific russh types/methods, check the russh 0.46 API and adjust accordingly. The SSH handler, connect, and channel APIs are the parts most likely to need version-specific tweaks.

- [ ] **Step 4: Commit**

```bash
git add crates/ciab-sandbox-ec2/src/runtime.rs
git commit -m "feat(ec2): implement Ec2Runtime with full SandboxRuntime trait"
```

---

## Phase 4: `ciab` Facade Crate

### Task 12: Scaffold `ciab` facade crate

**Files:**
- Create: `crates/ciab/Cargo.toml`
- Create: `crates/ciab/src/lib.rs`
- Modify: `Cargo.toml` (workspace root)

- [ ] **Step 1: Create Cargo.toml with feature flags**

Create `crates/ciab/Cargo.toml`:

```toml
[package]
name = "ciab"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "CIAB — Manage coding agent sandboxes with a single Rust API"

[features]
default = ["local"]
local = ["dep:ciab-sandbox"]
ec2 = ["dep:ciab-sandbox-ec2"]
kubernetes = ["dep:ciab-sandbox-k8s"]
packer = ["dep:ciab-packer"]
full = ["local", "ec2", "kubernetes", "packer"]

[dependencies]
ciab-core = { workspace = true }
ciab-db = { workspace = true }
ciab-streaming = { workspace = true }
ciab-credentials = { workspace = true }
ciab-provisioning = { workspace = true }

# Feature-gated runtime backends
ciab-sandbox = { workspace = true, optional = true }
ciab-sandbox-ec2 = { workspace = true, optional = true }
ciab-sandbox-k8s = { workspace = true, optional = true }
ciab-packer = { workspace = true, optional = true }

# Agent providers (always available)
ciab-agent-claude = { workspace = true }
ciab-agent-codex = { workspace = true }
ciab-agent-gemini = { workspace = true }
ciab-agent-cursor = { workspace = true }

tokio = { workspace = true }
async-trait = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
uuid = { workspace = true }
chrono = { workspace = true }
tracing = { workspace = true }
thiserror = { workspace = true }
```

- [ ] **Step 2: Create lib.rs with re-exports**

Create `crates/ciab/src/lib.rs`:

```rust
//! # CIAB — Claude-In-A-Box
//!
//! Native Rust library for managing coding agent sandboxes.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use ciab::CiabEngine;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Zero-config: uses embedded defaults, local runtime
//!     let engine = CiabEngine::builder().build().await?;
//!     Ok(())
//! }
//! ```

pub mod engine;

pub use engine::{CiabEngine, CiabEngineBuilder};

// Re-export core types so embedders don't need ciab-core directly.
pub use ciab_core::error::{CiabError, CiabResult};
pub use ciab_core::traits::agent::AgentProvider;
pub use ciab_core::traits::image_builder::ImageBuilder;
pub use ciab_core::traits::runtime::SandboxRuntime;
pub use ciab_core::types::config::AppConfig;
pub use ciab_core::types::image::*;
pub use ciab_core::types::sandbox::*;
pub use ciab_core::types::stream::StreamEvent;

// Re-export runtime constructors (feature-gated).
#[cfg(feature = "local")]
pub use ciab_sandbox::LocalProcessRuntime;

#[cfg(feature = "ec2")]
pub use ciab_sandbox_ec2::Ec2Runtime;

#[cfg(feature = "kubernetes")]
pub use ciab_sandbox_k8s::KubernetesRuntime;

#[cfg(feature = "packer")]
pub use ciab_packer::PackerImageBuilder;

// Re-export agent providers.
pub use ciab_agent_claude::ClaudeCodeProvider;
pub use ciab_agent_codex::CodexProvider;
pub use ciab_agent_gemini::GeminiProvider;
pub use ciab_agent_cursor::CursorProvider;
```

- [ ] **Step 3: Add to workspace**

In root `Cargo.toml`, add `"crates/ciab"` to `[workspace.members]` and:

```toml
ciab = { path = "crates/ciab" }
```

- [ ] **Step 4: Commit**

```bash
git add crates/ciab/Cargo.toml crates/ciab/src/lib.rs Cargo.toml
git commit -m "feat(ciab): scaffold facade crate with feature-gated re-exports"
```

---

### Task 13: Implement `CiabEngine` and `CiabEngineBuilder`

**Files:**
- Create: `crates/ciab/src/engine.rs`

- [ ] **Step 1: Implement the engine**

Create `crates/ciab/src/engine.rs`:

```rust
use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::mpsc;
use tracing::info;
use uuid::Uuid;

use ciab_core::error::{CiabError, CiabResult};
use ciab_core::traits::agent::AgentProvider;
use ciab_core::traits::runtime::SandboxRuntime;
use ciab_core::types::config::AppConfig;
use ciab_core::types::sandbox::*;
use ciab_core::types::stream::StreamEvent;
use ciab_credentials::CredentialStore;
use ciab_db::Database;
use ciab_provisioning::ProvisioningPipeline;
use ciab_streaming::StreamBroker;

#[cfg(feature = "packer")]
use ciab_core::traits::image_builder::ImageBuilder;
#[cfg(feature = "packer")]
use ciab_core::types::image::*;

/// The main CIAB engine. Provides a high-level API for managing agent sandboxes.
pub struct CiabEngine {
    config: Arc<AppConfig>,
    default_runtime: Arc<dyn SandboxRuntime>,
    runtimes: HashMap<String, Arc<dyn SandboxRuntime>>,
    agents: HashMap<String, Arc<dyn AgentProvider>>,
    provisioning: Arc<ProvisioningPipeline>,
    db: Arc<Database>,
    stream_broker: Arc<StreamBroker>,
    credential_store: Arc<CredentialStore>,
    #[cfg(feature = "packer")]
    image_builder: Option<Arc<dyn ImageBuilder>>,
}

impl CiabEngine {
    /// Create a new builder.
    pub fn builder() -> CiabEngineBuilder {
        CiabEngineBuilder::new()
    }

    /// Get a reference to the configuration.
    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    /// Get a named runtime backend.
    pub fn runtime(&self, name: &str) -> CiabResult<Arc<dyn SandboxRuntime>> {
        self.runtimes
            .get(name)
            .cloned()
            .ok_or_else(|| CiabError::RuntimeUnavailable(name.to_string()))
    }

    /// Get a named agent provider.
    pub fn agent(&self, name: &str) -> CiabResult<Arc<dyn AgentProvider>> {
        self.agents
            .get(name)
            .cloned()
            .ok_or_else(|| CiabError::AgentProviderNotFound(name.to_string()))
    }

    /// Get a reference to the database.
    pub fn db(&self) -> &Database {
        &self.db
    }

    /// Get a reference to the stream broker.
    pub fn stream_broker(&self) -> &StreamBroker {
        &self.stream_broker
    }

    /// Get a reference to the credential store.
    pub fn credential_store(&self) -> &CredentialStore {
        &self.credential_store
    }

    /// Get all registered runtimes.
    pub fn runtimes(&self) -> &HashMap<String, Arc<dyn SandboxRuntime>> {
        &self.runtimes
    }

    /// Get all registered agents.
    pub fn agents(&self) -> &HashMap<String, Arc<dyn AgentProvider>> {
        &self.agents
    }

    /// Get a reference to the provisioning pipeline.
    pub fn provisioning(&self) -> &ProvisioningPipeline {
        &self.provisioning
    }

    // --- High-level sandbox operations ---

    /// Create a new sandbox. Selects the runtime based on `spec.runtime_backend`
    /// or falls back to the default runtime.
    pub async fn create_sandbox(&self, spec: &SandboxSpec) -> CiabResult<SandboxInfo> {
        let runtime = if let Some(ref backend) = spec.runtime_backend {
            self.runtime(backend)?
        } else {
            self.default_runtime.clone()
        };

        runtime.create_sandbox(spec).await
    }

    /// Get sandbox info by ID. Checks all runtimes.
    pub async fn get_sandbox(&self, id: Uuid) -> CiabResult<SandboxInfo> {
        // Try default runtime first, then all others
        if let Ok(info) = self.default_runtime.get_sandbox(&id).await {
            return Ok(info);
        }
        for (_name, rt) in &self.runtimes {
            if let Ok(info) = rt.get_sandbox(&id).await {
                return Ok(info);
            }
        }
        Err(CiabError::SandboxNotFound(id.to_string()))
    }

    /// List all sandboxes across all runtimes.
    pub async fn list_sandboxes(
        &self,
        filters: Option<&SandboxFilters>,
    ) -> CiabResult<Vec<SandboxInfo>> {
        let mut all = Vec::new();
        for (_name, rt) in &self.runtimes {
            if let Ok(sandboxes) = rt.list_sandboxes(filters).await {
                all.extend(sandboxes);
            }
        }
        Ok(all)
    }

    /// Start a sandbox.
    pub async fn start_sandbox(&self, id: Uuid) -> CiabResult<()> {
        let info = self.get_sandbox(id).await?;
        let runtime = if let Some(ref spec) = info.spec {
            if let Some(ref backend) = spec.runtime_backend {
                self.runtime(backend)?
            } else {
                self.default_runtime.clone()
            }
        } else {
            self.default_runtime.clone()
        };
        runtime.start_sandbox(&id).await
    }

    /// Stop a sandbox.
    pub async fn stop_sandbox(&self, id: Uuid) -> CiabResult<()> {
        let info = self.get_sandbox(id).await?;
        let runtime = if let Some(ref spec) = info.spec {
            if let Some(ref backend) = spec.runtime_backend {
                self.runtime(backend)?
            } else {
                self.default_runtime.clone()
            }
        } else {
            self.default_runtime.clone()
        };
        runtime.stop_sandbox(&id).await
    }

    /// Terminate a sandbox.
    pub async fn terminate_sandbox(&self, id: Uuid) -> CiabResult<()> {
        let info = self.get_sandbox(id).await?;
        let runtime = if let Some(ref spec) = info.spec {
            if let Some(ref backend) = spec.runtime_backend {
                self.runtime(backend)?
            } else {
                self.default_runtime.clone()
            }
        } else {
            self.default_runtime.clone()
        };
        runtime.terminate_sandbox(&id).await
    }

    // --- Execution ---

    /// Execute a command in a sandbox.
    pub async fn exec(&self, sandbox_id: Uuid, request: &ExecRequest) -> CiabResult<ExecResult> {
        let info = self.get_sandbox(sandbox_id).await?;
        let runtime = if let Some(ref spec) = info.spec {
            if let Some(ref backend) = spec.runtime_backend {
                self.runtime(backend)?
            } else {
                self.default_runtime.clone()
            }
        } else {
            self.default_runtime.clone()
        };
        runtime.exec(&sandbox_id, request).await
    }

    // --- File operations ---

    /// Read a file from a sandbox.
    pub async fn read_file(&self, sandbox_id: Uuid, path: &str) -> CiabResult<Vec<u8>> {
        self.default_runtime.read_file(&sandbox_id, path).await
    }

    /// Write a file to a sandbox.
    pub async fn write_file(
        &self,
        sandbox_id: Uuid,
        path: &str,
        content: &[u8],
    ) -> CiabResult<()> {
        self.default_runtime
            .write_file(&sandbox_id, path, content)
            .await
    }

    /// List files in a sandbox.
    pub async fn list_files(&self, sandbox_id: Uuid, path: &str) -> CiabResult<Vec<FileInfo>> {
        self.default_runtime.list_files(&sandbox_id, path).await
    }

    // --- Image building (feature-gated) ---

    /// Build a machine image (requires `packer` feature).
    #[cfg(feature = "packer")]
    pub async fn build_image(&self, request: &ImageBuildRequest) -> CiabResult<ImageBuildResult> {
        self.image_builder
            .as_ref()
            .ok_or_else(|| {
                CiabError::ImageBuildError("No image builder configured".to_string())
            })?
            .build_image(request)
            .await
    }

    /// List built images (requires `packer` feature).
    #[cfg(feature = "packer")]
    pub async fn list_images(&self) -> CiabResult<Vec<BuiltImage>> {
        self.image_builder
            .as_ref()
            .ok_or_else(|| {
                CiabError::ImageBuildError("No image builder configured".to_string())
            })?
            .list_images()
            .await
    }

    /// Provision a sandbox using the full provisioning pipeline.
    /// This runs the 9-step provisioning process: validate, create, start,
    /// mount dirs, inject credentials, clone repos, setup AgentFS, run scripts, start agent.
    pub async fn provision_sandbox(
        &self,
        spec: &SandboxSpec,
        agent: &dyn AgentProvider,
    ) -> CiabResult<(SandboxInfo, mpsc::Receiver<StreamEvent>)> {
        let (tx, rx) = mpsc::channel(256);
        let info = self.provisioning.provision(spec, agent, tx).await?;
        Ok((info, rx))
    }
}

/// Builder for constructing a `CiabEngine`.
pub struct CiabEngineBuilder {
    config: Option<AppConfig>,
    config_source: Option<String>,
    runtimes: HashMap<String, Arc<dyn SandboxRuntime>>,
    agents: HashMap<String, Arc<dyn AgentProvider>>,
    db_url: Option<String>,
    #[cfg(feature = "packer")]
    image_builder: Option<Arc<dyn ImageBuilder>>,
}

impl CiabEngineBuilder {
    fn new() -> Self {
        Self {
            config: None,
            config_source: None,
            runtimes: HashMap::new(),
            agents: HashMap::new(),
            db_url: None,
            #[cfg(feature = "packer")]
            image_builder: None,
        }
    }

    /// Set config from an in-memory AppConfig.
    pub fn config(mut self, config: AppConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// Load config from a file path.
    pub fn config_from_file(mut self, path: &str) -> Self {
        self.config_source = Some(path.to_string());
        self
    }

    /// Load config from a URL.
    pub fn config_from_url(mut self, url: &str) -> Self {
        self.config_source = Some(url.to_string());
        self
    }

    /// Use the embedded default config.
    pub fn config_default(mut self) -> Self {
        self.config = Some(
            AppConfig::load_default().expect("Embedded default config should always parse"),
        );
        self
    }

    /// Register a named runtime backend.
    pub fn runtime(mut self, name: &str, runtime: Arc<dyn SandboxRuntime>) -> Self {
        self.runtimes.insert(name.to_string(), runtime);
        self
    }

    /// Register a named agent provider.
    pub fn agent(mut self, name: &str, agent: Arc<dyn AgentProvider>) -> Self {
        self.agents.insert(name.to_string(), agent);
        self
    }

    /// Set the database URL (default: "sqlite:ciab.db").
    pub fn database_url(mut self, url: &str) -> Self {
        self.db_url = Some(url.to_string());
        self
    }

    /// Set an image builder (requires `packer` feature).
    #[cfg(feature = "packer")]
    pub fn image_builder(mut self, builder: Arc<dyn ImageBuilder>) -> Self {
        self.image_builder = Some(builder);
        self
    }

    /// Build the engine.
    pub async fn build(self) -> CiabResult<CiabEngine> {
        // 1. Resolve config
        let config = if let Some(cfg) = self.config {
            cfg
        } else {
            AppConfig::load(self.config_source.as_deref()).await?
        };
        let config = Arc::new(config);

        // 2. Initialize database
        let db_url = self
            .db_url
            .unwrap_or_else(|| "sqlite:ciab.db?mode=rwc".to_string());
        let db = Arc::new(
            Database::new(&db_url)
                .await
                .map_err(|e| CiabError::Internal(format!("Database init failed: {}", e)))?,
        );

        // 3. Initialize stream broker
        let stream_broker = Arc::new(StreamBroker::new(
            config.streaming.buffer_size,
        ));

        // 4. Initialize credential store
        let encryption_key = std::env::var(&config.credentials.encryption_key_env)
            .unwrap_or_else(|_| {
                // Generate a random key for development/testing
                use rand::Rng;
                let key: [u8; 32] = rand::thread_rng().gen();
                hex::encode(key)
            });
        let credential_store = Arc::new(
            CredentialStore::new(db.clone(), &encryption_key)
                .map_err(|e| CiabError::Internal(format!("Credential store init failed: {}", e)))?,
        );

        // 5. Set up runtimes
        let mut runtimes = self.runtimes;

        // Auto-register local runtime if not manually provided
        #[cfg(feature = "local")]
        if !runtimes.contains_key("local") {
            let workdir = config
                .runtime
                .local_workdir
                .clone()
                .unwrap_or_else(|| "/tmp/ciab-sandboxes".to_string());
            let max_procs = config.runtime.local_max_processes.unwrap_or(10);
            let local_rt = ciab_sandbox::LocalProcessRuntime::new(&workdir, max_procs);
            runtimes.insert("local".to_string(), Arc::new(local_rt));
        }

        // Auto-register EC2 runtime if configured
        #[cfg(feature = "ec2")]
        if !runtimes.contains_key("ec2") {
            if let Some(ref ec2_config) = config.runtime.ec2 {
                let ec2_rt = ciab_sandbox_ec2::Ec2Runtime::new(ec2_config.clone()).await?;
                runtimes.insert("ec2".to_string(), Arc::new(ec2_rt));
            }
        }

        // Auto-register K8s runtime if configured
        #[cfg(feature = "kubernetes")]
        if !runtimes.contains_key("kubernetes") {
            if let Some(ref k8s_config) = config.runtime.kubernetes {
                let k8s_rt =
                    ciab_sandbox_k8s::KubernetesRuntime::new(k8s_config.clone().into()).await
                        .map_err(|e| CiabError::Internal(format!("K8s init failed: {}", e)))?;
                runtimes.insert("kubernetes".to_string(), Arc::new(k8s_rt));
            }
        }

        // Select default runtime
        let default_backend = &config.runtime.backend;
        let default_runtime = runtimes
            .get(default_backend.as_str())
            .or_else(|| runtimes.get("local"))
            .cloned()
            .ok_or_else(|| {
                CiabError::RuntimeUnavailable(format!(
                    "No runtime available for backend '{}'",
                    default_backend
                ))
            })?;

        // 6. Set up agents
        let mut agents = self.agents;
        if agents.is_empty() {
            // Auto-register from config
            for (name, provider_config) in &config.agents.providers {
                if !provider_config.enabled {
                    continue;
                }
                let agent: Arc<dyn AgentProvider> = match name.as_str() {
                    "claude-code" => Arc::new(ciab_agent_claude::ClaudeCodeProvider::new()),
                    "codex" => Arc::new(ciab_agent_codex::CodexProvider::new()),
                    "gemini" => Arc::new(ciab_agent_gemini::GeminiProvider::new()),
                    "cursor" => Arc::new(ciab_agent_cursor::CursorProvider::new()),
                    _ => continue,
                };
                agents.insert(name.clone(), agent);
            }
        }

        // 7. Initialize provisioning pipeline
        let provisioning = Arc::new(ProvisioningPipeline::new(
            default_runtime.clone(),
            credential_store.clone(),
            config.provisioning.timeout_secs,
        ));

        // 8. Image builder
        #[cfg(feature = "packer")]
        let image_builder = if self.image_builder.is_some() {
            self.image_builder
        } else if let Some(ref packer_config) = config.runtime.packer {
            Some(Arc::new(ciab_packer::PackerImageBuilder::new(
                packer_config.clone(),
            )) as Arc<dyn ImageBuilder>)
        } else {
            None
        };

        info!(
            backend = %default_backend,
            runtimes = ?runtimes.keys().collect::<Vec<_>>(),
            agents = ?agents.keys().collect::<Vec<_>>(),
            "CiabEngine initialized"
        );

        Ok(CiabEngine {
            config,
            default_runtime,
            runtimes,
            agents,
            provisioning,
            db,
            stream_broker,
            credential_store,
            #[cfg(feature = "packer")]
            image_builder,
        })
    }
}
```

- [ ] **Step 2: Add hex and rand dependencies**

In `crates/ciab/Cargo.toml`, add to `[dependencies]`:

```toml
hex = "0.4"
rand = "0.8"
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo check -p ciab`
Expected: Compiles with no errors.

- [ ] **Step 4: Run workspace check**

Run: `cargo check --workspace`
Expected: Full workspace compiles.

- [ ] **Step 5: Commit**

```bash
git add crates/ciab/src/engine.rs crates/ciab/Cargo.toml
git commit -m "feat(ciab): implement CiabEngine builder with config resolution and auto-wiring"
```

---

### Task 14: Wire CLI to use `CiabEngine`

**Files:**
- Modify: `crates/ciab-cli/Cargo.toml`
- Modify: `crates/ciab-cli/src/commands/server.rs`

- [ ] **Step 1: Add ciab facade dependency to CLI**

In `crates/ciab-cli/Cargo.toml`, add:

```toml
ciab = { workspace = true, features = ["full"] }
```

- [ ] **Step 2: Update config loading in server.rs**

In `crates/ciab-cli/src/commands/server.rs`, find the config loading section (around lines 18-24). The current code reads and parses TOML manually. Replace the config loading with:

Find the line where config is loaded (it reads the file and calls `toml::from_str`). Replace that block with:

```rust
    let app_config = ciab_core::types::config::AppConfig::load(
        args.config.as_deref()
    ).await.map_err(|e| anyhow::anyhow!("Failed to load config: {}", e))?;
```

This enables the full resolution chain: explicit path → CIAB_CONFIG env → ./config.toml → ~/.config/ciab/config.toml → embedded default.

- [ ] **Step 3: Verify it compiles**

Run: `cargo check -p ciab-cli`
Expected: Compiles with no errors.

- [ ] **Step 4: Commit**

```bash
git add crates/ciab-cli/Cargo.toml crates/ciab-cli/src/commands/server.rs
git commit -m "feat(cli): use config resolution chain for zero-config startup"
```

---

### Task 15: Add image builder API routes

**Files:**
- Create: `crates/ciab-api/src/routes/images.rs`
- Modify: `crates/ciab-api/src/routes/mod.rs`
- Modify: `crates/ciab-api/src/router.rs`
- Modify: `crates/ciab-api/src/state.rs`
- Modify: `crates/ciab-api/Cargo.toml`

- [ ] **Step 1: Add image builder to AppState**

In `crates/ciab-api/src/state.rs`, add to the `AppState` struct, after the existing fields:

```rust
    pub image_builder: Option<Arc<dyn ciab_core::traits::image_builder::ImageBuilder>>,
```

- [ ] **Step 2: Create the images route handler**

Create `crates/ciab-api/src/routes/images.rs`:

```rust
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use tracing::info;

use ciab_core::types::image::ImageBuildRequest;

use crate::state::AppState;

/// POST /api/v1/images/build — Start building an image.
pub async fn build_image(
    State(state): State<AppState>,
    Json(request): Json<ImageBuildRequest>,
) -> Result<impl IntoResponse, ciab_core::error::CiabError> {
    let builder = state.image_builder.as_ref().ok_or_else(|| {
        ciab_core::error::CiabError::ImageBuildError(
            "No image builder configured. Set [packer] in config.toml.".to_string(),
        )
    })?;

    info!("Starting image build");
    let result = builder.build_image(&request).await?;
    Ok((StatusCode::ACCEPTED, Json(result)))
}

/// GET /api/v1/images — List built images.
pub async fn list_images(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ciab_core::error::CiabError> {
    let builder = state.image_builder.as_ref().ok_or_else(|| {
        ciab_core::error::CiabError::ImageBuildError(
            "No image builder configured.".to_string(),
        )
    })?;

    let images = builder.list_images().await?;
    Ok(Json(images))
}

/// GET /api/v1/images/builds/{build_id} — Check build status.
pub async fn get_build_status(
    State(state): State<AppState>,
    axum::extract::Path(build_id): axum::extract::Path<uuid::Uuid>,
) -> Result<impl IntoResponse, ciab_core::error::CiabError> {
    let builder = state.image_builder.as_ref().ok_or_else(|| {
        ciab_core::error::CiabError::ImageBuildError(
            "No image builder configured.".to_string(),
        )
    })?;

    let status = builder.build_status(&build_id).await?;
    Ok(Json(status))
}

/// DELETE /api/v1/images/{image_id} — Delete a built image.
pub async fn delete_image(
    State(state): State<AppState>,
    axum::extract::Path(image_id): axum::extract::Path<String>,
) -> Result<impl IntoResponse, ciab_core::error::CiabError> {
    let builder = state.image_builder.as_ref().ok_or_else(|| {
        ciab_core::error::CiabError::ImageBuildError(
            "No image builder configured.".to_string(),
        )
    })?;

    builder.delete_image(&image_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
```

- [ ] **Step 3: Register the module**

In `crates/ciab-api/src/routes/mod.rs`, add:

```rust
pub mod images;
```

- [ ] **Step 4: Add routes to router**

In `crates/ciab-api/src/router.rs`, find the authenticated API routes section. Add the images routes after the existing route groups (e.g., after the agents section):

```rust
        // Images (Packer image builder)
        .route("/images/build", post(routes::images::build_image))
        .route("/images", get(routes::images::list_images))
        .route("/images/builds/{build_id}", get(routes::images::get_build_status))
        .route("/images/{image_id}", delete(routes::images::delete_image))
```

- [ ] **Step 5: Verify it compiles**

Run: `cargo check -p ciab-api`
Expected: Compiles with no errors.

- [ ] **Step 6: Commit**

```bash
git add crates/ciab-api/src/routes/images.rs crates/ciab-api/src/routes/mod.rs crates/ciab-api/src/router.rs crates/ciab-api/src/state.rs
git commit -m "feat(api): add image builder REST endpoints for Packer builds"
```

---

### Task 16: Add `ciab image` CLI commands

**Files:**
- Create: `crates/ciab-cli/src/commands/image.rs`
- Modify: `crates/ciab-cli/src/commands/mod.rs`
- Modify: `crates/ciab-cli/src/main.rs` (or wherever CLI commands are registered)

- [ ] **Step 1: Create image subcommand**

Create `crates/ciab-cli/src/commands/image.rs`:

```rust
use clap::Subcommand;
use comfy_table::{Cell, Table};

#[derive(Subcommand)]
pub enum ImageCommand {
    /// Build a machine image using Packer
    Build {
        /// Packer template source (file path, URL, or git:: URI)
        #[arg(short, long)]
        template: Option<String>,

        /// Packer variable in key=value format (can be repeated)
        #[arg(short, long)]
        var: Vec<String>,

        /// Agent provider to pre-install
        #[arg(long)]
        agent: Option<String>,
    },

    /// List built images
    List,

    /// Check build status
    Status {
        /// Build ID
        build_id: uuid::Uuid,
    },

    /// Delete a built image
    Delete {
        /// Image ID (e.g., ami-xxxxx)
        image_id: String,
    },
}

pub async fn execute(cmd: ImageCommand, server_url: &str) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let base = format!("{}/api/v1", server_url.trim_end_matches('/'));

    match cmd {
        ImageCommand::Build {
            template,
            var,
            agent,
        } => {
            let mut variables = std::collections::HashMap::new();
            for v in &var {
                if let Some((k, val)) = v.split_once('=') {
                    variables.insert(k.to_string(), val.to_string());
                }
            }

            let template_source = template.map(|t| {
                let src = ciab_core::resolve::parse_source_string(&t);
                match src {
                    ciab_core::resolve::ResourceSource::FilePath(p) => {
                        ciab_core::types::image::TemplateSource::FilePath { path: p }
                    }
                    ciab_core::resolve::ResourceSource::Url(u) => {
                        ciab_core::types::image::TemplateSource::Url { url: u }
                    }
                    ciab_core::resolve::ResourceSource::Git { url, path, ref_ } => {
                        ciab_core::types::image::TemplateSource::Git { url, path, ref_ }
                    }
                    ciab_core::resolve::ResourceSource::Builtin(n) => {
                        ciab_core::types::image::TemplateSource::Builtin { name: n }
                    }
                }
            });

            let request = ciab_core::types::image::ImageBuildRequest {
                template: template_source,
                variables,
                agent_provider: agent,
                tags: std::collections::HashMap::new(),
            };

            let resp = client
                .post(format!("{}/images/build", base))
                .json(&request)
                .send()
                .await?;

            if resp.status().is_success() {
                let result: ciab_core::types::image::ImageBuildResult = resp.json().await?;
                println!("Build started: {}", result.build_id);
                println!("Status: {:?}", result.status);
            } else {
                let text = resp.text().await?;
                eprintln!("Error: {}", text);
            }
        }

        ImageCommand::List => {
            let resp = client
                .get(format!("{}/images", base))
                .send()
                .await?;

            if resp.status().is_success() {
                let images: Vec<ciab_core::types::image::BuiltImage> = resp.json().await?;

                if images.is_empty() {
                    println!("No images found.");
                    return Ok(());
                }

                let mut table = Table::new();
                table.set_header(vec!["Image ID", "Provider", "Region", "Created"]);
                for img in &images {
                    table.add_row(vec![
                        Cell::new(&img.image_id),
                        Cell::new(&img.provider),
                        Cell::new(img.region.as_deref().unwrap_or("-")),
                        Cell::new(img.created_at.format("%Y-%m-%d %H:%M").to_string()),
                    ]);
                }
                println!("{}", table);
            } else {
                let text = resp.text().await?;
                eprintln!("Error: {}", text);
            }
        }

        ImageCommand::Status { build_id } => {
            let resp = client
                .get(format!("{}/images/builds/{}", base, build_id))
                .send()
                .await?;

            if resp.status().is_success() {
                let status: ciab_core::types::image::ImageBuildStatus = resp.json().await?;
                println!("Build {}: {:?}", build_id, status);
            } else {
                let text = resp.text().await?;
                eprintln!("Error: {}", text);
            }
        }

        ImageCommand::Delete { image_id } => {
            let resp = client
                .delete(format!("{}/images/{}", base, image_id))
                .send()
                .await?;

            if resp.status().is_success() {
                println!("Image {} deleted.", image_id);
            } else {
                let text = resp.text().await?;
                eprintln!("Error: {}", text);
            }
        }
    }

    Ok(())
}
```

- [ ] **Step 2: Register image command in CLI**

In `crates/ciab-cli/src/commands/mod.rs`, add:

```rust
pub mod image;
```

Then in the main CLI enum (likely in `main.rs` or `commands/mod.rs`), add the `Image` variant:

```rust
    /// Manage machine images (Packer builds)
    Image {
        #[command(subcommand)]
        command: image::ImageCommand,
    },
```

And in the match block where commands are dispatched:

```rust
    Commands::Image { command } => image::execute(command, &server_url).await?,
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo check -p ciab-cli`
Expected: Compiles with no errors.

- [ ] **Step 4: Commit**

```bash
git add crates/ciab-cli/src/commands/image.rs crates/ciab-cli/src/commands/mod.rs crates/ciab-cli/src/main.rs
git commit -m "feat(cli): add ciab image build/list/status/delete commands"
```

---

### Task 17: Update config.example.toml with EC2 and Packer sections

**Files:**
- Modify: `config.example.toml`

- [ ] **Step 1: Add EC2 and Packer config sections**

In `config.example.toml`, find the Kubernetes config section (around line 35-68) and add after it:

```toml

# For AWS EC2 backend:
# backend = "ec2"
#
# [runtime.ec2]
# region = "us-east-1"
# default_ami = "ami-0abcdef1234567890"    # Ubuntu 22.04 or Packer-built AMI
# instance_type = "t3.medium"
# # subnet_id = "subnet-xxxxx"             # Uses default VPC if omitted
# security_group_ids = ["sg-xxxxx"]         # Must allow SSH inbound from CIAB host
# ssh_user = "ubuntu"
# ssh_port = 22
# # iam_instance_profile = ""              # Optional IAM role for the instance
# root_volume_size_gb = 20
# instance_ready_timeout_secs = 180
#
# [runtime.ec2.tags]
# Environment = "ciab"
# ManagedBy = "ciab"

# --- Packer Image Builder ---
# Build machine images with HashiCorp Packer. Images can be used by EC2 or other runtimes.
# Packer templates can be loaded from: local files, HTTP URLs, or Git repositories.

# [packer]
# binary = "packer"
# auto_install = true
# template_cache_dir = "/tmp/ciab-packer-cache"
# template_cache_ttl_secs = 3600
#
# # Default template — used when no template is specified in a build request
# # Built-in:
# default_template = "builtin://default-ec2"
# # Or from Git:
# # default_template = "git::https://github.com/org/templates.git//agent.pkr.hcl?ref=main"
# # Or from URL:
# # default_template = "https://example.com/packer/agent.pkr.hcl"
# # Or local file:
# # default_template = "/path/to/template.pkr.hcl"
#
# [packer.variables]
# region = "us-east-1"
# base_ami = "ami-0abcdef1234567890"       # Ubuntu 22.04 base
# instance_type = "t3.medium"
```

- [ ] **Step 2: Commit**

```bash
git add config.example.toml
git commit -m "docs: add EC2 runtime and Packer image builder sections to config.example.toml"
```

---

### Task 18: Final workspace verification

- [ ] **Step 1: Full workspace build**

Run: `cargo build --workspace`
Expected: Builds with no errors.

- [ ] **Step 2: Run all tests**

Run: `cargo test --workspace`
Expected: All tests pass.

- [ ] **Step 3: Run clippy**

Run: `cargo clippy --workspace -- -D warnings`
Expected: No warnings.

- [ ] **Step 4: Fix any issues found in steps 1-3**

Address any compilation errors, test failures, or clippy warnings. Common issues to expect:
- Unused imports (remove them)
- Missing trait method implementations (check exact signatures match)
- russh API differences (adjust to actual 0.46 API if different from what was written)
- Feature flag conditional compilation issues

- [ ] **Step 5: Final commit if any fixes were needed**

```bash
git add -A
git commit -m "fix: address compilation and clippy issues across workspace"
```
