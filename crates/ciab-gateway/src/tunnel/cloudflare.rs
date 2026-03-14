use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use dashmap::DashMap;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Child;
use tokio::sync::Mutex;
use uuid::Uuid;

use ciab_core::error::{CiabError, CiabResult};
use ciab_core::types::config::CloudflareConfig;

use crate::types::{GatewayTunnel, TunnelProviderInfo, TunnelState, TunnelType};

use super::provider::find_binary;
use super::TunnelManager;

struct CfTunnel {
    tunnel: GatewayTunnel,
    process: Arc<Mutex<Option<Child>>>,
}

/// Manages Cloudflare Tunnel (cloudflared) instances.
///
/// Two modes:
/// - **Quick tunnel** (no token/name): `cloudflared tunnel --url http://localhost:PORT`
///   Gives a free *.trycloudflare.com URL.
/// - **Named tunnel** (with token): `cloudflared tunnel run --token TOKEN`
///   Uses a pre-configured tunnel from the Cloudflare dashboard.
pub struct CloudflareTunnelManager {
    config: CloudflareConfig,
    tunnels: DashMap<Uuid, Arc<CfTunnel>>,
}

impl CloudflareTunnelManager {
    pub fn new(config: CloudflareConfig) -> Self {
        Self {
            config,
            tunnels: DashMap::new(),
        }
    }

    fn resolve_binary(&self) -> String {
        let local_binary = dirs::data_local_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
            .join("ciab")
            .join("bin")
            .join(&self.config.binary);
        if local_binary.exists() {
            return local_binary.to_string_lossy().to_string();
        }
        self.config.binary.clone()
    }
}

#[async_trait]
impl TunnelManager for CloudflareTunnelManager {
    fn provider_name(&self) -> &str {
        "cloudflare"
    }

    async fn create_tunnel(
        &self,
        sandbox_id: Option<Uuid>,
        local_port: u16,
    ) -> CiabResult<GatewayTunnel> {
        let binary = self.resolve_binary();

        let mut cmd = tokio::process::Command::new(&binary);

        if let Some(ref token) = self.config.tunnel_token {
            // Named tunnel mode
            cmd.arg("tunnel").arg("run").arg("--token").arg(token);
        } else {
            // Quick tunnel mode (free *.trycloudflare.com)
            cmd.arg("tunnel")
                .arg("--url")
                .arg(format!("http://localhost:{}", local_port));
        }

        cmd.stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true);

        let mut child = cmd.spawn().map_err(|e| {
            CiabError::TunnelProviderError(format!(
                "Failed to start cloudflared ({}): {}",
                binary, e
            ))
        })?;

        // Parse the public URL from cloudflared output.
        // Quick tunnel prints something like:
        //   "Your quick Tunnel has been created! Visit it at (it may take some time to be reachable):
        //    https://something-random.trycloudflare.com"
        // or: "INF |  https://something.trycloudflare.com"
        let stderr = child.stderr.take();
        let public_url = if let Some(stderr) = stderr {
            let mut reader = BufReader::new(stderr).lines();
            let mut url = String::new();

            for _ in 0..30 {
                match tokio::time::timeout(std::time::Duration::from_secs(30), reader.next_line())
                    .await
                {
                    Ok(Ok(Some(line))) => {
                        tracing::debug!(line = %line, "cloudflared output");
                        // Look for the tunnel URL in the output
                        if let Some(pos) = line.find("https://") {
                            let rest = &line[pos..];
                            let end = rest
                                .find(|c: char| c.is_whitespace() || c == '"' || c == '\'')
                                .unwrap_or(rest.len());
                            url = rest[..end].to_string();
                            break;
                        }
                    }
                    Ok(Ok(None)) => break,
                    Ok(Err(_)) => break,
                    Err(_) => {
                        tracing::warn!("Timeout reading cloudflared output");
                        break;
                    }
                }
            }

            if url.is_empty() {
                "https://pending-cloudflare-tunnel.trycloudflare.com".to_string()
            } else {
                url
            }
        } else if let Some(ref name) = self.config.tunnel_name {
            format!("https://{}", name)
        } else {
            "https://pending-cloudflare-tunnel.trycloudflare.com".to_string()
        };

        let now = Utc::now();
        let tunnel = GatewayTunnel {
            id: Uuid::new_v4(),
            sandbox_id,
            tunnel_type: TunnelType::Cloudflare,
            public_url,
            local_port,
            state: TunnelState::Active,
            config_json: serde_json::json!({
                "provider": "cloudflare",
                "mode": if self.config.tunnel_token.is_some() { "named" } else { "quick" },
            }),
            error_message: None,
            created_at: now,
            updated_at: now,
        };

        tracing::info!(
            tunnel_id = %tunnel.id,
            public_url = %tunnel.public_url,
            pid = ?child.id(),
            "cloudflare tunnel created"
        );

        let cf_tunnel = Arc::new(CfTunnel {
            tunnel: tunnel.clone(),
            process: Arc::new(Mutex::new(Some(child))),
        });

        self.tunnels.insert(tunnel.id, cf_tunnel);
        Ok(tunnel)
    }

    async fn stop_tunnel(&self, tunnel_id: &Uuid) -> CiabResult<()> {
        let (_, ct) = self
            .tunnels
            .remove(tunnel_id)
            .ok_or_else(|| CiabError::TunnelNotFound(tunnel_id.to_string()))?;

        let mut guard = ct.process.lock().await;
        if let Some(ref mut child) = *guard {
            let _ = child.kill().await;
        }
        *guard = None;

        tracing::info!(tunnel_id = %tunnel_id, "cloudflare tunnel stopped");
        Ok(())
    }

    async fn list_tunnels(&self) -> CiabResult<Vec<GatewayTunnel>> {
        Ok(self
            .tunnels
            .iter()
            .map(|e| e.value().tunnel.clone())
            .collect())
    }

    fn is_running(&self) -> bool {
        !self.tunnels.is_empty()
    }

    async fn shutdown(&self) -> CiabResult<()> {
        let ids: Vec<Uuid> = self.tunnels.iter().map(|e| *e.key()).collect();
        for id in ids {
            let _ = self.stop_tunnel(&id).await;
        }
        Ok(())
    }

    fn info(&self) -> TunnelProviderInfo {
        let binary = self.resolve_binary();
        let installed = find_binary(&binary).is_some();
        TunnelProviderInfo {
            name: "cloudflare".to_string(),
            enabled: self.config.enabled,
            installed,
            binary_path: find_binary(&binary).map(|p| p.to_string_lossy().to_string()),
            version: None,
            process_running: self.is_running(),
            tunnel_count: self.tunnels.len(),
        }
    }
}
