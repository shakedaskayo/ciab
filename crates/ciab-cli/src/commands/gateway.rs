use anyhow::Result;

use crate::client::CiabClient;
use crate::output::{self, OutputFormat};

use super::{GatewayCommand, GatewayTokenCommand, GatewayTunnelCommand};

pub async fn execute(
    command: GatewayCommand,
    client: &CiabClient,
    format: &OutputFormat,
) -> Result<()> {
    match command {
        GatewayCommand::Status => {
            let result = client.gateway_status().await?;
            output::print_value(&result, format);
        }
        GatewayCommand::Discover => {
            let result = client.gateway_discover().await?;
            output::print_value(&result, format);
        }
        GatewayCommand::Prepare { provider } => {
            println!("Preparing tunnel provider: {}...", provider);
            let body = serde_json::json!({ "provider": provider });
            let result = client.gateway_prepare_provider(&body).await?;

            if let Some(msg) = result.get("message").and_then(|m| m.as_str()) {
                println!("{}", msg);
            }
            if let Some(path) = result.get("binary_path").and_then(|p| p.as_str()) {
                println!("Binary: {}", path);
            }
            if let Some(version) = result.get("version").and_then(|v| v.as_str()) {
                println!("Version: {}", version);
            }
            output::print_value(&result, format);
        }
        GatewayCommand::Expose {
            sandbox_id,
            token_name,
            expires,
            scope,
        } => {
            let body = serde_json::json!({
                "sandbox_id": sandbox_id,
                "token_name": token_name,
                "expires_secs": expires,
                "scope": parse_scope_arg(&scope, Some(&sandbox_id)),
            });
            let result = client.gateway_expose(&body).await?;

            // Show the raw token prominently since it's only shown once.
            if let Some(token) = result.get("token").and_then(|t| t.as_str()) {
                println!("Token (save this — it won't be shown again):");
                println!("  {}", token);
                println!();
            }
            if let Some(url) = result
                .get("tunnel")
                .and_then(|t| t.get("public_url"))
                .and_then(|u| u.as_str())
            {
                println!("Public URL: {}", url);
            }
            output::print_value(&result, format);
        }
        GatewayCommand::Tunnel { command } => match command {
            GatewayTunnelCommand::Create {
                sandbox_id,
                tunnel_type,
                public_url,
            } => {
                let body = serde_json::json!({
                    "sandbox_id": sandbox_id,
                    "tunnel_type": tunnel_type,
                    "local_port": 9090,
                    "public_url": public_url,
                });
                let result = client.gateway_create_tunnel(&body).await?;
                output::print_value(&result, format);
            }
            GatewayTunnelCommand::List => {
                let result = client.gateway_list_tunnels().await?;
                output::print_value(&result, format);
            }
            GatewayTunnelCommand::Stop { id } => {
                let result = client.gateway_delete_tunnel(&id).await?;
                output::print_value(&result, format);
            }
        },
        GatewayCommand::Token { command } => match command {
            GatewayTokenCommand::Create {
                name,
                scope,
                expires,
            } => {
                let body = serde_json::json!({
                    "name": name,
                    "scopes": [parse_scope_arg(&scope, None)],
                    "expires_secs": expires,
                });
                let result = client.gateway_create_token(&body).await?;

                // Show the raw token prominently.
                if let Some(token) = result.get("token").and_then(|t| t.as_str()) {
                    println!("Token (save this — it won't be shown again):");
                    println!("  {}", token);
                    println!();
                }
                output::print_value(&result, format);
            }
            GatewayTokenCommand::List => {
                let result = client.gateway_list_tokens().await?;
                output::print_value(&result, format);
            }
            GatewayTokenCommand::Revoke { id } => {
                let result = client.gateway_revoke_token(&id).await?;
                output::print_value(&result, format);
            }
        },
    }
    Ok(())
}

/// Parse a CLI scope argument into a JSON scope value.
fn parse_scope_arg(scope: &str, sandbox_id: Option<&str>) -> serde_json::Value {
    if scope == "full" {
        serde_json::json!({"type": "full_access"})
    } else if scope == "read_only" {
        serde_json::json!({"type": "read_only"})
    } else if scope == "sandbox" {
        if let Some(sid) = sandbox_id {
            serde_json::json!({"type": "sandbox_access", "sandbox_id": sid})
        } else {
            serde_json::json!({"type": "full_access"})
        }
    } else if let Some(sid) = scope.strip_prefix("sandbox:") {
        serde_json::json!({"type": "sandbox_access", "sandbox_id": sid})
    } else if let Some(wid) = scope.strip_prefix("workspace:") {
        serde_json::json!({"type": "workspace_access", "workspace_id": wid})
    } else if scope == "chat_only" || scope == "chat" {
        if let Some(sid) = sandbox_id {
            serde_json::json!({"type": "chat_only", "sandbox_id": sid})
        } else {
            serde_json::json!({"type": "full_access"})
        }
    } else if let Some(sid) = scope.strip_prefix("chat:") {
        serde_json::json!({"type": "chat_only", "sandbox_id": sid})
    } else {
        serde_json::json!({"type": "full_access"})
    }
}
