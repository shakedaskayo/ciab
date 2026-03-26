pub mod agent;
pub mod config;
pub mod credential;
pub mod files;
pub mod gateway;
pub mod image;
pub mod oauth;
pub mod sandbox;
pub mod server;
pub mod session;
pub mod workspace;

use crate::client::CiabClient;
use crate::output::OutputFormat;

#[derive(clap::Subcommand)]
pub enum Commands {
    /// Manage sandboxes
    Sandbox {
        #[command(subcommand)]
        command: SandboxCommand,
    },
    /// Interact with agents
    Agent {
        #[command(subcommand)]
        command: AgentCommand,
    },
    /// Manage sessions
    Session {
        #[command(subcommand)]
        command: SessionCommand,
    },
    /// Manage files in sandboxes
    Files {
        #[command(subcommand)]
        command: FilesCommand,
    },
    /// Manage credentials
    Credential {
        #[command(subcommand)]
        command: CredentialCommand,
    },
    /// OAuth authentication
    Oauth {
        #[command(subcommand)]
        command: OAuthCommand,
    },
    /// Configuration management
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    /// Start the API server
    Server {
        #[command(subcommand)]
        command: ServerCommand,
    },
    /// Manage workspaces (environment definitions)
    Workspace {
        #[command(subcommand)]
        command: WorkspaceCommand,
    },
    /// Gateway, tunnels, and remote access
    Gateway {
        #[command(subcommand)]
        command: GatewayCommand,
    },
    /// Manage machine images (Packer builds)
    Image {
        #[command(subcommand)]
        command: ImageCommand,
    },
}

// -------------------------------------------------------------------------
// Sandbox subcommands
// -------------------------------------------------------------------------

#[derive(clap::Subcommand)]
pub enum SandboxCommand {
    /// Create a new sandbox
    Create {
        /// Agent provider (e.g. claude-code, codex, gemini, cursor)
        #[arg(long, default_value = "claude-code")]
        provider: String,

        /// Sandbox name
        #[arg(long)]
        name: Option<String>,

        /// Container image
        #[arg(long)]
        image: Option<String>,

        /// CPU cores
        #[arg(long)]
        cpu: Option<f32>,

        /// Memory in MB
        #[arg(long)]
        memory: Option<u32>,

        /// Disk in MB
        #[arg(long)]
        disk: Option<u32>,

        /// Environment variables (KEY=VALUE, can repeat)
        #[arg(long = "env", short = 'e')]
        env_vars: Vec<String>,

        /// Git repo URL to clone
        #[arg(long)]
        git_repo: Option<String>,

        /// Credential IDs to inject
        #[arg(long)]
        credential: Vec<String>,

        /// Timeout in seconds
        #[arg(long)]
        timeout: Option<u32>,
    },
    /// List sandboxes
    List {
        /// Filter by state
        #[arg(long)]
        state: Option<String>,

        /// Filter by provider
        #[arg(long)]
        provider: Option<String>,
    },
    /// Get sandbox details
    Get {
        /// Sandbox ID
        id: String,
    },
    /// Delete a sandbox
    Delete {
        /// Sandbox ID
        id: String,
    },
    /// Start a sandbox
    Start {
        /// Sandbox ID
        id: String,
    },
    /// Stop a sandbox
    Stop {
        /// Sandbox ID
        id: String,
    },
    /// Pause a sandbox
    Pause {
        /// Sandbox ID
        id: String,
    },
    /// Resume a sandbox
    Resume {
        /// Sandbox ID
        id: String,
    },
    /// Get sandbox resource stats
    Stats {
        /// Sandbox ID
        id: String,
    },
    /// Get sandbox logs
    Logs {
        /// Sandbox ID
        id: String,

        /// Follow log output
        #[arg(long, short)]
        follow: bool,

        /// Number of lines to show from the end
        #[arg(long)]
        tail: Option<u32>,
    },
    /// Execute a command in a sandbox
    Exec {
        /// Sandbox ID
        id: String,

        /// Command and arguments
        #[arg(trailing_var_arg = true, required = true)]
        command: Vec<String>,

        /// Working directory
        #[arg(long)]
        workdir: Option<String>,
    },
}

// -------------------------------------------------------------------------
// Agent subcommands
// -------------------------------------------------------------------------

#[derive(clap::Subcommand)]
pub enum AgentCommand {
    /// Start an interactive chat session
    Chat {
        /// Sandbox ID
        #[arg(long)]
        sandbox_id: String,

        /// Session ID (creates new if not specified)
        #[arg(long)]
        session_id: Option<String>,

        /// Single message (non-interactive)
        #[arg(long)]
        message: Option<String>,

        /// Interactive mode
        #[arg(long, short)]
        interactive: bool,

        /// Use SSE streaming
        #[arg(long)]
        stream: bool,
    },
    /// Attach to a running session stream
    Attach {
        /// Session ID
        session_id: String,
    },
    /// Interrupt a running session
    Interrupt {
        /// Session ID
        session_id: String,
    },
    /// List available agent providers
    Providers,
}

