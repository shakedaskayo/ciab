use async_trait::async_trait;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::error::CiabResult;
use crate::types::agent::{
    AgentCommand, AgentConfig, AgentHealth, InteractiveProtocol, PromptMode, SlashCommand,
};
use crate::types::llm_provider::AgentLlmCompatibility;
use crate::types::session::Message;
use crate::types::stream::StreamEvent;

#[async_trait]
pub trait AgentProvider: Send + Sync {
    /// The name/identifier of this agent provider.
    fn name(&self) -> &str;

    /// The base container image for sandboxes using this provider.
    fn base_image(&self) -> &str;

    /// Commands to install the agent CLI inside the sandbox.
    fn install_commands(&self) -> Vec<String>;

    /// Build the command used to start the agent process in a sandbox.
    fn build_start_command(&self, config: &AgentConfig) -> AgentCommand;

    /// How prompts should be delivered to the agent process.
    /// Defaults to StdinJson (Claude Code's protocol).
    fn prompt_mode(&self) -> PromptMode {
        PromptMode::StdinJson
    }

    /// Whether the provider supports interactive stdin control protocol.
    /// Claude Code supports control_request/control_response; others don't.
    fn interactive_protocol(&self) -> InteractiveProtocol {
        InteractiveProtocol::None
    }

    /// Environment variables required by this provider.
    fn required_env_vars(&self) -> Vec<String>;

    /// Parse raw output from the agent process into stream events.
    fn parse_output(&self, sandbox_id: &Uuid, raw: &str) -> Vec<StreamEvent>;

    /// Validate that the given config is appropriate for this provider.
    fn validate_config(&self, config: &AgentConfig) -> CiabResult<()>;

    /// Send a message to the agent and receive a stream of events.
    async fn send_message(
        &self,
        sandbox_id: &Uuid,
        session_id: &Uuid,
        message: &Message,
        tx: &mpsc::Sender<StreamEvent>,
    ) -> CiabResult<()>;

    /// Interrupt the agent in the given sandbox.
    async fn interrupt(&self, sandbox_id: &Uuid) -> CiabResult<()>;

    /// Check the health of the agent in a sandbox.
    async fn health_check(&self, sandbox_id: &Uuid) -> CiabResult<AgentHealth>;

    /// Returns slash commands available for this provider.
    fn slash_commands(&self) -> Vec<SlashCommand> {
        vec![]
    }

    /// Returns the LLM providers this agent is compatible with.
    fn supported_llm_providers(&self) -> Vec<AgentLlmCompatibility> {
        vec![]
    }
}
