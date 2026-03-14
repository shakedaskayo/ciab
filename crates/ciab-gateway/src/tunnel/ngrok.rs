use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use dashmap::DashMap;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Child;
use tokio::sync::Mutex;
use uuid::Uuid;

use ciab_core::error::{CiabError, CiabResult};
use ciab_core::types::config::NgrokConfig;

use crate::types::{GatewayTunnel, TunnelProviderInfo, TunnelState, TunnelType};

use super::provider::find_binary;
use super::TunnelManager;

struct NgrokTunnel {
    tunnel: GatewayTunnel,
    process: Arc<Mutex<Option<Child>>>,
}

/// Manages ngrok tunnels — each tunnel gets its own `ngrok http` process.
pub struct NgrokTunnelManager {
    config: NgrokConfig,
    tunnels: DashMap<Uuid, Arc<NgrokTunnel>>,
}

impl NgrokTunnelManager {
    pub fn new(config: NgrokConfig) -> Self {
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
impl TunnelManager for NgrokTunnelManager {
    fn provider_name(&self) -> &str {
        "ngrok"
    }

    async fn create_tunnel(
        &self,
        sandbox_id: Option<Uuid>,
        local_port: u16,
    ) -> CiabResult<GatewayTunnel> {
        let binary = self.resolve_binary();

        let mut cmd = tokio::process::Command::new(&binary);
        cmd.arg("http")
            .arg(local_port.to_string())
            .arg("--log")
            .arg("stdout");

        if let Some(ref token) = self.config.authtoken {
            cmd.arg("--authtoken").arg(token);
        }

        if let Some(ref domain) = self.config.domain {
            cmd.arg("--domain").arg(domain);
        }

        if let Some(ref region) = self.config.region {
            cmd.arg("--region").arg(region);
        }

        cmd.stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true);

        let mut child = cmd.spawn().map_err(|e| {
            CiabError::TunnelProviderError(format!("Failed to start ngrok ({}): {}", binary, e))
        })?;

        // Parse the public URL from ngrok's log output.
        // ngrok with --log stdout prints lines like:
        //   "msg=\"started tunnel\" ... url=https://xxxx.ngrok-free.app"
        let stdout = child.stdout.take();
        let public_url = if let Some(stdout) = stdout {
            let mut reader = BufReader::new(stdout).lines();
            let mut url = String::new();

            for _ in 0..30 {
                match tokio::time::timeout(std::time::Duration::from_secs(15), reader.next_line())
                    .await
                {
                    Ok(Ok(Some(line))) => {
                        tracing::debug!(line = %line, "ngrok output");
                        // Look for url= in log output
                        if let Some(pos) = line.find("url=") {
                            let rest = &line[pos + 4..];
                            let end = rest
                                .find(|c: char| c.is_whitespace() || c == '"')
                                .unwrap_or(rest.len());
                            url = rest[..end].to_string();
                            if url.starts_with("https://") {
                                break;
                            }
                        }
                        // Also check for Forwarding line format
                        if line.contains("Forwarding") {
                            if let Some(pos) = line.find("https://") {
                                let rest = &line[pos..];
                                let end = rest
                                    .find(|c: char| c.is_whitespace() || c == '"' || c == '-')
                                    .unwrap_or(rest.len());
                                url = rest[..end].to_string();
                                break;
                            }
                        }
                    }
                    Ok(Ok(None)) => break,
                    Ok(Err(_)) => break,
                    Err(_) => {
                        tracing::warn!("Timeout reading ngrok output");
                        break;
                    }
                }
            }

            if url.is_empty() {
                // Fallback: try the ngrok API
                if let Some(api_url) = query_ngrok_api().await {
                    api_url
                } else {
                    format!("https://pending-ngrok-tunnel.ngrok-free.app")
                }
            } else {
                url
            }
        } else {
            "https://pending-ngrok-tunnel.ngrok-free.app".to_string()
        };

        let now = Utc::now();
        let tunnel = GatewayTunnel {
            id: Uuid::new_v4(),
            sandbox_id,
            tunnel_type: TunnelType::Ngrok,
            public_url,
            local_port,
            state: TunnelState::Active,
            config_json: serde_json::json!({
                "provider": "ngrok",
                "region": self.config.region,
            }),
            error_message: None,
            created_at: now,
            updated_at: now,
        };

        tracing::info!(
            tunnel_id = %tunnel.id,
            public_url = %tunnel.public_url,
            pid = ?child.id(),
            "ngrok tunnel created"
        );

        let ngrok_tunnel = Arc::new(NgrokTunnel {
            tunnel: tunnel.clone(),
            process: Arc::new(Mutex::new(Some(child))),
        });

        self.tunnels.insert(tunnel.id, ngrok_tunnel);
        Ok(tunnel)
    }

    async fn stop_tunnel(&self, tunnel_id: &Uuid) -> CiabResult<()> {
        let (_, nt) = self
            .tunnels
            .remove(tunnel_id)
            .ok_or_else(|| CiabError::TunnelNotFound(tunnel_id.to_string()))?;

        let mut guard = nt.process.lock().await;
        if let Some(ref mut child) = *guard {
            let _ = child.kill().await;
        }
        *guard = None;

        tracing::info!(tunnel_id = %tunnel_id, "ngrok tunnel stopped");
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
            name: "ngrok".to_string(),
            enabled: self.config.enabled,
            installed,
            binary_path: find_binary(&binary).map(|p| p.to_string_lossy().to_string()),
            version: None,
            process_running: self.is_running(),
            tunnel_count: self.tunnels.len(),
        }
    }
}

/// Try to query the ngrok local API (http://127.0.0.1:4040/api/tunnels) for the public URL.
async fn query_ngrok_api() -> Option<String> {
    // Give ngrok a moment to start its API
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .ok()?;

    let resp = client
        .get("http://127.0.0.1:4040/api/tunnels")
        .send()
        .await
        .ok()?;

    let json: serde_json::Value = resp.json().await.ok()?;
    let tunnels = json.get("tunnels")?.as_array()?;

    for t in tunnels {
        if let Some(url) = t.get("public_url").and_then(|u| u.as_str()) {
            if url.starts_with("https://") {
                return Some(url.to_string());
            }
        }
    }
    None
}
