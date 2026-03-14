use anyhow::Result;

use crate::client::CiabClient;
use crate::output::{self, OutputFormat};

use super::OAuthCommand;

pub async fn execute(
    command: OAuthCommand,
    client: &CiabClient,
    format: &OutputFormat,
) -> Result<()> {
    match command {
        OAuthCommand::Authorize { provider } => {
            let result = client.oauth_authorize(&provider).await?;
            output::print_value(&result, format);
        }

        OAuthCommand::DeviceCode { provider } => {
            let result = client.oauth_device_code(&provider).await?;
            output::print_value(&result, format);
        }

        OAuthCommand::DevicePoll {
            provider,
            device_code,
        } => {
            let result = client.oauth_device_poll(&provider, &device_code).await?;
            output::print_value(&result, format);
        }

        OAuthCommand::Refresh { provider } => {
            let result = client.oauth_refresh(&provider).await?;
            output::print_value(&result, format);
        }
    }

    Ok(())
}
