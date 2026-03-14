use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Supported channel providers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChannelProvider {
    WhatsApp,
    Slack,
    Webhook,
}

impl std::fmt::Display for ChannelProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WhatsApp => write!(f, "whatsapp"),
            Self::Slack => write!(f, "slack"),
            Self::Webhook => write!(f, "webhook"),
        }
    }
}

/// Channel lifecycle state
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChannelState {
    #[default]
    Inactive,
    Pairing,
    Connected,
    Reconnecting,
    Failed,
    Stopped,
}

/// How a channel resolves its target sandbox
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ChannelBinding {
    /// Route all messages to a fixed sandbox
    Static { sandbox_id: Uuid },
    /// Auto-provision a sandbox from a workspace for each sender
    AutoProvision {
        workspace_id: Uuid,
        #[serde(default = "default_ttl")]
        ttl_secs: u64,
        #[serde(default)]
        persist_on_expire: bool,
    },
}

fn default_ttl() -> u64 {
    3600
}

/// Policy for direct messages
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DmPolicy {
    #[default]
    Respond,
    AllowedOnly,
    Ignore,
}

/// Policy for group messages
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GroupPolicy {
    All,
    #[default]
    MentionOnly,
    CommandsOnly,
    Ignore,
}

/// Rules governing how messages are filtered and handled
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelRules {
    #[serde(default)]
    pub allowed_senders: Vec<String>,
    #[serde(default)]
    pub blocked_senders: Vec<String>,
    #[serde(default)]
    pub reset_trigger: Option<String>,
    #[serde(default)]
    pub dm_policy: DmPolicy,
    #[serde(default)]
    pub group_policy: GroupPolicy,
    #[serde(default)]
    pub rate_limit_per_minute: Option<u32>,
    #[serde(default = "default_true")]
    pub persist_conversation: bool,
    #[serde(default)]
    pub max_message_length: Option<usize>,
}

fn default_true() -> bool {
    true
}

impl Default for ChannelRules {
    fn default() -> Self {
        Self {
            allowed_senders: Vec::new(),
            blocked_senders: Vec::new(),
            reset_trigger: None,
            dm_policy: DmPolicy::default(),
            group_policy: GroupPolicy::default(),
            rate_limit_per_minute: None,
            persist_conversation: true,
            max_message_length: None,
        }
    }
}

/// Provider-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "provider", rename_all = "snake_case")]
pub enum ChannelProviderConfig {
    WhatsApp {
        #[serde(default)]
        session_dir: Option<String>,
        #[serde(default)]
        phone_number: Option<String>,
    },
    Slack {
        bot_token: String,
        #[serde(default)]
        app_token: Option<String>,
        #[serde(default)]
        signing_secret: Option<String>,
        #[serde(default)]
        listen_channels: Vec<String>,
    },
    Webhook {
        #[serde(default)]
        inbound_secret: Option<String>,
        #[serde(default)]
        outbound_url: Option<String>,
        #[serde(default)]
        outbound_headers: HashMap<String, String>,
    },
}

/// A channel binding a messaging platform to a sandbox
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub id: Uuid,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub provider: ChannelProvider,
    #[serde(default)]
    pub state: ChannelState,
    pub binding: ChannelBinding,
    pub provider_config: ChannelProviderConfig,
    #[serde(default)]
    pub rules: ChannelRules,
    #[serde(default)]
    pub labels: HashMap<String, String>,
    #[serde(default)]
    pub error_message: Option<String>,
    #[serde(default)]
    pub qr_code: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Direction of a channel message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MessageDirection {
    Inbound,
    Outbound,
}

/// Audit log entry for a message passing through a channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelMessage {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub direction: MessageDirection,
    pub sender_id: String,
    #[serde(default)]
    pub sender_name: Option<String>,
    #[serde(default)]
    pub sandbox_id: Option<Uuid>,
    #[serde(default)]
    pub session_id: Option<Uuid>,
    pub content: String,
    #[serde(default)]
    pub platform_metadata: HashMap<String, serde_json::Value>,
    pub timestamp: DateTime<Utc>,
}

/// Internal routing struct for inbound messages (not persisted directly)
#[derive(Debug, Clone)]
pub struct InboundMessage {
    pub platform_message_id: Option<String>,
    pub sender_id: String,
    pub sender_name: Option<String>,
    pub content: String,
    pub is_group: bool,
    pub group_id: Option<String>,
    pub is_mention: bool,
    pub platform_metadata: HashMap<String, serde_json::Value>,
}

/// Filters for listing channels
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChannelFilters {
    #[serde(default)]
    pub provider: Option<ChannelProvider>,
    #[serde(default)]
    pub state: Option<ChannelState>,
    #[serde(default)]
    pub name: Option<String>,
}
