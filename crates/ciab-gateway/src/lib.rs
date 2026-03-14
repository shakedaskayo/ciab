pub mod lan;
pub mod proxy;
pub mod tokens;
pub mod tunnel;
pub mod types;

use std::sync::Arc;

use chrono::Utc;
use ciab_core::error::{CiabError, CiabResult};
use ciab_core::types::config::GatewayConfig;
use ciab_db::Database;
use uuid::Uuid;

use crate::lan::LanDiscovery;
use crate::tokens::{generate_token, hash_token};
use crate::tunnel::bore::BoreTunnelManager;
use crate::tunnel::cloudflare::CloudflareTunnelManager;
use crate::tunnel::frp::FrpTunnelManager;
use crate::tunnel::ngrok::NgrokTunnelManager;
use crate::tunnel::TunnelManager;
use crate::types::{
    ClientToken, FrpStatus, GatewayStatus, GatewayTunnel, ProviderPrepareResult, TokenScope,
    TunnelProviderInfo, TunnelState, TunnelType,
};

/// Top-level coordinator for the gateway subsystem.
pub struct GatewayManager {
    pub config: GatewayConfig,
    pub db: Arc<Database>,
    pub lan: LanDiscovery,
    tunnel_manager: Option<Box<dyn TunnelManager>>,
}

impl GatewayManager {
    pub fn new(config: GatewayConfig, db: Arc<Database>) -> Self {
        let lan = LanDiscovery::new(config.lan.clone());

        // Pick the active tunnel manager based on the configured provider.
        let tunnel_manager: Option<Box<dyn TunnelManager>> = match config.tunnel_provider.as_str() {
            "frp" if config.frp.enabled => {
                Some(Box::new(FrpTunnelManager::new(config.frp.clone())))
            }
            "bore" if config.bore.enabled => {
                Some(Box::new(BoreTunnelManager::new(config.bore.clone())))
            }
            "cloudflare" if config.cloudflare.enabled => Some(Box::new(
                CloudflareTunnelManager::new(config.cloudflare.clone()),
            )),
            "ngrok" if config.ngrok.enabled => {
                Some(Box::new(NgrokTunnelManager::new(config.ngrok.clone())))
            }
            // Legacy: if no tunnel_provider set but frp is enabled, use frp
            _ if config.frp.enabled => Some(Box::new(FrpTunnelManager::new(config.frp.clone()))),
            _ => None,
        };

        Self {
            config,
            db,
            lan,
            tunnel_manager,
        }
    }

    pub async fn start(&self) -> CiabResult<()> {
        if !self.config.enabled {
            tracing::info!("Gateway subsystem disabled");
            return Ok(());
        }

        self.lan.start().await?;
        tracing::info!("Gateway subsystem started");
        Ok(())
    }

    pub async fn shutdown(&self) -> CiabResult<()> {
        self.lan.stop().await?;
        if let Some(ref tm) = self.tunnel_manager {
            tm.shutdown().await?;
        }
        Ok(())
    }

    pub async fn status(&self) -> CiabResult<GatewayStatus> {
        let tunnel_rows = self.db.list_gateway_tunnel_rows().await?;
        let active_tunnels = tunnel_rows.iter().filter(|t| t.state == "active").count();
        let token_rows = self.db.list_client_token_rows().await?;
        let active_tokens = token_rows.iter().filter(|t| t.revoked_at.is_none()).count();

        let frp_status = FrpStatus {
            enabled: self.config.frp.enabled,
            process_running: self
                .tunnel_manager
                .as_ref()
                .map(|tm| tm.is_running())
                .unwrap_or(false),
            server_addr: self.config.frp.server_addr.clone(),
            proxy_count: if let Some(ref tm) = self.tunnel_manager {
                tm.list_tunnels().await?.len()
            } else {
                0
            },
        };

        let active_provider = self
            .tunnel_manager
            .as_ref()
            .map(|tm| tm.provider_name().to_string())
            .unwrap_or_else(|| self.config.tunnel_provider.clone());

        let providers = self.collect_provider_infos();

        Ok(GatewayStatus {
            enabled: self.config.enabled,
            active_provider,
            lan: self.lan.status(),
            providers,
            frp: frp_status,
            active_tunnels,
            active_tokens,
        })
    }

