use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub server: ServerConfig,
    pub runtime: RuntimeConfig,
    pub agents: AgentsConfig,
    #[serde(default)]
    pub credentials: CredentialsConfig,
    #[serde(default)]
    pub provisioning: ProvisioningConfig,
    #[serde(default)]
    pub streaming: StreamingConfig,
    #[serde(default)]
    pub security: SecurityConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
    #[serde(default)]
    pub oauth: Option<OAuthConfig>,
    #[serde(default)]
    pub gateway: GatewayConfig,
    #[serde(default)]
    pub channels: ChannelsConfig,
    #[serde(default)]
    pub llm_providers: LlmProvidersConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default)]
    pub workers: Option<usize>,
    #[serde(default = "default_request_timeout")]
    pub request_timeout_secs: u64,
    #[serde(default)]
    pub cors_origins: Vec<String>,
    /// Path to built web UI assets (e.g. desktop/dist). When set, the server
    /// serves the SPA at `/` with index.html fallback so the UI is accessible
    /// via the gateway tunnel URL.
    #[serde(default)]
    pub web_ui_dir: Option<String>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            workers: None,
            request_timeout_secs: default_request_timeout(),
            cors_origins: Vec::new(),
            web_ui_dir: None,
        }
    }
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    9090
}

fn default_request_timeout() -> u64 {
    300
}

#[derive(Debug, Clone, Deserialize)]
pub struct RuntimeConfig {
    /// Runtime backend: "opensandbox", "docker", "local"
    #[serde(default = "default_runtime_backend")]
    pub backend: String,
    /// OpenSandbox URL (only for opensandbox backend)
    #[serde(default)]
    pub opensandbox_url: Option<String>,
    #[serde(default)]
    pub opensandbox_api_key: Option<String>,
    /// Docker socket path (only for docker backend, default: unix:///var/run/docker.sock)
    #[serde(default)]
    pub docker_socket: Option<String>,
    /// Local process working directory (only for local backend)
    #[serde(default)]
    pub local_workdir: Option<String>,
    /// Maximum concurrent local processes (only for local backend)
    #[serde(default)]
    pub local_max_processes: Option<u32>,
}

