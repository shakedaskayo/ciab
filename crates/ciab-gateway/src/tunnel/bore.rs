use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use dashmap::DashMap;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Child;
use tokio::sync::Mutex;
use uuid::Uuid;

use ciab_core::error::{CiabError, CiabResult};
use ciab_core::types::config::BoreConfig;

use crate::types::{GatewayTunnel, TunnelProviderInfo, TunnelState, TunnelType};

use super::provider::find_binary;
use super::TunnelManager;

struct BoreTunnel {
    tunnel: GatewayTunnel,
    process: Arc<Mutex<Option<Child>>>,
}

/// Manages bore tunnels — each tunnel gets its own `bore local` process.
pub struct BoreTunnelManager {
    config: BoreConfig,
    tunnels: DashMap<Uuid, Arc<BoreTunnel>>,
}

impl BoreTunnelManager {
    pub fn new(config: BoreConfig) -> Self {
        Self {
            config,
            tunnels: DashMap::new(),
        }
    }

    fn resolve_binary(&self) -> String {
        // Check ciab local install dir first
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
impl TunnelManager for BoreTunnelManager {
    fn provider_name(&self) -> &str {
        "bore"
    }

    async fn create_tunnel(
        &self,
        sandbox_id: Option<Uuid>,
        local_port: u16,
    ) -> CiabResult<GatewayTunnel> {
        let binary = self.resolve_binary();

        let mut cmd = tokio::process::Command::new(&binary);
        cmd.arg("local")
            .arg(local_port.to_string())
            .arg("--to")
            .arg(&self.config.server);

        if let Some(port) = self.config.server_port {
            cmd.arg("--port").arg(port.to_string());
        }

        if let Some(ref secret) = self.config.secret {
            cmd.arg("--secret").arg(secret);
        }

        cmd.stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true);

        let mut child = cmd.spawn().map_err(|e| {
            CiabError::TunnelProviderError(format!("Failed to start bore ({}): {}", binary, e))
        })?;

        // Parse the public URL from bore's stderr output.
        // bore prints: "listening at bore.pub:PORT"
        let stderr = child.stderr.take();
        let public_url = if let Some(stderr) = stderr {
            let mut reader = BufReader::new(stderr).lines();
            let mut url = format!("{}:{}", self.config.server, local_port);

            // Read up to 10 lines looking for the listening line
            for _ in 0..10 {
                match tokio::time::timeout(std::time::Duration::from_secs(10), reader.next_line())
                    .await
                {
                    Ok(Ok(Some(line))) => {
                        tracing::debug!(line = %line, "bore output");
                        if line.contains("listening at") {
                            // Extract "host:port" from the line
                            if let Some(addr) = line.split("listening at ").nth(1) {
                                let addr = addr.trim();
                                url = format!("http://{}", addr);
                                break;
                            }
                        }
                        if line.contains("remote_port:") || line.contains("remote port:") {
                            if let Some(port_str) = line.split(':').next_back() {
                                if let Ok(remote_port) = port_str.trim().parse::<u16>() {
                                    url = format!("http://{}:{}", self.config.server, remote_port);
                                    break;
                                }
                            }
                        }
                    }
                    Ok(Ok(None)) => break,
                    Ok(Err(_)) => break,
                    Err(_) => {
                        tracing::warn!("Timeout reading bore output, using default URL");
                        break;
                    }
                }
            }
            url
        } else {
            format!("http://{}:{}", self.config.server, local_port)
        };

        let now = Utc::now();
        let tunnel = GatewayTunnel {
            id: Uuid::new_v4(),
            sandbox_id,
            tunnel_type: TunnelType::Bore,
            public_url,
            local_port,
            state: TunnelState::Active,
            config_json: serde_json::json!({
                "server": self.config.server,
                "provider": "bore",
            }),
            error_message: None,
            created_at: now,
            updated_at: now,
        };

        tracing::info!(
            tunnel_id = %tunnel.id,
            public_url = %tunnel.public_url,
            pid = ?child.id(),
            "bore tunnel created"
        );

        let bore_tunnel = Arc::new(BoreTunnel {
            tunnel: tunnel.clone(),
            process: Arc::new(Mutex::new(Some(child))),
        });

        self.tunnels.insert(tunnel.id, bore_tunnel);
        Ok(tunnel)
    }

    async fn stop_tunnel(&self, tunnel_id: &Uuid) -> CiabResult<()> {
        let (_, bt) = self
            .tunnels
            .remove(tunnel_id)
            .ok_or_else(|| CiabError::TunnelNotFound(tunnel_id.to_string()))?;

        let mut guard = bt.process.lock().await;
        if let Some(ref mut child) = *guard {
            let _ = child.kill().await;
        }
        *guard = None;

        tracing::info!(tunnel_id = %tunnel_id, "bore tunnel stopped");
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
            name: "bore".to_string(),
            enabled: self.config.enabled,
            installed,
            binary_path: find_binary(&binary).map(|p| p.to_string_lossy().to_string()),
            version: None,
            process_running: self.is_running(),
            tunnel_count: self.tunnels.len(),
        }
    }
}
