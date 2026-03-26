use std::collections::HashMap;
use std::sync::Arc;

use ciab_channels::ChannelManager;
use ciab_core::traits::agent::AgentProvider;
use ciab_core::traits::runtime::SandboxRuntime;
use ciab_core::traits::stream::StreamHandler;
use ciab_core::types::agent::PermissionPolicy;
use ciab_core::types::config::AppConfig;
use ciab_core::types::session::{MessageContent, MessageRole};
use ciab_credentials::CredentialStore;
use ciab_db::Database;
use ciab_gateway::GatewayManager;
use ciab_provisioning::ProvisioningPipeline;
use serde::{Deserialize, Serialize};
use tokio::sync::{oneshot, RwLock};
use uuid::Uuid;

/// A pending permission request waiting for user response.
pub struct PendingPermission {
    pub tx: oneshot::Sender<bool>,
}

/// A pending user input request waiting for user's answer.
pub struct PendingUserInput {
    pub tx: oneshot::Sender<String>,
}

/// A queued message waiting to be processed by the agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedMessage {
    pub id: Uuid,
    pub session_id: Uuid,
    pub role: MessageRole,
    pub content: Vec<MessageContent>,
    pub prompt_text: String,
    pub queued_at: chrono::DateTime<chrono::Utc>,
}

/// Per-session message queue. Messages are processed FIFO.
/// The `processing` flag indicates if an agent is currently running for this session.
#[derive(Default)]
pub struct SessionQueue {
    pub messages: std::collections::VecDeque<QueuedMessage>,
    pub processing: bool,
}

#[derive(Clone)]
pub struct AppState {
    pub runtime: Arc<dyn SandboxRuntime>,
    pub agents: HashMap<String, Arc<dyn AgentProvider>>,
    /// All available runtimes keyed by backend name ("local", "opensandbox", "docker")
    pub runtimes: HashMap<String, Arc<dyn SandboxRuntime>>,
    pub credentials: Arc<CredentialStore>,
    pub stream_handler: Arc<dyn StreamHandler>,
    pub provisioning: Arc<ProvisioningPipeline>,
    pub db: Arc<Database>,
    pub config: Arc<AppConfig>,
    /// Path to the config.toml file on disk (for runtime updates).
    pub config_path: Option<String>,
    /// Gateway manager — wrapped in RwLock so it can be enabled/reconfigured at runtime.
    pub gateway: Arc<RwLock<Option<Arc<GatewayManager>>>>,
    /// Channel manager — wrapped in RwLock so channels can be managed at runtime.
    pub channel_manager: Arc<RwLock<Option<Arc<ChannelManager>>>>,
    /// Pending permission requests keyed by request_id (agent-provided string), awaiting user approve/deny.
    pub pending_permissions: Arc<RwLock<HashMap<String, PendingPermission>>>,
    /// Per-session permission policies (defaults to AutoApprove).
    pub session_permissions: Arc<RwLock<HashMap<Uuid, PermissionPolicy>>>,
    /// Pending user input requests keyed by request_id (agent-provided string), awaiting user's answer.
    pub pending_user_inputs: Arc<RwLock<HashMap<String, PendingUserInput>>>,
    /// Per-session message queues. Messages are processed FIFO, one at a time per session.
    pub session_queues: Arc<RwLock<HashMap<Uuid, SessionQueue>>>,
    /// Optional image builder (e.g., Packer). Present when [packer] is configured.
    pub image_builder: Option<Arc<dyn ciab_core::traits::image_builder::ImageBuilder>>,
}