    /// Collect provider info for all known providers.
    fn collect_provider_infos(&self) -> Vec<TunnelProviderInfo> {
        let mut infos = Vec::new();

        if let Some(ref tm) = self.tunnel_manager {
            infos.push(tm.info());
        }

        // Add info for providers not currently active
        let active_name = self
            .tunnel_manager
            .as_ref()
            .map(|tm| tm.provider_name().to_string())
            .unwrap_or_default();

        for (name, enabled) in [
            ("frp", self.config.frp.enabled),
            ("bore", self.config.bore.enabled),
            ("cloudflare", self.config.cloudflare.enabled),
            ("ngrok", self.config.ngrok.enabled),
        ] {
            if name != active_name {
                infos.push(TunnelProviderInfo {
                    name: name.to_string(),
                    enabled,
                    installed: false,
                    binary_path: None,
                    version: None,
                    process_running: false,
                    tunnel_count: 0,
                });
            }
        }

        infos
    }

    /// Prepare (download/install/validate) a tunnel provider.
    pub async fn prepare_provider(&self, provider: &str) -> CiabResult<ProviderPrepareResult> {
        let (binary, auto_install) = match provider {
            "bore" => (
                self.config.bore.binary.as_str(),
                self.config.bore.auto_install,
            ),
            "cloudflare" => (
                self.config.cloudflare.binary.as_str(),
                self.config.cloudflare.auto_install,
            ),
            "ngrok" => (
                self.config.ngrok.binary.as_str(),
                self.config.ngrok.auto_install,
            ),
            "frp" => (self.config.frp.frpc_binary.as_str(), false),
            other => {
                return Err(CiabError::TunnelProviderError(format!(
                    "Unknown tunnel provider: {}",
                    other
                )));
            }
        };

        tunnel::provider::prepare_provider(provider, binary, auto_install).await
    }

    // --- Token operations ---

    pub async fn create_token(
        &self,
        name: String,
        scopes: Vec<TokenScope>,
        expires_secs: Option<u64>,
    ) -> CiabResult<(String, ClientToken)> {
        let raw_token = generate_token();
        let token_hash = hash_token(&raw_token);
        let now = Utc::now();
        let expires_at = expires_secs.map(|s| now + chrono::Duration::seconds(s as i64));

        let token = ClientToken {
            id: Uuid::new_v4(),
            name,
            token_hash,
            scopes,
            expires_at,
            last_used_at: None,
            created_at: now,
            revoked_at: None,
        };

        let row = token_to_row(&token)?;
        self.db.insert_client_token_row(&row).await?;

        Ok((raw_token, token))
    }

    pub async fn validate_token(&self, raw_token: &str) -> CiabResult<ClientToken> {
        let hash = hash_token(raw_token);
        let row = self.db.get_client_token_row_by_hash(&hash).await?.ok_or(
            CiabError::ClientTokenNotFound("token not found".to_string()),
        )?;

        let token = row_to_token(&row)?;

        if token.revoked_at.is_some() {
            return Err(CiabError::ClientTokenRevoked);
        }

        if let Some(expires_at) = token.expires_at {
            if Utc::now() > expires_at {
                return Err(CiabError::ClientTokenExpired);
            }
        }

        let _ = self.db.touch_client_token_row(&row.id).await;

        Ok(token)
    }

    pub async fn revoke_token(&self, token_id: &Uuid) -> CiabResult<()> {
        self.db.revoke_client_token_row(&token_id.to_string()).await
    }

    pub async fn list_tokens(&self) -> CiabResult<Vec<ClientToken>> {
        let rows = self.db.list_client_token_rows().await?;
        rows.into_iter().map(|r| row_to_token(&r)).collect()
    }

    pub async fn get_token(&self, token_id: &Uuid) -> CiabResult<ClientToken> {
        let row = self
            .db
            .get_client_token_row(&token_id.to_string())
            .await?
            .ok_or_else(|| CiabError::ClientTokenNotFound(token_id.to_string()))?;
        row_to_token(&row)
    }

    // --- Tunnel operations ---

