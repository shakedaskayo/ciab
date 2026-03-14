use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use dashmap::DashMap;
use tokio::process::Child;
use tokio::sync::Mutex;
use uuid::Uuid;

use ciab_core::error::{CiabError, CiabResult};
use ciab_core::types::config::FrpConfig;

use crate::types::{GatewayTunnel, TunnelProviderInfo, TunnelState, TunnelType};

use super::provider::find_binary;
use super::TunnelManager;

/// An FRP proxy entry.
#[derive(Debug, Clone)]
struct FrpProxy {
    tunnel: GatewayTunnel,
}

/// Manages a single `frpc` process with multiple proxy entries.
pub struct FrpTunnelManager {
    config: FrpConfig,
    proxies: DashMap<Uuid, FrpProxy>,
    process: Arc<Mutex<Option<Child>>>,
    config_path: String,
}

impl FrpTunnelManager {
    pub fn new(config: FrpConfig) -> Self {
        let config_path = std::env::temp_dir()
            .join("ciab-frpc.toml")
            .to_string_lossy()
            .to_string();
        Self {
            config,
            proxies: DashMap::new(),
            process: Arc::new(Mutex::new(None)),
            config_path,
        }
    }

    /// Generate frpc.toml content from current proxies.
    fn generate_config(&self) -> String {
        let server_addr = self.config.server_addr.as_deref().unwrap_or("127.0.0.1");
        let server_port = self.config.server_port.unwrap_or(7000);

        let mut config = format!(
            r#"serverAddr = "{}"
serverPort = {}
"#,
            server_addr, server_port
        );

        if let Some(ref token) = self.config.auth_token {
            config.push_str(&format!(
                r#"
[auth]
method = "token"
token = "{}"
"#,
                token
            ));
        }

        if self.config.tls_enable {
            config.push_str(
                r#"
[transport]
tls.enable = true
"#,
            );
        }

        for entry in self.proxies.iter() {
            let proxy = entry.value();
            let tunnel = &proxy.tunnel;
            let name = if let Some(sid) = &tunnel.sandbox_id {
                format!("sandbox-{}", sid)
            } else {
                "ciab-gateway".to_string()
            };

            config.push_str(&format!(
                r#"
[[proxies]]
name = "{}"
type = "tcp"
localIP = "127.0.0.1"
localPort = {}
remotePort = {}
"#,
                name,
                tunnel.local_port,
                tunnel.local_port, // remote port defaults to same; frps assigns if needed
            ));

            if let Some(ref prefix) = self.config.subdomain_prefix {
                let subdomain = if let Some(sid) = &tunnel.sandbox_id {
                    format!("{}-{}", prefix, &sid.to_string()[..8])
                } else {
                    prefix.clone()
                };
                // For HTTP type proxies, add subdomain
                config.push_str(&format!("subdomain = \"{}\"\n", subdomain));
            }
        }

        config
    }

    /// Write config and (re)start the frpc process.
    async fn reload(&self) -> CiabResult<()> {
        let config_content = self.generate_config();
        tokio::fs::write(&self.config_path, &config_content)
            .await
            .map_err(|e| CiabError::FrpError(format!("failed to write frpc config: {}", e)))?;

        let mut proc_guard = self.process.lock().await;

        // Kill existing process if running.
        if let Some(ref mut child) = *proc_guard {
            let _ = child.kill().await;
        }

        if self.proxies.is_empty() {
            *proc_guard = None;
            return Ok(());
        }

        let binary = &self.config.frpc_binary;
        let child = tokio::process::Command::new(binary)
            .arg("-c")
            .arg(&self.config_path)
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| {
                CiabError::FrpError(format!("failed to start frpc ({}): {}", binary, e))
            })?;

        tracing::info!(pid = ?child.id(), "frpc process started");
        *proc_guard = Some(child);

        Ok(())
    }
}

#[async_trait]
impl TunnelManager for FrpTunnelManager {
    fn provider_name(&self) -> &str {
        "frp"
    }

    async fn create_tunnel(
        &self,
        sandbox_id: Option<Uuid>,
        local_port: u16,
    ) -> CiabResult<GatewayTunnel> {
        let server_addr = self.config.server_addr.as_deref().unwrap_or("127.0.0.1");
        let public_url = if let Some(ref prefix) = self.config.subdomain_prefix {
            let subdomain = if let Some(sid) = &sandbox_id {
                format!("{}-{}", prefix, &sid.to_string()[..8])
            } else {
                prefix.clone()
            };
            format!("https://{}.{}", subdomain, server_addr)
        } else {
            format!("tcp://{}:{}", server_addr, local_port)
        };

        let now = Utc::now();
        let tunnel = GatewayTunnel {
            id: Uuid::new_v4(),
            sandbox_id,
            tunnel_type: TunnelType::Frp,
            public_url,
            local_port,
            state: TunnelState::Active,
            config_json: serde_json::json!({}),
            error_message: None,
            created_at: now,
            updated_at: now,
        };

        self.proxies.insert(
            tunnel.id,
            FrpProxy {
                tunnel: tunnel.clone(),
            },
        );

        if let Err(e) = self.reload().await {
            self.proxies.remove(&tunnel.id);
            return Err(e);
        }

        Ok(tunnel)
    }

    async fn stop_tunnel(&self, tunnel_id: &Uuid) -> CiabResult<()> {
        self.proxies
            .remove(tunnel_id)
            .ok_or_else(|| CiabError::TunnelNotFound(tunnel_id.to_string()))?;

        self.reload().await
    }

    async fn list_tunnels(&self) -> CiabResult<Vec<GatewayTunnel>> {
        Ok(self
            .proxies
            .iter()
            .map(|e| e.value().tunnel.clone())
            .collect())
    }

    fn is_running(&self) -> bool {
        // Check without awaiting — best effort.
        // The process field requires async lock so we just report based on proxy count.
        !self.proxies.is_empty()
    }

    async fn shutdown(&self) -> CiabResult<()> {
        let mut proc_guard = self.process.lock().await;
        if let Some(ref mut child) = *proc_guard {
            let _ = child.kill().await;
        }
        *proc_guard = None;
        self.proxies.clear();
        let _ = tokio::fs::remove_file(&self.config_path).await;
        Ok(())
    }

    fn info(&self) -> TunnelProviderInfo {
        let binary = &self.config.frpc_binary;
        let installed = find_binary(binary).is_some();
        TunnelProviderInfo {
            name: "frp".to_string(),
            enabled: true, // FrpTunnelManager is only created when enabled
            installed,
            binary_path: find_binary(binary).map(|p| p.to_string_lossy().to_string()),
            version: None,
            process_running: self.is_running(),
            tunnel_count: self.proxies.len(),
        }
    }
}
