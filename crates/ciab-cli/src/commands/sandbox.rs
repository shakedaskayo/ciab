use std::collections::HashMap;

use anyhow::Result;

use crate::client::CiabClient;
use crate::output::{self, OutputFormat};

use super::SandboxCommand;

pub async fn execute(
    command: SandboxCommand,
    client: &CiabClient,
    format: &OutputFormat,
) -> Result<()> {
    match command {
        SandboxCommand::Create {
            provider,
            name,
            image,
            cpu,
            memory,
            disk,
            env_vars,
            git_repo,
            credential,
            timeout,
        } => {
            let mut env_map = HashMap::new();
            for kv in &env_vars {
                if let Some((k, v)) = kv.split_once('=') {
                    env_map.insert(k.to_string(), v.to_string());
                }
            }

            let mut resource_limits = serde_json::Value::Null;
            if cpu.is_some() || memory.is_some() || disk.is_some() {
                resource_limits = serde_json::json!({
                    "cpu_cores": cpu.unwrap_or(1.0),
                    "memory_mb": memory.unwrap_or(512),
                    "disk_mb": disk.unwrap_or(1024),
                });
            }

            let mut git_repos = Vec::new();
            if let Some(url) = git_repo {
                git_repos.push(serde_json::json!({"url": url}));
            }

            let mut spec = serde_json::json!({
                "agent_provider": provider,
                "env_vars": env_map,
                "credentials": credential,
                "git_repos": git_repos,
            });

            if let Some(n) = name {
                spec["name"] = serde_json::Value::String(n);
            }
            if let Some(img) = image {
                spec["image"] = serde_json::Value::String(img);
            }
            if !resource_limits.is_null() {
                spec["resource_limits"] = resource_limits;
            }
            if let Some(t) = timeout {
                spec["timeout_secs"] = serde_json::json!(t);
            }

            let result = client.create_sandbox(&spec).await?;
            output::print_value(&result, format);
        }

        SandboxCommand::List { state, provider } => {
            let result = client
                .list_sandboxes(state.as_deref(), provider.as_deref())
                .await?;
            output::print_sandboxes(&result, format);
        }

        SandboxCommand::Get { id } => {
            let result = client.get_sandbox(&id).await?;
            output::print_sandbox(&result, format);
        }

        SandboxCommand::Delete { id } => {
            client.delete_sandbox(&id).await?;
            println!("Sandbox {} deleted", id);
        }

        SandboxCommand::Start { id } => {
            let result = client.start_sandbox(&id).await?;
            output::print_value(&result, format);
        }

        SandboxCommand::Stop { id } => {
            let result = client.stop_sandbox(&id).await?;
            output::print_value(&result, format);
        }

        SandboxCommand::Pause { id } => {
            let result = client.pause_sandbox(&id).await?;
            output::print_value(&result, format);
        }

        SandboxCommand::Resume { id } => {
            let result = client.resume_sandbox(&id).await?;
            output::print_value(&result, format);
        }

        SandboxCommand::Stats { id } => {
            let result = client.sandbox_stats(&id).await?;
            output::print_value(&result, format);
        }

        SandboxCommand::Logs { id, follow, tail } => {
            let result = client.sandbox_logs(&id, follow, tail).await?;
            // For follow mode the server returns SSE; for non-follow it returns JSON.
            if let Some(logs) = result.get("logs").and_then(|v| v.as_array()) {
                for line in logs {
                    if let Some(s) = line.as_str() {
                        println!("{}", s);
                    } else {
                        println!("{}", line);
                    }
                }
            } else {
                output::print_value(&result, format);
            }
        }

        SandboxCommand::Exec {
            id,
            command,
            workdir,
        } => {
            let req = serde_json::json!({
                "command": command,
                "workdir": workdir,
                "env": {},
            });
            let result = client.exec_command(&id, &req).await?;

            // Print stdout/stderr from ExecResult
            if let Some(stdout) = result.get("stdout").and_then(|v| v.as_str()) {
                if !stdout.is_empty() {
                    print!("{}", stdout);
                }
            }
            if let Some(stderr) = result.get("stderr").and_then(|v| v.as_str()) {
                if !stderr.is_empty() {
                    eprint!("{}", stderr);
                }
            }
            if let Some(exit_code) = result.get("exit_code").and_then(|v| v.as_i64()) {
                if exit_code != 0 {
                    std::process::exit(exit_code as i32);
                }
            }
        }
    }

    Ok(())
}
