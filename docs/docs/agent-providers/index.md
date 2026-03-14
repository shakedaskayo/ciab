# Agent Providers

CIAB supports multiple coding agent providers through the `AgentProvider` trait. Each provider knows how to install, start, communicate with, and parse output from a specific coding agent.

## Available Providers

| Provider | Agent | Status |
|----------|-------|--------|
| [`claude-code`](claude-code.md) | Claude Code | Supported |
| [`codex`](codex.md) | OpenAI Codex CLI | Supported |
| [`gemini`](gemini.md) | Gemini CLI | Supported |
| [`cursor`](cursor.md) | Cursor CLI | Supported |

## AgentProvider Trait

All providers implement this trait:

```rust
pub trait AgentProvider: Send + Sync {
    /// Provider name (e.g., "claude-code")
    fn name(&self) -> &str;

    /// Base container image
    fn base_image(&self) -> &str;

    /// Commands to install the agent in the container
    fn install_commands(&self) -> Vec<String>;

    /// Build the command to start the agent
    fn build_start_command(&self, config: &AgentConfig) -> AgentCommand;

    /// Environment variables required by this agent
    fn required_env_vars(&self) -> Vec<String>;

    /// Parse raw agent output into StreamEvents
    fn parse_output(&self, sandbox_id: &Uuid, raw: &str) -> Vec<StreamEvent>;

    /// Validate agent-specific configuration
    fn validate_config(&self, config: &AgentConfig) -> CiabResult<()>;

    /// Send a message and stream the response
    async fn send_message(
        &self,
        sandbox_id: &Uuid,
        session_id: &Uuid,
        message: &Message,
        tx: &mpsc::Sender<StreamEvent>,
    ) -> CiabResult<()>;

    /// Interrupt the agent
    async fn interrupt(&self, sandbox_id: &Uuid) -> CiabResult<()>;

    /// Check agent health
    async fn health_check(&self, sandbox_id: &Uuid) -> CiabResult<AgentHealth>;
}
```

## How Providers Work

1. During **provisioning**, the pipeline calls `install_commands()` to install the agent in the container
2. After installation, `build_start_command()` generates the command to launch the agent process
3. During **chat**, `send_message()` forwards user messages and streams responses via the `tx` channel
4. The `parse_output()` method converts raw agent output (often JSON) into typed `StreamEvent`s
5. `health_check()` verifies the agent process is running and responsive

## Adding a Custom Provider

See [Custom Provider](custom-provider.md) for a step-by-step guide.
