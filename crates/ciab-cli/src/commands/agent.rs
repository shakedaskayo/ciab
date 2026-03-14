use anyhow::{Context, Result};
use futures::StreamExt;

use crate::client::CiabClient;
use crate::output::{self, OutputFormat};

use super::AgentCommand;

pub async fn execute(
    command: AgentCommand,
    client: &CiabClient,
    format: &OutputFormat,
) -> Result<()> {
    match command {
        AgentCommand::Chat {
            sandbox_id,
            session_id,
            message,
            interactive,
            stream,
        } => {
            // Resolve or create a session.
            let sid = if let Some(id) = session_id {
                id
            } else {
                let session = client.create_session(&sandbox_id).await?;
                session
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string()
            };

            if interactive {
                // Interactive REPL loop.
                println!("Interactive session {} (Ctrl+D to exit)", sid);
                use tokio::io::AsyncBufReadExt;
                let stdin = tokio::io::stdin();
                let reader = tokio::io::BufReader::new(stdin);
                let mut lines = reader.lines();

                loop {
                    eprint!("> ");
                    let line: String = match lines.next_line().await? {
                        Some(line) => line,
                        None => break, // EOF
                    };

                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }

                    if stream {
                        // Send message then attach to SSE stream.
                        let _resp = client.send_message(&sid, trimmed).await?;
                        stream_session_events(client, &sid).await?;
                    } else {
                        let resp = client.send_message(&sid, trimmed).await?;
                        print_assistant_response(&resp);
                    }
                }
            } else if let Some(msg) = message {
                if stream {
                    let _resp = client.send_message(&sid, &msg).await?;
                    stream_session_events(client, &sid).await?;
                } else {
                    let resp = client.send_message(&sid, &msg).await?;
                    output::print_value(&resp, format);
                }
            } else {
                anyhow::bail!("Either --message or --interactive must be specified for agent chat");
            }
        }

        AgentCommand::Attach { session_id } => {
            stream_session_events(client, &session_id).await?;
        }

        AgentCommand::Interrupt { session_id } => {
            let result = client.interrupt_session(&session_id).await?;
            output::print_value(&result, format);
        }

        AgentCommand::Providers => {
            let providers = vec![
                serde_json::json!({"name": "claude-code", "description": "Claude Code (Anthropic)"}),
                serde_json::json!({"name": "codex", "description": "Codex (OpenAI)"}),
                serde_json::json!({"name": "gemini", "description": "Gemini (Google)"}),
                serde_json::json!({"name": "cursor", "description": "Cursor"}),
            ];
            let val = serde_json::Value::Array(providers);
            output::print_value(&val, format);
        }
    }

    Ok(())
}

fn print_assistant_response(resp: &serde_json::Value) {
    // Extract text from assistant message content.
    if let Some(content) = resp.get("content").and_then(|v| v.as_array()) {
        for part in content {
            if let Some(text) = part.get("text").and_then(|v| v.as_str()) {
                println!("{}", text);
            }
        }
    } else if let Some(text) = resp.get("text").and_then(|v| v.as_str()) {
        println!("{}", text);
    } else {
        output::print_json(resp);
    }
}

async fn stream_session_events(client: &CiabClient, session_id: &str) -> Result<()> {
    let resp = client.stream_session(session_id).await?;
    let mut stream = resp.bytes_stream();
    let mut buffer = String::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("reading SSE chunk")?;
        let text = String::from_utf8_lossy(&chunk);
        buffer.push_str(&text);

        // Process complete SSE events (double newline delimited).
        while let Some(pos) = buffer.find("\n\n") {
            let event_text = buffer[..pos].to_string();
            buffer = buffer[pos + 2..].to_string();

            // Parse SSE event.
            let mut data_lines = Vec::new();
            for line in event_text.lines() {
                if let Some(data) = line.strip_prefix("data:") {
                    data_lines.push(data.trim().to_string());
                }
            }

            if data_lines.is_empty() {
                continue;
            }

            let data = data_lines.join("\n");
            if let Ok(event) = serde_json::from_str::<serde_json::Value>(&data) {
                // Print text deltas inline.
                if let Some(evt_data) = event.get("data") {
                    if let Some(text) = evt_data.get("text").and_then(|v| v.as_str()) {
                        print!("{}", text);
                    }
                }
            }
        }
    }

    println!();
    Ok(())
}
