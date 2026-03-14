use clap::Parser;

mod cli;
mod client;
mod commands;
mod output;

#[derive(Parser)]
#[command(
    name = "ciab",
    version,
    about = "Claude-In-A-Box: Manage coding agent sandboxes"
)]
struct Cli {
    #[command(subcommand)]
    command: commands::Commands,

    /// API server URL
    #[arg(long, env = "CIAB_SERVER_URL", default_value = "http://localhost:9090")]
    server_url: String,

    /// API key
    #[arg(long, env = "CIAB_API_KEY")]
    api_key: Option<String>,

    /// Output format
    #[arg(long, default_value = "table")]
    output: output::OutputFormat,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    let client = client::CiabClient::new(&cli.server_url, cli.api_key.as_deref());

    commands::execute(cli.command, &client, cli.output).await
}
