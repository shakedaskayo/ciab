use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::Json;
use ciab_core::error::{CiabError, CiabResult};
use ciab_core::types::config::GatewayConfig;
use ciab_gateway::types::{TokenScope, TunnelType};
use ciab_gateway::GatewayManager;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::state::AppState;

async fn get_gateway(state: &AppState) -> CiabResult<Arc<GatewayManager>> {
    let guard = state.gateway.read().await;
    guard.clone().ok_or(CiabError::GatewayNotEnabled)
}

// -------------------------------------------------------------------------
// Status
// -------------------------------------------------------------------------

pub async fn gateway_status(State(state): State<AppState>) -> Result<impl IntoResponse, CiabError> {
    let gw = get_gateway(&state).await?;
    let status = gw.status().await?;
    Ok(Json(status))
}

// -------------------------------------------------------------------------
// Config (read + update)
// -------------------------------------------------------------------------

pub async fn get_gateway_config(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, CiabError> {
    Ok(Json(state.config.gateway.clone()))
}

#[derive(Deserialize)]
pub struct UpdateGatewayConfigRequest {
    #[serde(default)]
    pub enabled: Option<bool>,
    /// Switch the active tunnel provider: "frp", "bore", "cloudflare", "ngrok"
    #[serde(default)]
    pub tunnel_provider: Option<String>,
    #[serde(default)]
    pub lan: Option<UpdateLanConfig>,
    #[serde(default)]
    pub frp: Option<UpdateFrpConfig>,
    #[serde(default)]
    pub bore: Option<UpdateBoreConfig>,
    #[serde(default)]
    pub cloudflare: Option<UpdateCloudflareConfig>,
    #[serde(default)]
    pub ngrok: Option<UpdateNgrokConfig>,
    #[serde(default)]
    pub routing: Option<UpdateRoutingConfig>,
    #[serde(default)]
    pub advanced: Option<UpdateAdvancedConfig>,
}

#[derive(Deserialize)]
pub struct UpdateLanConfig {
    pub enabled: Option<bool>,
    pub mdns_name: Option<String>,
    pub advertise_port: Option<u16>,
}

#[derive(Deserialize)]
pub struct UpdateFrpConfig {
    pub enabled: Option<bool>,
    pub server_addr: Option<String>,
    pub server_port: Option<u16>,
    pub auth_token: Option<String>,
    pub subdomain_prefix: Option<String>,
    pub tls_enable: Option<bool>,
}

#[derive(Deserialize)]
pub struct UpdateBoreConfig {
    pub enabled: Option<bool>,
    pub binary: Option<String>,
    pub server: Option<String>,
    pub server_port: Option<u16>,
    pub secret: Option<String>,
    pub auto_install: Option<bool>,
}

#[derive(Deserialize)]
pub struct UpdateCloudflareConfig {
    pub enabled: Option<bool>,
    pub binary: Option<String>,
    pub tunnel_token: Option<String>,
    pub tunnel_name: Option<String>,
    pub auto_install: Option<bool>,
}

#[derive(Deserialize)]
pub struct UpdateNgrokConfig {
    pub enabled: Option<bool>,
    pub binary: Option<String>,
    pub authtoken: Option<String>,
    pub domain: Option<String>,
    pub region: Option<String>,
    pub auto_install: Option<bool>,
}

#[derive(Deserialize)]
pub struct UpdateRoutingConfig {
    pub mode: Option<String>,
    pub base_domain: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateAdvancedConfig {
    pub custom_dns_cname: Option<String>,
    pub k8s_ingress_class: Option<String>,
    pub k8s_ingress_annotations: Option<std::collections::HashMap<String, String>>,
}

pub async fn update_gateway_config(
    State(state): State<AppState>,
    Json(req): Json<UpdateGatewayConfigRequest>,
) -> Result<impl IntoResponse, CiabError> {
    // 1. Build new GatewayConfig by merging current config with the update.
    let mut new_gw_config = state.config.gateway.clone();

    if let Some(enabled) = req.enabled {
        new_gw_config.enabled = enabled;
    }

    if let Some(provider) = req.tunnel_provider {
        new_gw_config.tunnel_provider = provider;
    }

    if let Some(lan) = req.lan {
        if let Some(v) = lan.enabled {
            new_gw_config.lan.enabled = v;
        }
        if let Some(v) = lan.mdns_name {
            new_gw_config.lan.mdns_name = v;
        }
        if let Some(v) = lan.advertise_port {
            new_gw_config.lan.advertise_port = v;
        }
    }

    if let Some(frp) = req.frp {
        if let Some(v) = frp.enabled {
            new_gw_config.frp.enabled = v;
        }
        if let Some(v) = frp.server_addr {
            new_gw_config.frp.server_addr = Some(v);
        }
        if let Some(v) = frp.server_port {
            new_gw_config.frp.server_port = Some(v);
        }
        if let Some(v) = frp.auth_token {
            new_gw_config.frp.auth_token = Some(v);
        }
        if let Some(v) = frp.subdomain_prefix {
            new_gw_config.frp.subdomain_prefix = Some(v);
        }
        if let Some(v) = frp.tls_enable {
            new_gw_config.frp.tls_enable = v;
        }
    }

    if let Some(bore) = req.bore {
        if let Some(v) = bore.enabled {
            new_gw_config.bore.enabled = v;
        }
        if let Some(v) = bore.binary {
            new_gw_config.bore.binary = v;
        }
        if let Some(v) = bore.server {
            new_gw_config.bore.server = v;
        }
        if let Some(v) = bore.server_port {
            new_gw_config.bore.server_port = Some(v);
        }
        if let Some(v) = bore.secret {
            new_gw_config.bore.secret = Some(v);
        }
        if let Some(v) = bore.auto_install {
            new_gw_config.bore.auto_install = v;
        }
    }

    if let Some(cf) = req.cloudflare {
        if let Some(v) = cf.enabled {
            new_gw_config.cloudflare.enabled = v;
        }
        if let Some(v) = cf.binary {
            new_gw_config.cloudflare.binary = v;
        }
        if let Some(v) = cf.tunnel_token {
            new_gw_config.cloudflare.tunnel_token = Some(v);
        }
        if let Some(v) = cf.tunnel_name {
            new_gw_config.cloudflare.tunnel_name = Some(v);
        }
        if let Some(v) = cf.auto_install {
            new_gw_config.cloudflare.auto_install = v;
        }
    }

    if let Some(ngrok) = req.ngrok {
        if let Some(v) = ngrok.enabled {
            new_gw_config.ngrok.enabled = v;
        }
        if let Some(v) = ngrok.binary {
            new_gw_config.ngrok.binary = v;
        }
        if let Some(v) = ngrok.authtoken {
            new_gw_config.ngrok.authtoken = Some(v);
        }
        if let Some(v) = ngrok.domain {
            new_gw_config.ngrok.domain = Some(v);
        }
        if let Some(v) = ngrok.region {
            new_gw_config.ngrok.region = Some(v);
        }
        if let Some(v) = ngrok.auto_install {
            new_gw_config.ngrok.auto_install = v;
        }
    }

    if let Some(routing) = req.routing {
        if let Some(v) = routing.mode {
            new_gw_config.routing.mode = v;
        }
        if let Some(v) = routing.base_domain {
            new_gw_config.routing.base_domain = Some(v);
        }
    }

    if let Some(advanced) = req.advanced {
        if let Some(v) = advanced.custom_dns_cname {
            new_gw_config.advanced.custom_dns_cname = Some(v);
        }
        if let Some(v) = advanced.k8s_ingress_class {
            new_gw_config.advanced.k8s_ingress_class = Some(v);
        }
        if let Some(v) = advanced.k8s_ingress_annotations {
            new_gw_config.advanced.k8s_ingress_annotations = v;
        }
    }

    // 2. Persist to config.toml if path is available.
    if let Some(ref config_path) = state.config_path {
        persist_gateway_config(config_path, &new_gw_config).await?;
    }

    // 3. Hot-reload: shutdown old gateway manager (if any), create + start new one.
    {
        let guard = state.gateway.read().await;
        if let Some(ref old_gw) = *guard {
            let _ = old_gw.shutdown().await;
        }
    }

    if new_gw_config.enabled {
        let new_gw = GatewayManager::new(new_gw_config.clone(), state.db.clone());
        new_gw.start().await?;
        let mut guard = state.gateway.write().await;
        *guard = Some(Arc::new(new_gw));
    } else {
        let mut guard = state.gateway.write().await;
        *guard = None;
    }

    Ok(Json(serde_json::json!({
        "status": "updated",
        "config": new_gw_config,
    })))
}

/// Persist the gateway config section to the config.toml file.
///
/// Reads the full TOML, updates only the `[gateway]` section, writes it back.
async fn persist_gateway_config(
    config_path: &str,
    gateway_config: &GatewayConfig,
) -> CiabResult<()> {
    let content = tokio::fs::read_to_string(config_path)
        .await
        .map_err(|e| CiabError::ConfigError(format!("reading config file: {e}")))?;

    let mut doc: toml::Table = toml::from_str(&content)
        .map_err(|e| CiabError::ConfigError(format!("parsing config TOML: {e}")))?;

    // Serialize the new gateway config and insert it.
    let gw_value = toml::Value::try_from(gateway_config)
        .map_err(|e| CiabError::ConfigError(format!("serializing gateway config: {e}")))?;

    doc.insert("gateway".to_string(), gw_value);

    let new_content = toml::to_string_pretty(&doc)
        .map_err(|e| CiabError::ConfigError(format!("writing config TOML: {e}")))?;

    tokio::fs::write(config_path, new_content)
        .await
        .map_err(|e| CiabError::ConfigError(format!("writing config file: {e}")))?;

    Ok(())
}

// -------------------------------------------------------------------------
// Tokens
// -------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct CreateTokenRequest {
    pub name: String,
    #[serde(default)]
    pub scopes: Vec<TokenScope>,
    pub expires_secs: Option<u64>,
}

pub async fn create_token(
    State(state): State<AppState>,
    Json(req): Json<CreateTokenRequest>,
) -> Result<impl IntoResponse, CiabError> {
    let gw = get_gateway(&state).await?;
    let scopes = if req.scopes.is_empty() {
        vec![TokenScope::FullAccess]
    } else {
        req.scopes
    };
    let (raw_token, token) = gw.create_token(req.name, scopes, req.expires_secs).await?;
    Ok(Json(serde_json::json!({
        "token": raw_token,
        "token_info": token,
    })))
}

pub async fn list_tokens(State(state): State<AppState>) -> Result<impl IntoResponse, CiabError> {
    let gw = get_gateway(&state).await?;
    let tokens = gw.list_tokens().await?;
    Ok(Json(tokens))
}

pub async fn get_token(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, CiabError> {
    let gw = get_gateway(&state).await?;
    let uuid = Uuid::parse_str(&id).map_err(|_| CiabError::ClientTokenNotFound(id))?;
    let token = gw.get_token(&uuid).await?;
    Ok(Json(token))
}

pub async fn revoke_token(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, CiabError> {
    let gw = get_gateway(&state).await?;
    let uuid = Uuid::parse_str(&id).map_err(|_| CiabError::ClientTokenNotFound(id))?;
    gw.revoke_token(&uuid).await?;
    Ok(Json(serde_json::json!({"status": "revoked"})))
}

// -------------------------------------------------------------------------
// Tunnels
// -------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct CreateTunnelRequest {
    pub sandbox_id: Option<Uuid>,
    #[serde(default = "default_tunnel_type")]
    pub tunnel_type: String,
    #[serde(default = "default_local_port")]
    pub local_port: u16,
    pub public_url: Option<String>,
}

fn default_tunnel_type() -> String {
    "frp".to_string()
}

fn default_local_port() -> u16 {
    9090
}

pub async fn create_tunnel(
    State(state): State<AppState>,
    Json(req): Json<CreateTunnelRequest>,
) -> Result<impl IntoResponse, CiabError> {
    let gw = get_gateway(&state).await?;
    let tunnel_type: TunnelType = req
        .tunnel_type
        .parse()
        .map_err(|e: String| CiabError::ConfigValidationError(e))?;
    let tunnel = gw
        .create_tunnel(req.sandbox_id, tunnel_type, req.local_port, req.public_url)
        .await?;
    Ok(Json(tunnel))
}

pub async fn list_tunnels(State(state): State<AppState>) -> Result<impl IntoResponse, CiabError> {
    let gw = get_gateway(&state).await?;
    let tunnels = gw.list_tunnels().await?;
    Ok(Json(tunnels))
}

pub async fn get_tunnel(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, CiabError> {
    let gw = get_gateway(&state).await?;
    let uuid = Uuid::parse_str(&id).map_err(|_| CiabError::TunnelNotFound(id))?;
    let tunnel = gw.get_tunnel(&uuid).await?;
    Ok(Json(tunnel))
}

pub async fn delete_tunnel(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, CiabError> {
    let gw = get_gateway(&state).await?;
    let uuid = Uuid::parse_str(&id).map_err(|_| CiabError::TunnelNotFound(id))?;
    gw.stop_tunnel(&uuid).await?;
    Ok(Json(serde_json::json!({"status": "stopped"})))
}

pub async fn create_sandbox_tunnel(
    State(state): State<AppState>,
    Path(sandbox_id): Path<String>,
) -> Result<impl IntoResponse, CiabError> {
    let gw = get_gateway(&state).await?;
    let sid = Uuid::parse_str(&sandbox_id).map_err(|_| CiabError::SandboxNotFound(sandbox_id))?;
    let tunnel_type: TunnelType = gw.config.tunnel_provider.parse().unwrap_or(TunnelType::Lan);
    let local_port = state.config.server.port;

    let public_url = if tunnel_type == TunnelType::Lan {
        let addrs = gw.lan.status().local_addresses;
        let addr = addrs
            .first()
            .cloned()
            .unwrap_or_else(|| "127.0.0.1".to_string());
        Some(format!("http://{}:{}", addr, local_port))
    } else {
        None
    };

    let tunnel = gw
        .create_tunnel(Some(sid), tunnel_type, local_port, public_url)
        .await?;
    Ok(Json(tunnel))
}

// -------------------------------------------------------------------------
// Provider Prepare (download/install/validate)
// -------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct PrepareProviderRequest {
    pub provider: String,
}

pub async fn prepare_provider(
    State(state): State<AppState>,
    Json(req): Json<PrepareProviderRequest>,
) -> Result<impl IntoResponse, CiabError> {
    let gw = get_gateway(&state).await?;
    let result = gw.prepare_provider(&req.provider).await?;
    Ok(Json(result))
}

// -------------------------------------------------------------------------
// Expose (convenience)
// -------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct ExposeRequest {
    pub sandbox_id: Uuid,
    pub token_name: Option<String>,
    pub expires_secs: Option<u64>,
    pub scope: Option<TokenScope>,
}

pub async fn expose(
    State(state): State<AppState>,
    Json(req): Json<ExposeRequest>,
) -> Result<impl IntoResponse, CiabError> {
    let gw = get_gateway(&state).await?;
    let local_port = state.config.server.port;
    let result = gw
        .expose_sandbox(
            req.sandbox_id,
            req.token_name,
            req.expires_secs,
            req.scope,
            local_port,
        )
        .await?;
    Ok(Json(result))
}

// -------------------------------------------------------------------------
// Discover
// -------------------------------------------------------------------------

pub async fn discover(State(state): State<AppState>) -> Result<impl IntoResponse, CiabError> {
    let gw = get_gateway(&state).await?;
    let lan_status = gw.lan.status();
    Ok(Json(serde_json::json!({
        "lan": lan_status,
        "server_version": env!("CARGO_PKG_VERSION"),
    })))
}
