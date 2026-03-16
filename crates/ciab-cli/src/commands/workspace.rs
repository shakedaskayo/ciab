use anyhow::Result;

use crate::client::CiabClient;
use crate::output::{self, OutputFormat};

use super::WorkspaceCommand;

pub async fn execute(
    command: WorkspaceCommand,
    client: &CiabClient,
    format: &OutputFormat,
) -> Result<()> {
    match command {
        WorkspaceCommand::Create {
            name,
            description,
            provider,
            from_toml,
            runtime_backend,
            k8s_namespace,
            k8s_runtime_class,
            k8s_image,
        } => {
            if let Some(toml_path) = from_toml {
                let content = std::fs::read_to_string(&toml_path)?;
                let result = client.import_workspace_toml(&content).await?;
                output::print_value(&result, format);
            } else {
                let mut spec = serde_json::json!({});
                if let Some(p) = provider {
                    spec["agent"] = serde_json::json!({"provider": p});
                }

                // Build runtime config if any K8s flags or backend specified
                if runtime_backend.is_some()
                    || k8s_namespace.is_some()
                    || k8s_runtime_class.is_some()
                    || k8s_image.is_some()
                {
                    let mut rt = serde_json::json!({});
                    if let Some(ref b) = runtime_backend {
                        rt["backend"] = serde_json::Value::String(b.clone());
                    }
                    if k8s_namespace.is_some() || k8s_runtime_class.is_some() || k8s_image.is_some()
                    {
                        let mut k8s = serde_json::json!({});
                        if let Some(ns) = k8s_namespace {
                            k8s["kubernetes_namespace"] = serde_json::Value::String(ns);
                        }
                        if let Some(rc) = k8s_runtime_class {
                            k8s["kubernetes_runtime_class"] = serde_json::Value::String(rc);
                        }
                        if let Some(img) = k8s_image {
                            k8s["kubernetes_image"] = serde_json::Value::String(img);
                        }
                        rt.as_object_mut()
                            .unwrap()
                            .extend(k8s.as_object().unwrap().clone());
                    }
                    spec["runtime"] = rt;
                }

                let body = serde_json::json!({
                    "name": name,
                    "description": description,
                    "spec": spec,
                });
                let result = client.create_workspace(&body).await?;
                output::print_value(&result, format);
            }
        }

        WorkspaceCommand::List { name } => {
            let result = client.list_workspaces(name.as_deref()).await?;
            output::print_workspaces(&result, format);
        }

        WorkspaceCommand::Get { id } => {
            let result = client.get_workspace(&id).await?;
            output::print_workspace(&result, format);
        }

        WorkspaceCommand::Update {
            id,
            name,
            description,
        } => {
            let mut body = serde_json::json!({});
            if let Some(n) = name {
                body["name"] = serde_json::Value::String(n);
            }
            if let Some(d) = description {
                body["description"] = serde_json::Value::String(d);
            }
            let result = client.update_workspace(&id, &body).await?;
            output::print_value(&result, format);
        }

        WorkspaceCommand::Delete { id } => {
            client.delete_workspace(&id).await?;
            println!("Workspace {} deleted", id);
        }

        WorkspaceCommand::Launch { id } => {
            let result = client.launch_workspace(&id).await?;
            output::print_value(&result, format);
        }

        WorkspaceCommand::Sandboxes { id } => {
            let result = client.list_workspace_sandboxes(&id).await?;
            output::print_value(&result, format);
        }

        WorkspaceCommand::Export { id, output: path } => {
            let toml_content = client.export_workspace_toml(&id).await?;
            if let Some(p) = path {
                std::fs::write(&p, &toml_content)?;
                println!("Exported to {}", p);
            } else {
                println!("{}", toml_content);
            }
        }

        WorkspaceCommand::Import { file } => {
            let content = std::fs::read_to_string(&file)?;
            let result = client.import_workspace_toml(&content).await?;
            output::print_value(&result, format);
        }
    }

    Ok(())
}
