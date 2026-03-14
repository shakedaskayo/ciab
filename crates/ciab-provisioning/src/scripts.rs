use std::collections::HashMap;

use chrono::Utc;
use ciab_core::error::{CiabError, CiabResult};
use ciab_core::traits::runtime::SandboxRuntime;
use ciab_core::types::sandbox::ExecRequest;
use ciab_core::types::stream::{StreamEvent, StreamEventType};
use tokio::sync::mpsc;
use uuid::Uuid;

/// Upload and execute a provisioning script inside a sandbox.
pub async fn run_script(
    runtime: &dyn SandboxRuntime,
    sandbox_id: &Uuid,
    script_content: &str,
    tx: mpsc::Sender<StreamEvent>,
    sandbox_uuid: Uuid,
) -> CiabResult<()> {
    let script_id = Uuid::new_v4();
    let script_path = format!("/tmp/provision_{}.sh", script_id);

    // Upload script to sandbox
    runtime
        .write_file(sandbox_id, &script_path, script_content.as_bytes())
        .await?;

    // Make script executable
    let chmod_request = ExecRequest {
        command: vec!["chmod".to_string(), "+x".to_string(), script_path.clone()],
        workdir: None,
        env: HashMap::new(),
        stdin: None,
        timeout_secs: Some(10),
        tty: false,
    };
    runtime.exec(sandbox_id, &chmod_request).await?;

    // Execute the script
    let exec_request = ExecRequest {
        command: vec!["/bin/sh".to_string(), script_path.clone()],
        workdir: Some("/workspace".to_string()),
        env: HashMap::new(),
        stdin: None,
        timeout_secs: Some(600),
        tty: false,
    };

    let result = runtime.exec(sandbox_id, &exec_request).await?;

    // Forward stdout as log events
    if !result.stdout.is_empty() {
        for line in result.stdout.lines() {
            let event = StreamEvent {
                id: format!("script-{}", Uuid::new_v4()),
                sandbox_id: sandbox_uuid,
                session_id: None,
                event_type: StreamEventType::LogLine,
                data: serde_json::json!({ "line": line, "stream": "stdout" }),
                timestamp: Utc::now(),
            };
            let _ = tx.send(event).await;
        }
    }

    // Forward stderr as log events
    if !result.stderr.is_empty() {
        for line in result.stderr.lines() {
            let event = StreamEvent {
                id: format!("script-{}", Uuid::new_v4()),
                sandbox_id: sandbox_uuid,
                session_id: None,
                event_type: StreamEventType::LogLine,
                data: serde_json::json!({ "line": line, "stream": "stderr" }),
                timestamp: Utc::now(),
            };
            let _ = tx.send(event).await;
        }
    }

    // Clean up script file (best effort)
    let cleanup_request = ExecRequest {
        command: vec!["rm".to_string(), "-f".to_string(), script_path],
        workdir: None,
        env: HashMap::new(),
        stdin: None,
        timeout_secs: Some(10),
        tty: false,
    };
    let _ = runtime.exec(sandbox_id, &cleanup_request).await;

    if result.exit_code != 0 {
        return Err(CiabError::ScriptExecutionFailed(format!(
            "provisioning script failed (exit code {}): {}",
            result.exit_code, result.stderr
        )));
    }

    Ok(())
}
