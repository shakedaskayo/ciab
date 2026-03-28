use anyhow::Result;

use crate::client::CiabClient;
use crate::output::{self, OutputFormat};

use super::ImageCommand;

pub async fn execute(
    command: ImageCommand,
    client: &CiabClient,
    format: &OutputFormat,
) -> Result<()> {
    match command {
        ImageCommand::Build {
            template,
            var,
            agent,
        } => {
            let mut body = serde_json::json!({});

            if let Some(tmpl) = template {
                // Detect template source type from the string
                if tmpl.starts_with("builtin://") {
                    body["template"] = serde_json::json!({
                        "type": "builtin",
                        "name": tmpl.strip_prefix("builtin://").unwrap(),
                    });
                } else if tmpl.starts_with("git::") {
                    // git::https://github.com/org/repo.git//path?ref=main
                    let rest = tmpl.strip_prefix("git::").unwrap();
                    let (url_part, ref_part) = if let Some((u, r)) = rest.split_once("?ref=") {
                        (u, Some(r.to_string()))
                    } else {
                        (rest, None)
                    };
                    let (url, path) = if let Some((u, p)) = url_part.split_once("//") {
                        (u.to_string(), p.to_string())
                    } else {
                        (url_part.to_string(), ".".to_string())
                    };
                    body["template"] = serde_json::json!({
                        "type": "git",
                        "url": url,
                        "path": path,
                        "ref": ref_part,
                    });
                } else if tmpl.starts_with("http://") || tmpl.starts_with("https://") {
                    body["template"] = serde_json::json!({
                        "type": "url",
                        "url": tmpl,
                    });
                } else {
                    body["template"] = serde_json::json!({
                        "type": "file_path",
                        "path": tmpl,
                    });
                }
            }

            // Parse --var key=value pairs
            if !var.is_empty() {
                let mut variables = serde_json::Map::new();
                for kv in &var {
                    if let Some((k, v)) = kv.split_once('=') {
                        variables.insert(k.to_string(), serde_json::Value::String(v.to_string()));
                    } else {
                        anyhow::bail!("Invalid --var format '{}', expected key=value", kv);
                    }
                }
                body["variables"] = serde_json::Value::Object(variables);
            }

            if let Some(provider) = agent {
                body["agent_provider"] = serde_json::Value::String(provider);
            }

            let result = client.build_image(&body).await?;
            output::print_value(&result, format);
        }

        ImageCommand::List => {
            let result = client.list_images().await?;
            output::print_value(&result, format);
        }

        ImageCommand::Status { build_id } => {
            let result = client.get_build_status(&build_id).await?;
            output::print_value(&result, format);
        }

        ImageCommand::Delete { image_id } => {
            let result = client.delete_image(&image_id).await?;
            output::print_value(&result, format);
        }
    }

    Ok(())
}
