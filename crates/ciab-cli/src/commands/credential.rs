use anyhow::Result;

use crate::client::CiabClient;
use crate::output::{self, OutputFormat};

use super::CredentialCommand;

pub async fn execute(
    command: CredentialCommand,
    client: &CiabClient,
    format: &OutputFormat,
) -> Result<()> {
    match command {
        CredentialCommand::Create {
            name,
            cred_type,
            value,
        } => {
            let result = client.create_credential(&name, &cred_type, &value).await?;
            output::print_value(&result, format);
        }

        CredentialCommand::List => {
            let result = client.list_credentials().await?;
            output::print_credentials(&result, format);
        }

        CredentialCommand::Get { id } => {
            let result = client.get_credential(&id).await?;
            output::print_value(&result, format);
        }

        CredentialCommand::Delete { id } => {
            client.delete_credential(&id).await?;
            println!("Credential {} deleted", id);
        }
    }

    Ok(())
}