    pub async fn create_tunnel(
        &self,
        sandbox_id: Option<Uuid>,
        tunnel_type: TunnelType,
        local_port: u16,
        public_url: Option<String>,
    ) -> CiabResult<GatewayTunnel> {
        let tunnel = match tunnel_type {
            TunnelType::Frp | TunnelType::Bore | TunnelType::Cloudflare | TunnelType::Ngrok => {
                let tm = self
                    .tunnel_manager
                    .as_ref()
                    .ok_or(CiabError::GatewayNotEnabled)?;
                tm.create_tunnel(sandbox_id, local_port).await?
            }
            TunnelType::Manual | TunnelType::Lan => {
                let url = public_url.ok_or_else(|| {
                    CiabError::TunnelCreationFailed(
                        "public_url required for manual/lan tunnel".to_string(),
                    )
                })?;
                let now = Utc::now();
                GatewayTunnel {
                    id: Uuid::new_v4(),
                    sandbox_id,
                    tunnel_type,
                    public_url: url,
                    local_port,
                    state: TunnelState::Active,
                    config_json: serde_json::json!({}),
                    error_message: None,
                    created_at: now,
                    updated_at: now,
                }
            }
        };

        let row = tunnel_to_row(&tunnel)?;
        self.db.insert_gateway_tunnel_row(&row).await?;
        Ok(tunnel)
    }

    pub async fn stop_tunnel(&self, tunnel_id: &Uuid) -> CiabResult<()> {
        let row = self
            .db
            .get_gateway_tunnel_row(&tunnel_id.to_string())
            .await?
            .ok_or_else(|| CiabError::TunnelNotFound(tunnel_id.to_string()))?;

        // Try to stop via the tunnel manager if it matches the active provider
        let provider_types = ["frp", "bore", "cloudflare", "ngrok"];
        if provider_types.contains(&row.tunnel_type.as_str()) {
            if let Some(ref tm) = self.tunnel_manager {
                let _ = tm.stop_tunnel(tunnel_id).await;
            }
        }

        self.db
            .delete_gateway_tunnel_row(&tunnel_id.to_string())
            .await
    }

    pub async fn list_tunnels(&self) -> CiabResult<Vec<GatewayTunnel>> {
        let rows = self.db.list_gateway_tunnel_rows().await?;
        rows.into_iter().map(|r| row_to_tunnel(&r)).collect()
    }

    pub async fn get_tunnel(&self, tunnel_id: &Uuid) -> CiabResult<GatewayTunnel> {
        let row = self
            .db
            .get_gateway_tunnel_row(&tunnel_id.to_string())
            .await?
            .ok_or_else(|| CiabError::TunnelNotFound(tunnel_id.to_string()))?;
        row_to_tunnel(&row)
    }

    pub async fn expose_sandbox(
        &self,
        sandbox_id: Uuid,
        token_name: Option<String>,
        expires_secs: Option<u64>,
        scope: Option<TokenScope>,
        local_port: u16,
    ) -> CiabResult<ExposeResult> {
        let tunnel_type = if self.tunnel_manager.is_some() {
            self.config
                .tunnel_provider
                .parse::<TunnelType>()
                .unwrap_or(TunnelType::Lan)
        } else {
            TunnelType::Lan
        };

        // For LAN tunnels, compute the public URL from local addresses.
        let public_url = if tunnel_type == TunnelType::Lan {
            let addrs = self.lan.status().local_addresses;
            let addr = addrs
                .first()
                .cloned()
                .unwrap_or_else(|| "127.0.0.1".to_string());
            Some(format!("http://{}:{}", addr, local_port))
        } else {
            None
        };

        let tunnel = self
            .create_tunnel(Some(sandbox_id), tunnel_type, local_port, public_url)
            .await?;

        let name = token_name.unwrap_or_else(|| format!("expose-{}", &sandbox_id.to_string()[..8]));
        let scopes = vec![scope.unwrap_or(TokenScope::SandboxAccess { sandbox_id })];
        let (raw_token, token) = self.create_token(name, scopes, expires_secs).await?;

        Ok(ExposeResult {
            tunnel,
            raw_token,
            token,
        })
    }
}