fn default_runtime_backend() -> String {
    "local".to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct AgentsConfig {
    pub default_provider: String,
    #[serde(default)]
    pub providers: HashMap<String, AgentProviderConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AgentProviderConfig {
    #[serde(default)]
    pub enabled: bool,
    /// Container image (required for docker/opensandbox backends, ignored for local)
    #[serde(default)]
    pub image: Option<String>,
    /// Local binary path (for local backend, e.g. "claude" or "/usr/local/bin/claude")
    #[serde(default)]
    pub binary: Option<String>,
    #[serde(default)]
    pub default_model: Option<String>,
    #[serde(default)]
    pub api_key_env: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CredentialsConfig {
    #[serde(default = "default_credentials_backend")]
    pub backend: String,
    #[serde(default = "default_encryption_key_env")]
    pub encryption_key_env: String,
}

impl Default for CredentialsConfig {
    fn default() -> Self {
        Self {
            backend: default_credentials_backend(),
            encryption_key_env: default_encryption_key_env(),
        }
    }
}

fn default_credentials_backend() -> String {
    "sqlite".to_string()
}

fn default_encryption_key_env() -> String {
    "CIAB_ENCRYPTION_KEY".to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProvisioningConfig {
    #[serde(default = "default_provisioning_timeout")]
    pub timeout_secs: u64,
    #[serde(default = "default_max_script_size")]
    pub max_script_size_bytes: u64,
}

impl Default for ProvisioningConfig {
    fn default() -> Self {
        Self {
            timeout_secs: default_provisioning_timeout(),
            max_script_size_bytes: default_max_script_size(),
        }
    }
}

fn default_provisioning_timeout() -> u64 {
    300
}

fn default_max_script_size() -> u64 {
    1_048_576
}

#[derive(Debug, Clone, Deserialize)]
pub struct StreamingConfig {
    #[serde(default = "default_buffer_size")]
    pub buffer_size: usize,
    #[serde(default = "default_keepalive_interval")]
    pub keepalive_interval_secs: u64,
    #[serde(default = "default_max_stream_duration")]
    pub max_stream_duration_secs: u64,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            buffer_size: default_buffer_size(),
            keepalive_interval_secs: default_keepalive_interval(),
            max_stream_duration_secs: default_max_stream_duration(),
        }
    }
}

fn default_buffer_size() -> usize {
    2000
}

fn default_keepalive_interval() -> u64 {
    15
}

fn default_max_stream_duration() -> u64 {
    3600
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct SecurityConfig {
    #[serde(default)]
    pub api_keys: Vec<String>,
    #[serde(default)]
    pub drop_capabilities: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default = "default_log_format")]
    pub format: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            format: default_log_format(),
        }
    }
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_log_format() -> String {
    "json".to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct OAuthConfig {
    #[serde(default)]
    pub providers: HashMap<String, OAuthProviderConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OAuthProviderConfig {
    pub client_id: String,
    pub client_secret_env: String,
    pub auth_url: String,
    pub token_url: String,
    #[serde(default)]
    pub scopes: Vec<String>,
    pub redirect_uri: String,
}

// -------------------------------------------------------------------------
// Gateway
// -------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GatewayConfig {
    #[serde(default)]
    pub enabled: bool,
    /// Which tunnel provider to use: "frp", "bore", "cloudflare", "ngrok"
    #[serde(default = "default_tunnel_provider")]
    pub tunnel_provider: String,
    #[serde(default)]
    pub lan: LanConfig,
    #[serde(default)]
    pub frp: FrpConfig,
    #[serde(default)]
    pub bore: BoreConfig,
    #[serde(default)]
    pub cloudflare: CloudflareConfig,
    #[serde(default)]
    pub ngrok: NgrokConfig,
    #[serde(default)]
    pub routing: RoutingConfig,
    #[serde(default)]
    pub advanced: AdvancedGatewayConfig,
}

fn default_tunnel_provider() -> String {
    "bore".to_string()
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            tunnel_provider: default_tunnel_provider(),
            lan: LanConfig::default(),
            frp: FrpConfig::default(),
            bore: BoreConfig::default(),
            cloudflare: CloudflareConfig::default(),
            ngrok: NgrokConfig::default(),
            routing: RoutingConfig::default(),
            advanced: AdvancedGatewayConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LanConfig {
    #[serde(default = "default_lan_enabled")]
    pub enabled: bool,
    #[serde(default = "default_mdns_name")]
    pub mdns_name: String,
    #[serde(default = "default_port")]
    pub advertise_port: u16,
}

impl Default for LanConfig {
    fn default() -> Self {
        Self {
            enabled: default_lan_enabled(),
            mdns_name: default_mdns_name(),
            advertise_port: default_port(),
        }
    }
}

fn default_lan_enabled() -> bool {
    true
}

fn default_mdns_name() -> String {
    "ciab".to_string()
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FrpConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_frpc_binary")]
    pub frpc_binary: String,
    #[serde(default)]
    pub server_addr: Option<String>,
    #[serde(default)]
    pub server_port: Option<u16>,
    #[serde(default)]
    pub auth_token: Option<String>,
    #[serde(default)]
    pub subdomain_prefix: Option<String>,
    #[serde(default)]
    pub tls_enable: bool,
    #[serde(default)]
    pub config_template: Option<String>,
}

impl Default for FrpConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            frpc_binary: default_frpc_binary(),
            server_addr: None,
            server_port: None,
            auth_token: None,
            subdomain_prefix: None,
            tls_enable: false,
            config_template: None,
        }
    }
}

fn default_frpc_binary() -> String {
    "frpc".to_string()
}

// --- Bore ---

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BoreConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_bore_binary")]
    pub binary: String,
    #[serde(default = "default_bore_server")]
    pub server: String,
    #[serde(default)]
    pub server_port: Option<u16>,
    #[serde(default)]
    pub secret: Option<String>,
    /// Auto-download bore binary if not found
    #[serde(default = "default_true")]
    pub auto_install: bool,
}

impl Default for BoreConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            binary: default_bore_binary(),
            server: default_bore_server(),
            server_port: None,
            secret: None,
            auto_install: true,
        }
    }
}

fn default_bore_binary() -> String {
    "bore".to_string()
}

