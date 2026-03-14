use std::collections::HashMap;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use chrono::Utc;
use ciab_core::error::CiabError;
use ciab_core::types::channel::{
    Channel, ChannelBinding, ChannelFilters, ChannelProvider, ChannelProviderConfig, ChannelRules,
    ChannelState,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::state::AppState;

// ---------------------------------------------------------------------------
// create_channel
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateChannelRequest {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub provider: ChannelProvider,
    pub binding: ChannelBinding,
    pub provider_config: ChannelProviderConfig,
    #[serde(default)]
    pub rules: ChannelRules,
    #[serde(default)]
    pub labels: HashMap<String, String>,
}

pub async fn create_channel(
    State(state): State<AppState>,
    Json(body): Json<CreateChannelRequest>,
) -> Result<impl IntoResponse, CiabError> {
    let now = Utc::now();
    let channel = Channel {
        id: Uuid::new_v4(),
        name: body.name,
        description: body.description,
        provider: body.provider,
        state: ChannelState::Inactive,
        binding: body.binding,
        provider_config: body.provider_config,
        rules: body.rules,
        labels: body.labels,
        error_message: None,
        qr_code: None,
        created_at: now,
        updated_at: now,
    };

    state.db.insert_channel(&channel).await?;
    Ok((StatusCode::CREATED, Json(channel)))
}

// ---------------------------------------------------------------------------
// list_channels
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Default)]
pub struct ListChannelsQuery {
    pub provider: Option<ChannelProvider>,
    pub state: Option<ChannelState>,
    pub name: Option<String>,
}

pub async fn list_channels(
    State(state): State<AppState>,
    Query(params): Query<ListChannelsQuery>,
) -> Result<impl IntoResponse, CiabError> {
    let filters = ChannelFilters {
        provider: params.provider,
        state: params.state,
        name: params.name,
    };
    let channels = state.db.list_channels(&filters).await?;
    Ok(Json(channels))
}

// ---------------------------------------------------------------------------
// get_channel
// ---------------------------------------------------------------------------

pub async fn get_channel(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, CiabError> {
    let channel = state
        .db
        .get_channel(&id)
        .await?
        .ok_or_else(|| CiabError::ChannelNotFound(id.to_string()))?;
    Ok(Json(channel))
}

// ---------------------------------------------------------------------------
// update_channel
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct UpdateChannelRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub binding: Option<ChannelBinding>,
    #[serde(default)]
    pub provider_config: Option<ChannelProviderConfig>,
    #[serde(default)]
    pub rules: Option<ChannelRules>,
    #[serde(default)]
    pub labels: Option<HashMap<String, String>>,
}

pub async fn update_channel(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateChannelRequest>,
) -> Result<impl IntoResponse, CiabError> {
    let mut channel = state
        .db
        .get_channel(&id)
        .await?
        .ok_or_else(|| CiabError::ChannelNotFound(id.to_string()))?;

    if let Some(name) = body.name {
        channel.name = name;
    }
    if let Some(description) = body.description {
        channel.description = Some(description);
    }
    if let Some(binding) = body.binding {
        channel.binding = binding;
    }
    if let Some(config) = body.provider_config {
        channel.provider_config = config;
    }
    if let Some(rules) = body.rules {
        channel.rules = rules;
    }
    if let Some(labels) = body.labels {
        channel.labels = labels;
    }
    channel.updated_at = Utc::now();

    state.db.update_channel(&id, &channel).await?;
    Ok(Json(channel))
}

// ---------------------------------------------------------------------------
// delete_channel
// ---------------------------------------------------------------------------

