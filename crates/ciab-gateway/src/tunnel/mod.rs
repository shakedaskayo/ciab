pub mod bore;
pub mod cloudflare;
pub mod frp;
pub mod ngrok;
pub mod provider;

use async_trait::async_trait;
use ciab_core::error::CiabResult;
use uuid::Uuid;

use crate::types::{GatewayTunnel, TunnelProviderInfo};

/// Trait for tunnel backends (bore, cloudflare, frp, ngrok, etc.).
#[async_trait]
pub trait TunnelManager: Send + Sync {
    /// Provider name (e.g. "bore", "cloudflare", "frp", "ngrok").
    fn provider_name(&self) -> &str;

    /// Create a tunnel for a sandbox (or the gateway itself if sandbox_id is None).
    async fn create_tunnel(
        &self,
        sandbox_id: Option<Uuid>,
        local_port: u16,
    ) -> CiabResult<GatewayTunnel>;

    /// Stop and remove a tunnel.
    async fn stop_tunnel(&self, tunnel_id: &Uuid) -> CiabResult<()>;

    /// List all active tunnels.
    async fn list_tunnels(&self) -> CiabResult<Vec<GatewayTunnel>>;

    /// Check if the tunnel backend process is running.
    fn is_running(&self) -> bool;

    /// Gracefully shut down the tunnel manager.
    async fn shutdown(&self) -> CiabResult<()>;

    /// Get provider info (installed, version, running status, etc.).
    fn info(&self) -> TunnelProviderInfo;
}