fn default_bore_server() -> String {
    "bore.pub".to_string()
}

fn default_true() -> bool {
    true
}

// --- Cloudflare Tunnel ---

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CloudflareConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_cloudflared_binary")]
    pub binary: String,
    /// Tunnel token from Cloudflare dashboard (for named tunnels)
    #[serde(default)]
    pub tunnel_token: Option<String>,
    /// If empty, uses `cloudflared tunnel --url` (quick tunnels, no auth needed)
    #[serde(default)]
    pub tunnel_name: Option<String>,
    /// Auto-download cloudflared binary if not found
    #[serde(default = "default_true")]
    pub auto_install: bool,
}

impl Default for CloudflareConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            binary: default_cloudflared_binary(),
            tunnel_token: None,
            tunnel_name: None,
            auto_install: true,
        }
    }
}

fn default_cloudflared_binary() -> String {
    "cloudflared".to_string()
}

// --- ngrok ---

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NgrokConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_ngrok_binary")]
    pub binary: String,
    /// ngrok authtoken
    #[serde(default)]
    pub authtoken: Option<String>,
    /// Custom domain (paid plans)
    #[serde(default)]
    pub domain: Option<String>,
    /// Region (us, eu, ap, au, sa, jp, in)
    #[serde(default)]
    pub region: Option<String>,
    /// Auto-download ngrok binary if not found
    #[serde(default = "default_true")]
    pub auto_install: bool,
}

impl Default for NgrokConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            binary: default_ngrok_binary(),
            authtoken: None,
            domain: None,
            region: None,
            auto_install: true,
        }
    }
}

fn default_ngrok_binary() -> String {
    "ngrok".to_string()
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RoutingConfig {
    #[serde(default = "default_routing_mode")]
    pub mode: String,
    #[serde(default)]
    pub base_domain: Option<String>,
}

impl Default for RoutingConfig {
    fn default() -> Self {
        Self {
            mode: default_routing_mode(),
            base_domain: None,
        }
    }
}

fn default_routing_mode() -> String {
    "path".to_string()
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct AdvancedGatewayConfig {
    #[serde(default)]
    pub custom_dns_cname: Option<String>,
    #[serde(default)]
    pub k8s_ingress_class: Option<String>,
    #[serde(default)]
    pub k8s_ingress_annotations: HashMap<String, String>,
}

// -------------------------------------------------------------------------
// Channels
// -------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChannelsConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_reaper_interval")]
    pub reaper_interval_secs: u64,
    #[serde(default)]
    pub whatsapp: WhatsAppGlobalConfig,
    #[serde(default)]
    pub slack: SlackGlobalConfig,
    #[serde(default)]
    pub webhook: WebhookGlobalConfig,
}

impl Default for ChannelsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            reaper_interval_secs: default_reaper_interval(),
            whatsapp: WhatsAppGlobalConfig::default(),
            slack: SlackGlobalConfig::default(),
            webhook: WebhookGlobalConfig::default(),
        }
    }
}

fn default_reaper_interval() -> u64 {
    60
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WhatsAppGlobalConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_whatsapp_session_dir")]
    pub session_dir: String,
}

impl Default for WhatsAppGlobalConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            session_dir: default_whatsapp_session_dir(),
        }
    }
}

fn default_whatsapp_session_dir() -> String {
    "/tmp/ciab-whatsapp-sessions".to_string()
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SlackGlobalConfig {
    #[serde(default)]
    pub enabled: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WebhookGlobalConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
}

impl Default for WebhookGlobalConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

// -------------------------------------------------------------------------
// LLM Providers
// -------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct LlmProvidersConfig {
    /// Auto-detect local providers like Ollama on startup.
    #[serde(default = "default_true")]
    pub auto_detect_ollama: bool,
    /// Seed providers from config on first run.
    #[serde(default)]
    pub providers: HashMap<String, LlmProviderSeedConfig>,
}

impl Default for LlmProvidersConfig {
    fn default() -> Self {
        Self {
            auto_detect_ollama: true,
            providers: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct LlmProviderSeedConfig {
    pub kind: String,
    #[serde(default)]
    pub api_key_env: Option<String>,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub default_model: Option<String>,
}