/// Result of the convenience `expose` operation.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ExposeResult {
    pub tunnel: GatewayTunnel,
    #[serde(rename = "token")]
    pub raw_token: String,
    #[serde(rename = "token_info")]
    pub token: ClientToken,
}

// -------------------------------------------------------------------------
// Row <-> Type conversion helpers
// -------------------------------------------------------------------------

use ciab_db::{ClientTokenRow, GatewayTunnelRow};

fn tunnel_to_row(t: &GatewayTunnel) -> CiabResult<GatewayTunnelRow> {
    Ok(GatewayTunnelRow {
        id: t.id.to_string(),
        sandbox_id: t.sandbox_id.map(|id| id.to_string()),
        tunnel_type: t.tunnel_type.to_string(),
        public_url: t.public_url.clone(),
        local_port: t.local_port as i64,
        state: t.state.to_string(),
        config_json: serde_json::to_string(&t.config_json)?,
        error_message: t.error_message.clone(),
        created_at: t.created_at.to_rfc3339(),
        updated_at: t.updated_at.to_rfc3339(),
    })
}

fn row_to_tunnel(r: &GatewayTunnelRow) -> CiabResult<GatewayTunnel> {
    Ok(GatewayTunnel {
        id: Uuid::parse_str(&r.id).map_err(|e| CiabError::Database(e.to_string()))?,
        sandbox_id: r
            .sandbox_id
            .as_ref()
            .map(|s| Uuid::parse_str(s))
            .transpose()
            .map_err(|e| CiabError::Database(e.to_string()))?,
        tunnel_type: r
            .tunnel_type
            .parse()
            .map_err(|e: String| CiabError::Database(e))?,
        public_url: r.public_url.clone(),
        local_port: r.local_port as u16,
        state: r
            .state
            .parse()
            .map_err(|e: String| CiabError::Database(e))?,
        config_json: serde_json::from_str(&r.config_json)?,
        error_message: r.error_message.clone(),
        created_at: chrono::DateTime::parse_from_rfc3339(&r.created_at)
            .map_err(|e| CiabError::Database(e.to_string()))?
            .with_timezone(&Utc),
        updated_at: chrono::DateTime::parse_from_rfc3339(&r.updated_at)
            .map_err(|e| CiabError::Database(e.to_string()))?
            .with_timezone(&Utc),
    })
}

fn token_to_row(t: &ClientToken) -> CiabResult<ClientTokenRow> {
    Ok(ClientTokenRow {
        id: t.id.to_string(),
        name: t.name.clone(),
        token_hash: t.token_hash.clone(),
        scopes_json: serde_json::to_string(&t.scopes)?,
        expires_at: t.expires_at.map(|d| d.to_rfc3339()),
        last_used_at: t.last_used_at.map(|d| d.to_rfc3339()),
        created_at: t.created_at.to_rfc3339(),
        revoked_at: t.revoked_at.map(|d| d.to_rfc3339()),
    })
}

fn row_to_token(r: &ClientTokenRow) -> CiabResult<ClientToken> {
    Ok(ClientToken {
        id: Uuid::parse_str(&r.id).map_err(|e| CiabError::Database(e.to_string()))?,
        name: r.name.clone(),
        token_hash: r.token_hash.clone(),
        scopes: serde_json::from_str(&r.scopes_json)?,
        expires_at: r
            .expires_at
            .as_ref()
            .map(|s| chrono::DateTime::parse_from_rfc3339(s).map(|d| d.with_timezone(&Utc)))
            .transpose()
            .map_err(|e| CiabError::Database(e.to_string()))?,
        last_used_at: r
            .last_used_at
            .as_ref()
            .map(|s| chrono::DateTime::parse_from_rfc3339(s).map(|d| d.with_timezone(&Utc)))
            .transpose()
            .map_err(|e| CiabError::Database(e.to_string()))?,
        created_at: chrono::DateTime::parse_from_rfc3339(&r.created_at)
            .map_err(|e| CiabError::Database(e.to_string()))?
            .with_timezone(&Utc),
        revoked_at: r
            .revoked_at
            .as_ref()
            .map(|s| chrono::DateTime::parse_from_rfc3339(s).map(|d| d.with_timezone(&Utc)))
            .transpose()
            .map_err(|e| CiabError::Database(e.to_string()))?,
    })
}
