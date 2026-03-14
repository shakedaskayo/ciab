# Custom Provider

Add support for a new coding agent by implementing the `AgentProvider` trait.

## Step 1: Create the Crate

```bash
cargo new --lib crates/ciab-agent-myagent
```

Add to the workspace in `Cargo.toml`:

```toml
members = [
    # ...
    "crates/ciab-agent-myagent",
]
```

## Step 2: Implement AgentProvider

```rust
use async_trait::async_trait;
use ciab_core::{
    traits::AgentProvider,
    types::{AgentCommand, AgentConfig, AgentHealth, Message, StreamEvent},
    error::CiabResult,
};
use tokio::sync::mpsc;
use uuid::Uuid;

pub struct MyAgentProvider {
    // Provider-specific configuration
}

#[async_trait]
impl AgentProvider for MyAgentProvider {
    fn name(&self) -> &str {
        "my-agent"
    }

    fn base_image(&self) -> &str {
        "ghcr.io/shakedaskayo/ciab-myagent:latest"
    }

    fn install_commands(&self) -> Vec<String> {
        vec![
            "npm install -g my-agent-cli@latest".into(),
        ]
    }

    fn build_start_command(&self, config: &AgentConfig) -> AgentCommand {
        AgentCommand {
            command: vec!["my-agent".into(), "--headless".into()],
            env: Default::default(),
        }
    }

    fn required_env_vars(&self) -> Vec<String> {
        vec!["MY_AGENT_API_KEY".into()]
    }

    fn parse_output(&self, sandbox_id: &Uuid, raw: &str) -> Vec<StreamEvent> {
        // Parse your agent's output format into StreamEvents
        vec![]
    }

    fn validate_config(&self, config: &AgentConfig) -> CiabResult<()> {
        Ok(())
    }

    async fn send_message(
        &self,
        sandbox_id: &Uuid,
        session_id: &Uuid,
        message: &Message,
        tx: &mpsc::Sender<StreamEvent>,
    ) -> CiabResult<()> {
        // Send message to agent via sandbox execd API
        // Parse streaming output and send events via tx
        Ok(())
    }

    async fn interrupt(&self, sandbox_id: &Uuid) -> CiabResult<()> {
        // Send interrupt signal to agent process
        Ok(())
    }

    async fn health_check(&self, sandbox_id: &Uuid) -> CiabResult<AgentHealth> {
        Ok(AgentHealth::Healthy)
    }
}
```

## Step 3: Create the Container Image

Create `images/myagent-sandbox/Dockerfile`:

```dockerfile
FROM node:22-slim

RUN apt-get update && apt-get install -y \
    git curl ca-certificates openssh-client \
    && rm -rf /var/lib/apt/lists/*

RUN npm install -g my-agent-cli@latest

WORKDIR /workspace
```

## Step 4: Register the Provider

In `ciab-api`, register your provider during server initialization alongside the existing providers.

## Step 5: Add Configuration

Add to `config.toml`:

```toml
[agents.providers.my-agent]
enabled = true
image = "ghcr.io/shakedaskayo/ciab-myagent:latest"
api_key_env = "MY_AGENT_API_KEY"
```
