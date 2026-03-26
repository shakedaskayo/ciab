# Crate Map

CIAB consists of 16 Rust crates organized in a workspace, plus a library facade crate.

## Crate Overview

| Crate | Purpose | Key Types |
|-------|---------|-----------|
| `ciab` | Rust library facade | `CiabEngine`, `CiabEngineBuilder`, feature-gated re-exports |
| `ciab-core` | Foundation types, traits, errors | `SandboxInfo`, `Session`, `Message`, `StreamEvent`, `CiabError` |
| `ciab-db` | SQLite persistence via sqlx | `Database`, migrations |
| `ciab-streaming` | SSE broker and event buffering | `StreamBroker`, `EventBuffer` |
| `ciab-sandbox` | OpenSandbox container client | `SandboxRuntime`, lifecycle + execd APIs |
| `ciab-sandbox-k8s` | Kubernetes runtime backend | `KubernetesRuntime`, Pod builder, PVC, RBAC |
| `ciab-sandbox-ec2` | AWS EC2 runtime backend | `Ec2Runtime`, instance lifecycle, SSH execution |
| `ciab-agent-claude` | Claude Code agent provider | `ClaudeProvider` |
| `ciab-agent-codex` | Codex agent provider | `CodexProvider` |
| `ciab-agent-gemini` | Gemini CLI agent provider | `GeminiProvider` |
| `ciab-agent-cursor` | Cursor agent provider | `CursorProvider` |
| `ciab-credentials` | Encrypted credential store | `CredentialStore`, AES-GCM encryption |
| `ciab-provisioning` | 9-step sandbox provisioning | `ProvisioningPipeline`, `ProvisioningStep` |
| `ciab-packer` | HashiCorp Packer image builder | `PackerBuilder`, template resolution, build tracking |
| `ciab-api` | Axum REST API server | Routes, handlers, middleware |
| `ciab-cli` | CLI binary (`ciab`) | Command definitions, HTTP client |

## Dependency Graph

```mermaid
graph BT
    core[ciab-core]
    db[ciab-db] --> core
    streaming[ciab-streaming] --> core
    sandbox[ciab-sandbox] --> core
    k8s[ciab-sandbox-k8s] --> core
    ec2[ciab-sandbox-ec2] --> core
    claude[ciab-agent-claude] --> core
    codex[ciab-agent-codex] --> core
    gemini[ciab-agent-gemini] --> core
    cursor[ciab-agent-cursor] --> core
    creds[ciab-credentials] --> core
    creds --> db
    packer[ciab-packer] --> core
    prov[ciab-provisioning] --> core
    prov --> sandbox
    prov --> creds
    prov --> streaming
    api[ciab-api] --> core
    api --> db
    api --> streaming
    api --> sandbox
    api --> prov
    api --> creds
    api --> claude
    api --> codex
    api --> gemini
    api --> cursor
    api --> packer
    facade[ciab] --> core
    facade --> db
    facade --> sandbox
    facade --> ec2
    facade --> k8s
    facade --> packer
    facade --> api
    cli[ciab-cli] --> core
```

## Core Types (`ciab-core`)

### Types (`ciab-core/src/types/`)

- **`sandbox.rs`** — `SandboxState`, `SandboxInfo`, `SandboxSpec`, `ResourceLimits`, `ResourceStats`, `ExecRequest`, `ExecResult`, `FileInfo`
- **`session.rs`** — `Session`, `SessionState`, `Message`, `MessageRole`, `MessageContent`
- **`stream.rs`** — `StreamEvent`, `StreamEventType`
- **`config.rs`** — `AppConfig`, `ServerConfig`, `AgentProviderConfig`
- **`credentials.rs`** — `CredentialSet`, `CredentialType`

### Traits (`ciab-core/src/traits/`)

- **`agent.rs`** — `AgentProvider` trait (10 methods)
- **`runtime.rs`** — `SandboxRuntime` trait
- **`stream.rs`** — `StreamHandler` trait
