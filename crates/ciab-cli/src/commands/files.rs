use anyhow::{Context, Result};
use std::path::Path;

use crate::client::CiabClient;
use crate::output::{self, OutputFormat};

use super::FilesCommand;

pub async fn execute(
    command: FilesCommand,
    client: &CiabClient,
    format: &OutputFormat,
) -> Result<()> {
    match command {
        FilesCommand::List { sandbox_id, path } => {
            let result = client.list_files(&sandbox_id, Some(&path)).await?;
            output::print_value(&result, format);
        }

        FilesCommand::Upload {
            sandbox_id,
            local_path,
            remote_path,
        } => {
            let content = tokio::fs::read(&local_path)
                .await
                .with_context(|| format!("reading local file: {}", local_path))?;
            client
                .upload_file(&sandbox_id, &remote_path, content)
                .await?;
            println!("Uploaded {} -> sandbox:{}", local_path, remote_path);
        }

        FilesCommand::Download {
            sandbox_id,
            remote_path,
            local_path,
        } => {
            let content = client.download_file(&sandbox_id, &remote_path).await?;
            // Ensure parent directory exists.
            if let Some(parent) = Path::new(&local_path).parent() {
                tokio::fs::create_dir_all(parent)
                    .await
                    .with_context(|| format!("creating directory: {}", parent.display()))?;
            }
            tokio::fs::write(&local_path, &content)
                .await
                .with_context(|| format!("writing local file: {}", local_path))?;
            println!("Downloaded sandbox:{} -> {}", remote_path, local_path);
        }

        FilesCommand::Delete {
            sandbox_id,
            remote_path,
        } => {
            client.delete_file(&sandbox_id, &remote_path).await?;
            println!("Deleted sandbox:{}", remote_path);
        }
    }

    Ok(())
}