pub async fn delete_channel(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, CiabError> {
    // Stop channel if running
    let mgr = state.channel_manager.read().await;
    if let Some(ref cm) = *mgr {
        let _ = cm.stop_channel(&id).await;
    }
    drop(mgr);

    state.db.delete_channel(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// start_channel
// ---------------------------------------------------------------------------

pub async fn start_channel(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, CiabError> {
    let mgr = state.channel_manager.read().await;
    let cm = mgr
        .as_ref()
        .ok_or_else(|| CiabError::ChannelAdapterError("channels not enabled".to_string()))?;
    cm.start_channel(&id).await?;
    Ok(Json(serde_json::json!({"status": "started"})))
}

// ---------------------------------------------------------------------------
// stop_channel
// ---------------------------------------------------------------------------

pub async fn stop_channel(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, CiabError> {
    let mgr = state.channel_manager.read().await;
    let cm = mgr
        .as_ref()
        .ok_or_else(|| CiabError::ChannelAdapterError("channels not enabled".to_string()))?;
    cm.stop_channel(&id).await?;
    Ok(Json(serde_json::json!({"status": "stopped"})))
}

// ---------------------------------------------------------------------------
// restart_channel
// ---------------------------------------------------------------------------

pub async fn restart_channel(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, CiabError> {
    let mgr = state.channel_manager.read().await;
    let cm = mgr
        .as_ref()
        .ok_or_else(|| CiabError::ChannelAdapterError("channels not enabled".to_string()))?;
    cm.restart_channel(&id).await?;
    Ok(Json(serde_json::json!({"status": "restarted"})))
}

// ---------------------------------------------------------------------------
// channel_status
// ---------------------------------------------------------------------------

pub async fn channel_status(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, CiabError> {
    let mgr = state.channel_manager.read().await;
    let channel_state = if let Some(ref cm) = *mgr {
        cm.channel_state(&id).await?
    } else {
        let ch = state
            .db
            .get_channel(&id)
            .await?
            .ok_or_else(|| CiabError::ChannelNotFound(id.to_string()))?;
        ch.state
    };
    Ok(Json(serde_json::json!({"state": channel_state})))
}

// ---------------------------------------------------------------------------
// channel_qr (WhatsApp pairing)
// ---------------------------------------------------------------------------

pub async fn channel_qr(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, CiabError> {
    let mgr = state.channel_manager.read().await;
    let qr = if let Some(ref cm) = *mgr {
        cm.whatsapp_qr(&id).await?
    } else {
        None
    };
    Ok(Json(serde_json::json!({"qr_code": qr})))
}

// ---------------------------------------------------------------------------
// channel_messages
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Default)]
pub struct ChannelMessagesQuery {
    pub limit: Option<u32>,
    pub sender_id: Option<String>,
}

pub async fn channel_messages(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(params): Query<ChannelMessagesQuery>,
) -> Result<impl IntoResponse, CiabError> {
    let messages = state
        .db
        .list_channel_messages(&id, params.limit, params.sender_id.as_deref())
        .await?;
    Ok(Json(messages))
}

// ---------------------------------------------------------------------------
// webhook_inbound — public endpoint, no auth
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct WebhookInboundPayload {
    pub sender_id: String,
    #[serde(default)]
    pub sender_name: Option<String>,
    pub content: String,
    #[serde(default)]
    pub platform_metadata: HashMap<String, serde_json::Value>,
}

pub async fn webhook_inbound(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<WebhookInboundPayload>,
) -> Result<impl IntoResponse, CiabError> {
    // Verify channel exists and is a webhook
    let channel = state
        .db
        .get_channel(&id)
        .await?
        .ok_or_else(|| CiabError::ChannelNotFound(id.to_string()))?;

    if channel.provider != ChannelProvider::Webhook {
        return Err(CiabError::ChannelAdapterError(
            "channel is not a webhook".to_string(),
        ));
    }

    let msg = ciab_core::types::channel::InboundMessage {
        platform_message_id: None,
        sender_id: body.sender_id,
        sender_name: body.sender_name,
        content: body.content,
        is_group: false,
        group_id: None,
        is_mention: false,
        platform_metadata: body.platform_metadata,
    };

    // Log the message even if the channel manager isn't running
    let inbound_log = ciab_core::types::channel::ChannelMessage {
        id: Uuid::new_v4(),
        channel_id: id,
        direction: ciab_core::types::channel::MessageDirection::Inbound,
        sender_id: msg.sender_id.clone(),
        sender_name: msg.sender_name.clone(),
        sandbox_id: None,
        session_id: None,
        content: msg.content.clone(),
        platform_metadata: msg.platform_metadata.clone(),
        timestamp: Utc::now(),
    };
    state.db.insert_channel_message(&inbound_log).await?;

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({"status": "received"})),
    ))
}
