use ciab_core::error::{CiabError, CiabResult};
use ciab_core::types::config::LanConfig;
use mdns_sd::{ServiceDaemon, ServiceInfo};
use tokio::sync::Mutex;

use crate::types::LanStatus;

const SERVICE_TYPE: &str = "_ciab._tcp.local.";

/// Manages mDNS/DNS-SD advertisement for LAN discovery.
pub struct LanDiscovery {
    config: LanConfig,
    daemon: Mutex<Option<ServiceDaemon>>,
}

impl LanDiscovery {
    pub fn new(config: LanConfig) -> Self {
        Self {
            config,
            daemon: Mutex::new(None),
        }
    }

    /// Start advertising the CIAB service via mDNS.
    pub async fn start(&self) -> CiabResult<()> {
        if !self.config.enabled {
            tracing::info!("LAN discovery disabled");
            return Ok(());
        }

        let daemon = ServiceDaemon::new()
            .map_err(|e| CiabError::Internal(format!("mDNS daemon creation failed: {}", e)))?;

        let hostname = format!("{}.local.", self.config.mdns_name);
        let service_info = ServiceInfo::new(
            SERVICE_TYPE,
            &self.config.mdns_name,
            &hostname,
            "",
            self.config.advertise_port,
            [("version", env!("CARGO_PKG_VERSION")), ("api", "/api/v1")].as_ref(),
        )
        .map_err(|e| CiabError::Internal(format!("mDNS service info creation failed: {}", e)))?;

        daemon
            .register(service_info)
            .map_err(|e| CiabError::Internal(format!("mDNS registration failed: {}", e)))?;

        tracing::info!(
            name = %self.config.mdns_name,
            port = self.config.advertise_port,
            "mDNS service registered"
        );

        *self.daemon.lock().await = Some(daemon);
        Ok(())
    }

    /// Stop mDNS advertisement.
    pub async fn stop(&self) -> CiabResult<()> {
        let mut guard = self.daemon.lock().await;
        if let Some(daemon) = guard.take() {
            let _ = daemon.shutdown();
        }
        Ok(())
    }

    /// Get current LAN status (local addresses, mDNS name, etc.).
    pub fn status(&self) -> LanStatus {
        LanStatus {
            enabled: self.config.enabled,
            mdns_name: if self.config.enabled {
                Some(format!("{}.local", self.config.mdns_name))
            } else {
                None
            },
            local_addresses: enumerate_local_addresses(),
            advertise_port: self.config.advertise_port,
        }
    }

    /// Browse the LAN for CIAB services.
    pub fn discover() -> CiabResult<Vec<DiscoveredService>> {
        let daemon = ServiceDaemon::new()
            .map_err(|e| CiabError::Internal(format!("mDNS browse failed: {}", e)))?;

        let receiver = daemon
            .browse(SERVICE_TYPE)
            .map_err(|e| CiabError::Internal(format!("mDNS browse failed: {}", e)))?;

        let mut services = Vec::new();

        // Poll for a short duration to find services.
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
        while std::time::Instant::now() < deadline {
            match receiver.try_recv() {
                Ok(event) => {
                    if let mdns_sd::ServiceEvent::ServiceResolved(info) = event {
                        services.push(DiscoveredService {
                            name: info.get_fullname().to_string(),
                            host: info.get_hostname().to_string(),
                            port: info.get_port(),
                            addresses: info.get_addresses().iter().map(|a| a.to_string()).collect(),
                        });
                    }
                }
                Err(_) => {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            }
        }

        let _ = daemon.shutdown();
        Ok(services)
    }
}

/// A CIAB service discovered on the LAN.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiscoveredService {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub addresses: Vec<String>,
}

/// Enumerate non-loopback local IP addresses.
fn enumerate_local_addresses() -> Vec<String> {
    let mut addrs = Vec::new();
    if let Ok(interfaces) = std::net::UdpSocket::bind("0.0.0.0:0") {
        // Use a connect trick to find the default route address.
        if interfaces.connect("8.8.8.8:80").is_ok() {
            if let Ok(local_addr) = interfaces.local_addr() {
                addrs.push(local_addr.ip().to_string());
            }
        }
    }

    // Also try to get all interface addresses via DNS lookup of hostname.
    if let Ok(hostname) = hostname::get() {
        if let Some(name) = hostname.to_str() {
            if let Ok(resolved) = std::net::ToSocketAddrs::to_socket_addrs(&(name, 0)) {
                for addr in resolved {
                    let ip = addr.ip();
                    if !ip.is_loopback() && !addrs.contains(&ip.to_string()) {
                        addrs.push(ip.to_string());
                    }
                }
            }
        }
    }

    if addrs.is_empty() {
        // Fallback
        addrs.push("127.0.0.1".to_string());
    }
    addrs
}
