use anyhow::Result;

use crate::client::CiabClient;
use crate::output::{self, OutputFormat};

use super::SessionCommand;

pub async fn execute(
    command: SessionCommand,
    client: &CiabClient,
    format: &OutputFormat,
) -> Result<()> {
    match command {
        SessionCommand::Create { sandbox_id } => {
            let result = client.create_session(&sandbox_id).await?;
            output::print_value(&result, format);
        }

        SessionCommand::List { sandbox_id } => {
            let result = client.list_sessions(&sandbox_id).await?;
            output::print_sessions(&result, format);
        }

        SessionCommand::Get { id } => {
            let result = client.get_session(&id).await?;
            output::print_value(&result, format);
        }

        SessionCommand::Send {
            session_id,
            message,
        } => {
            let result = client.send_message(&session_id, &message).await?;
            output::print_value(&result, format);
        }

        SessionCommand::Interrupt { id } => {
            let result = client.interrupt_session(&id).await?;
            output::print_value(&result, format);
        }
    }

    Ok(())
}
