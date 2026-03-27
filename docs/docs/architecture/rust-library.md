# Rust Library

Embed CIAB in any Rust application using the `ciab` library crate. The library provides a high-level `CiabEngine` API that wraps all CIAB functionality behind a single entry point.

## Overview

The `ciab` crate is a facade that re-exports types from the internal workspace crates and provides `CiabEngine` -- a batteries-included entry point for creating and managing sandboxes programmatically.

```toml
[dependencies]
ciab = "0.1"
```

## Feature Flags

The library uses feature flags to control which runtime backends are compiled in:

| Feature | Default | Description |
|---------|---------|-------------|
| `local` | Yes | Local process runtime (no Docker) |
| `ec2` | No | AWS EC2 runtime backend |
| `kubernetes` | No | Kubernetes runtime backend |
| `packer` | No | Packer image builder |
| `full` | No | Enables all features |

```toml
# Just local runtime (default)
ciab = "0.1"

# EC2 + Packer support
ciab = { version = "0.2", features = ["ec2", "packer"] }

# Everything
ciab = { version = "0.2", features = ["full"] }
```

## CiabEngine

`CiabEngine` is the main entry point. It owns the database, runtime, streaming broker, and credential store.

### Builder Pattern

```rust
use ciab::{CiabEngine, CiabEngineBuilder};

// Zero-config: uses config resolution chain (see below)
let engine = CiabEngine::builder().build().await?;

// With explicit config file
let engine = CiabEngine::builder()
    .config_path("./my-config.toml")
    .build()
    .await?;

// With inline configuration
let engine = CiabEngine::builder()
    .port(9090)
    .runtime_backend("ec2")
    .database_path("/var/lib/ciab/data.db")
    .build()
    .await?;
```

### Sandbox Lifecycle

```rust
use ciab::{CiabEngine, SandboxSpec};

let engine = CiabEngine::builder().build().await?;

// Create a sandbox
let spec = SandboxSpec::builder()
    .provider("claude-code")
    .env("ANTHROPIC_API_KEY", std::env::var("ANTHROPIC_API_KEY")?)
    .build();

let sandbox = engine.create_sandbox(spec).await?;
println!("Sandbox {} is {:?}", sandbox.id, sandbox.state);

// Execute a command
let result = engine.exec(&sandbox.id, "cargo test").await?;
println!("Exit code: {}", result.exit_code);

// Send a chat message
let response = engine.chat(&sandbox.id, "Explain the codebase").await?;
println!("{}", response.text());

// Clean up
engine.delete_sandbox(&sandbox.id).await?;
```

### Image Building (requires `packer` feature)

```rust
#[cfg(feature = "packer")]
{
    let build = engine.build_image("claude-code", "us-east-1").await?;
    println!("Build started: {}", build.id);

    let result = engine.wait_for_build(&build.id).await?;
    println!("AMI: {}", result.ami_id.unwrap());
}
```

## Re-exports

The `ciab` crate re-exports commonly used types so you do not need to depend on `ciab-core` directly:

- `ciab::SandboxInfo`, `ciab::SandboxSpec`, `ciab::SandboxState`
- `ciab::Session`, `ciab::Message`, `ciab::MessageRole`
- `ciab::StreamEvent`, `ciab::StreamEventType`
- `ciab::ExecRequest`, `ciab::ExecResult`
- `ciab::AppConfig`, `ciab::CiabError`

## Config Resolution Chain

When `CiabEngine::builder().build()` is called without an explicit config path, CIAB resolves configuration through a 5-step chain. Each step overrides values from the previous:

1. **Built-in defaults** -- Sensible defaults for all fields (local runtime, port 8080, etc.)
2. **`./config.toml`** -- Config file in the current working directory
3. **`~/.config/ciab/config.toml`** -- User-level config file
4. **Environment variables** -- `CIAB_PORT`, `CIAB_RUNTIME_BACKEND`, etc. (see [Environment Variables](../configuration/environment-variables.md))
5. **Builder overrides** -- Values set explicitly on `CiabEngineBuilder`

This means CIAB works with zero configuration out of the box -- just `CiabEngine::builder().build().await?` and it starts with the local runtime on port 8080.

!!! tip
    The resolution chain applies to the CLI and server as well. Running `ciab server start` with no config file works by using built-in defaults.

!!! note
    Environment variables use the prefix `CIAB_` and map to config keys with underscores. For example, `server.port` becomes `CIAB_PORT` and `runtime.backend` becomes `CIAB_RUNTIME_BACKEND`.