// -------------------------------------------------------------------------
// Session subcommands
// -------------------------------------------------------------------------

#[derive(clap::Subcommand)]
pub enum SessionCommand {
    /// Create a new session
    Create {
        /// Sandbox ID
        sandbox_id: String,
    },
    /// List sessions for a sandbox
    List {
        /// Sandbox ID
        sandbox_id: String,
    },
    /// Get session details
    Get {
        /// Session ID
        id: String,
    },
    /// Send a message to a session
    Send {
        /// Session ID
        session_id: String,

        /// Message text
        message: String,
    },
    /// Interrupt a session
    Interrupt {
        /// Session ID
        id: String,
    },
}

// -------------------------------------------------------------------------
// Files subcommands
// -------------------------------------------------------------------------

#[derive(clap::Subcommand)]
pub enum FilesCommand {
    /// List files in a sandbox
    List {
        /// Sandbox ID
        sandbox_id: String,

        /// Path pattern
        #[arg(long, default_value = "/")]
        path: String,
    },
    /// Upload a file to a sandbox
    Upload {
        /// Sandbox ID
        sandbox_id: String,

        /// Local file path
        local_path: String,

        /// Remote file path in sandbox
        remote_path: String,
    },
    /// Download a file from a sandbox
    Download {
        /// Sandbox ID
        sandbox_id: String,

        /// Remote file path in sandbox
        remote_path: String,

        /// Local file path to save to
        local_path: String,
    },
    /// Delete a file in a sandbox
    Delete {
        /// Sandbox ID
        sandbox_id: String,

        /// Remote file path
        remote_path: String,
    },
}

// -------------------------------------------------------------------------
// Credential subcommands
// -------------------------------------------------------------------------

#[derive(clap::Subcommand)]
pub enum CredentialCommand {
    /// Create a credential
    Create {
        /// Credential name
        name: String,

        /// Credential type (api_key, env_vars, git_token, oauth_token, ssh_key, file)
        #[arg(long, default_value = "api_key")]
        cred_type: String,

        /// Credential value
        value: String,
    },
    /// List credentials
    List,
    /// Get credential details
    Get {
        /// Credential ID
        id: String,
    },
    /// Delete a credential
    Delete {
        /// Credential ID
        id: String,
    },
}

// -------------------------------------------------------------------------
// OAuth subcommands
// -------------------------------------------------------------------------

#[derive(clap::Subcommand)]
pub enum OAuthCommand {
    /// Start OAuth authorization flow
    Authorize {
        /// OAuth provider (github, gcp, aws, azure)
        provider: String,
    },
    /// Start device code flow
    DeviceCode {
        /// OAuth provider
        provider: String,
    },
    /// Poll device code status
    DevicePoll {
        /// OAuth provider
        provider: String,

        /// Device code
        device_code: String,
    },
    /// Refresh OAuth token
    Refresh {
        /// OAuth provider
        provider: String,
    },
}

// -------------------------------------------------------------------------
// Config subcommands
// -------------------------------------------------------------------------

#[derive(clap::Subcommand)]
pub enum ConfigCommand {
    /// Show current configuration
    Show {
        /// Config file path
        #[arg(long, default_value = "ciab.toml")]
        config: String,
    },
    /// Validate configuration file
    Validate {
        /// Config file path
        #[arg(long, default_value = "ciab.toml")]
        config: String,
    },
    /// Initialize a new configuration file
    Init {
        /// Output file path
        #[arg(long, default_value = "ciab.toml")]
        output: String,
    },
}

// -------------------------------------------------------------------------
// Server subcommands
// -------------------------------------------------------------------------

#[derive(clap::Subcommand)]
pub enum ServerCommand {
    /// Start the API server
    Start {
        /// Configuration file path
        #[arg(long, short, default_value = "ciab.toml")]
        config: String,

        /// Database URL
        #[arg(long, env = "CIAB_DATABASE_URL", default_value = "sqlite://ciab.db")]
        database_url: String,
    },
}

// -------------------------------------------------------------------------
// Workspace subcommands
// -------------------------------------------------------------------------

#[derive(clap::Subcommand)]
pub enum WorkspaceCommand {
    /// Create a new workspace
    Create {
        /// Workspace name
        #[arg(long)]
        name: String,

        /// Description
        #[arg(long)]
        description: Option<String>,

        /// Default agent provider
        #[arg(long)]
        provider: Option<String>,

        /// Create from a TOML file
        #[arg(long)]
        from_toml: Option<String>,

        /// Runtime backend (local, opensandbox, docker, kubernetes)
        #[arg(long)]
        runtime_backend: Option<String>,

        /// Kubernetes namespace override
        #[arg(long)]
        k8s_namespace: Option<String>,

        /// Kubernetes RuntimeClass for microvm (e.g. kata-containers)
        #[arg(long)]
        k8s_runtime_class: Option<String>,

        /// Kubernetes container image override
        #[arg(long)]
        k8s_image: Option<String>,
    },
    /// List workspaces
    List {
        /// Filter by name
        #[arg(long)]
        name: Option<String>,
    },
    /// Get workspace details
    Get {
        /// Workspace ID
        id: String,
    },
    /// Update a workspace
    Update {
        /// Workspace ID
        id: String,

        /// New name
        #[arg(long)]
        name: Option<String>,

        /// New description
        #[arg(long)]
        description: Option<String>,
    },
    /// Delete a workspace
    Delete {
        /// Workspace ID
        id: String,
    },
    /// Launch a sandbox from a workspace
    Launch {
        /// Workspace ID
        id: String,
    },
    /// List sandboxes created from a workspace
    Sandboxes {
        /// Workspace ID
        id: String,
    },
    /// Export workspace as TOML
    Export {
        /// Workspace ID
        id: String,

        /// Output file (stdout if omitted)
        #[arg(long, short)]
        output: Option<String>,
    },
    /// Import workspace from TOML file
    Import {
        /// TOML file path
        file: String,
    },
}

