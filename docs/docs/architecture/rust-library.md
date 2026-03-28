# Rust Library

Embed CIAB in any Rust application using the `ciab` library crate. The library provides a high-level `CiabEngine` API that wraps all CIAB functionality behind a single entry point.

## Overview

The `ciab` crate is a facade that re-exports types from the internal workspace crates and provides `CiabEngine` — a batteries-included entry point for creating and managing sandboxes programmatically.

```toml
[dependencies]
ciab = "0.1"
```

## Feature Flags

The library uses feature flags to control which runtime backends are compiled in:

| Feature | Default | Description |
|---------|---------|-------------|
| `local` | Yes | Local process runtime (no Docker needed) |
| `ec2` | No | AWS EC2 runtime backend |
| `kubernetes` | No | Kubernetes runtime backend (Kata Containers support) |
| `packer` | No | HashiCorp Packer image builder |
| `full` | No | Enables all features |

```toml
# Just local runtime (default)
ciab = "0.1"

# EC2 + Packer support
ciab = { version = "0.1", features = ["ec2", "packer"] }

# Everything
ciab = { version = "0.1", features = ["full"] }
```

## CiabEngine

`CiabEngine` is the main entry point. It owns the database, runtime, streaming broker, and credential store.

### Builder Pattern

```rust
use ciab::{CiabEngine, CiabEngineBuilder};

// Zero-config: uses the config resolution chain (see below)
let engine = CiabEngine::builder().build().await?;

// With explicit config file
let engine = CiabEngine::builder()
    .config_from_file("./my-config.toml")
    .build()
    .await?;

// With embedded defaults (no file needed)
let engine = CiabEngine::builder()
    .config_default()
    .build()
    .await?;

// With a custom database path
let engine = CiabEngine::builder()
    .database_url("sqlite:/var/lib/ciab/data.db?mode=rwc")
    .build()
    .await?;
```

### Sandbox Lifecycle

```rust
use std::collections::HashMap;
use ciab::{CiabEngine, SandboxSpec};

let engine = CiabEngine::builder().build().await?;

// Create a sandbox spec from JSON (easiest way to set only what you need)
let spec: SandboxSpec = serde_json::from_value(serde_json::json!({
    "agent_provider": "claude-code",
    "env_vars": {
        "ANTHROPIC_API_KEY": std::env::var("ANTHROPIC_API_KEY")?
    }
}))?;

let sandbox = engine.create_sandbox(&spec).await?;
println!("Sandbox {} is {:?}", sandbox.id, sandbox.state);

// Execute a command
let exec_req: ciab::ExecRequest = serde_json::from_value(serde_json::json!({
    "command": ["cargo", "test"]
}))?;
let result = engine.exec(&sandbox.id, &exec_req).await?;
println!("Exit code: {}", result.exit_code);
println!("{}", result.stdout);

// File operations
let files = engine.list_files(&sandbox.id, "/workspace").await?;
let content = engine.read_file(&sandbox.id, "/workspace/README.md").await?;
engine.write_file(&sandbox.id, "/workspace/data.json", b"{}").await?;

// Clean up
engine.stop_sandbox(&sandbox.id).await?;
engine.terminate_sandbox(&sandbox.id).await?;
```

### Image Building (requires `packer` feature)

```rust
#[cfg(feature = "packer")]
{
    let request: ciab::ImageBuildRequest = serde_json::from_value(serde_json::json!({
        "agent_provider": "claude-code",
        "region": "us-east-1"
    }))?;
    let result = engine.build_image(&request).await?;
    println!("AMI: {:?}", result.ami_id);
}
```

### Provisioning with Streaming

```rust
use tokio::sync::mpsc;

let (tx, mut rx) = mpsc::channel(32);
let agent = engine.agent("claude-code").unwrap();

// Provision in background, receive SSE events as they happen
tokio::spawn(async move {
    while let Some(event) = rx.recv().await {
        println!("Step: {:?}", event);
    }
});

let sandbox = engine.provision_sandbox(&spec, agent.as_ref(), tx).await?;
```

## Re-exports

The `ciab` crate re-exports commonly used types so you don't need to depend on internal crates directly:

**Core types:**

- `ciab::SandboxSpec`, `ciab::SandboxInfo`, `ciab::SandboxState`
- `ciab::ExecRequest`, `ciab::ExecResult`, `ciab::FileInfo`
- `ciab::StreamEvent`
- `ciab::AppConfig`, `ciab::CiabError`, `ciab::CiabResult`

**Traits:**

- `ciab::AgentProvider`, `ciab::SandboxRuntime`, `ciab::ImageBuilder`

**Runtime backends (feature-gated):**

- `ciab::LocalProcessRuntime` (requires `local`)
- `ciab::Ec2Runtime` (requires `ec2`)
- `ciab::KubernetesRuntime` (requires `kubernetes`)
- `ciab::PackerImageBuilder` (requires `packer`)

**Agent providers (always available):**

- `ciab::ClaudeCodeProvider`, `ciab::CodexProvider`, `ciab::GeminiProvider`, `ciab::CursorProvider`

## Config Resolution Chain

When `CiabEngine::builder().build()` is called without an explicit config, CIAB resolves configuration through a 5-step chain. Each step overrides values from the previous:

1. **Built-in defaults** — Sensible defaults for all fields (local runtime, port 9090, etc.)
2. **`./config.toml`** — Config file in the current working directory
3. **`~/.config/ciab/config.toml`** — User-level config file
4. **Environment variables** — `CIAB_PORT`, `CIAB_RUNTIME_BACKEND`, etc. (see [Environment Variables](../configuration/environment-variables.md))
5. **Builder overrides** — Values set explicitly on `CiabEngineBuilder`

This means CIAB works with zero configuration out of the box — just `CiabEngine::builder().build().await?` and it starts with the local runtime.

!!! tip
    The resolution chain applies to the CLI and server as well. Running `ciab server start` with no config file works by using built-in defaults.

!!! note
    Environment variables use the prefix `CIAB_` and map to config keys with underscores. For example, `server.port` becomes `CIAB_PORT` and `runtime.backend` becomes `CIAB_RUNTIME_BACKEND`.
