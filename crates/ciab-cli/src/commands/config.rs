use anyhow::{Context, Result};

use ciab_core::types::config::AppConfig;

use super::ConfigCommand;

pub async fn execute(command: ConfigCommand) -> Result<()> {
    match command {
        ConfigCommand::Show { config } => {
            let content = tokio::fs::read_to_string(&config)
                .await
                .with_context(|| format!("reading config file: {}", config))?;
            // Validate it parses.
            let parsed: AppConfig =
                toml::from_str(&content).with_context(|| "parsing config TOML")?;
            println!("{:#?}", parsed);
            Ok(())
        }

        ConfigCommand::Validate { config } => {
            let content = tokio::fs::read_to_string(&config)
                .await
                .with_context(|| format!("reading config file: {}", config))?;
            let _parsed: AppConfig =
                toml::from_str(&content).with_context(|| "parsing config TOML")?;
            println!("Configuration is valid.");
            Ok(())
        }

        ConfigCommand::Init { output } => {
            let template = r#"[server]
host = "0.0.0.0"
port = 9090

# Runtime backend: "local" (default), "docker", or "opensandbox"
[runtime]
backend = "local"
# local_workdir = "/tmp/ciab-sandboxes"
# local_max_processes = 10

# For container-based runtimes:
# backend = "opensandbox"
# opensandbox_url = "http://localhost:9090"

[agents]
default_provider = "claude-code"

[agents.providers.claude-code]
enabled = true
binary = "claude"
# image = "ghcr.io/ciab/claude-sandbox:latest"  # for docker/opensandbox
api_key_env = "ANTHROPIC_API_KEY"

[credentials]
backend = "sqlite"
encryption_key_env = "CIAB_ENCRYPTION_KEY"

[provisioning]
timeout_secs = 300

[streaming]
buffer_size = 500
keepalive_interval_secs = 15

[security]

[logging]
level = "info"
format = "json"
"#;
            tokio::fs::write(&output, template)
                .await
                .with_context(|| format!("writing config file: {}", output))?;
            println!("Configuration written to {}", output);
            Ok(())
        }
    }
}