// -------------------------------------------------------------------------
// Gateway subcommands
// -------------------------------------------------------------------------

#[derive(clap::Subcommand)]
pub enum GatewayCommand {
    /// Show gateway status
    Status,
    /// Discover CIAB instances on the LAN
    Discover,
    /// Prepare (download/install/validate) a tunnel provider
    Prepare {
        /// Provider name (bore, cloudflare, ngrok, frp)
        provider: String,
    },
    /// Expose a sandbox (creates tunnel + token)
    Expose {
        /// Sandbox ID
        sandbox_id: String,

        /// Token name
        #[arg(long)]
        token_name: Option<String>,

        /// Token expiry in seconds
        #[arg(long)]
        expires: Option<u64>,

        /// Token scope (full, read_only, sandbox, chat_only)
        #[arg(long, default_value = "sandbox")]
        scope: String,
    },
    /// Manage tunnels
    Tunnel {
        #[command(subcommand)]
        command: GatewayTunnelCommand,
    },
    /// Manage client tokens
    Token {
        #[command(subcommand)]
        command: GatewayTokenCommand,
    },
}

#[derive(clap::Subcommand)]
pub enum GatewayTunnelCommand {
    /// Create a tunnel
    Create {
        /// Sandbox ID (optional, omit for gateway tunnel)
        #[arg(long)]
        sandbox_id: Option<String>,

        /// Tunnel type (frp, bore, cloudflare, ngrok, manual, lan)
        #[arg(long, default_value = "bore")]
        tunnel_type: String,

        /// Public URL (required for manual tunnels)
        #[arg(long)]
        public_url: Option<String>,
    },
    /// List tunnels
    List,
    /// Stop a tunnel
    Stop {
        /// Tunnel ID
        id: String,
    },
}

#[derive(clap::Subcommand)]
pub enum GatewayTokenCommand {
    /// Create a client token
    Create {
        /// Token name
        #[arg(long)]
        name: String,

        /// Scope (full, read_only, sandbox:<id>, workspace:<id>, chat:<id>)
        #[arg(long, default_value = "full")]
        scope: String,

        /// Expiry in seconds
        #[arg(long)]
        expires: Option<u64>,
    },
    /// List client tokens
    List,
    /// Revoke a client token
    Revoke {
        /// Token ID
        id: String,
    },
}

// -------------------------------------------------------------------------
// Image subcommands
// -------------------------------------------------------------------------

#[derive(clap::Subcommand)]
pub enum ImageCommand {
    /// Build a machine image via Packer
    Build {
        /// Template source (path, URL, git::url, or builtin://name)
        #[arg(long)]
        template: Option<String>,

        /// Packer variables (key=value, repeatable)
        #[arg(long = "var", short = 'v')]
        var: Vec<String>,

        /// Agent provider to pre-install in the image
        #[arg(long)]
        agent: Option<String>,
    },
    /// List built images
    List,
    /// Check image build status
    Status {
        /// Build ID
        build_id: String,
    },
    /// Delete a built image
    Delete {
        /// Image ID (e.g. AMI ID)
        image_id: String,
    },
}

// -------------------------------------------------------------------------
// Dispatch
// -------------------------------------------------------------------------

pub async fn execute(
    command: Commands,
    client: &CiabClient,
    format: OutputFormat,
) -> anyhow::Result<()> {
    match command {
        Commands::Sandbox { command } => sandbox::execute(command, client, &format).await,
        Commands::Agent { command } => agent::execute(command, client, &format).await,
        Commands::Session { command } => session::execute(command, client, &format).await,
        Commands::Files { command } => files::execute(command, client, &format).await,
        Commands::Credential { command } => credential::execute(command, client, &format).await,
        Commands::Oauth { command } => oauth::execute(command, client, &format).await,
        Commands::Config { command } => config::execute(command).await,
        Commands::Server { command } => server::execute(command).await,
        Commands::Workspace { command } => workspace::execute(command, client, &format).await,
        Commands::Gateway { command } => gateway::execute(command, client, &format).await,
        Commands::Image { command } => image::execute(command, client, &format).await,
    }
}
