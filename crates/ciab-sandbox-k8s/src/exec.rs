use futures::TryStreamExt;
use k8s_openapi::api::core::v1::Pod;
use kube::{
    api::AttachParams,
    Api, Client,
};
use tokio::sync::mpsc;

use ciab_core::types::sandbox::ExecResult;

use crate::error::K8sError;

/// Execute a command in a running Pod and return collected output.
pub async fn exec_command(
    client: &Client,
    namespace: &str,
    pod_name: &str,
    command: &[String],
    workdir: Option<&str>,
) -> Result<ExecResult, K8sError> {
    let api: Api<Pod> = Api::namespaced(client.clone(), namespace);

    // Prepend cd if workdir specified
    let full_cmd: Vec<String> = if let Some(wd) = workdir {
        vec![
            "sh".to_string(),
            "-c".to_string(),
            format!("cd {} && {}", wd, command.join(" ")),
        ]
    } else {
        command.to_vec()
    };

    let cmd_refs: Vec<&str> = full_cmd.iter().map(|s| s.as_str()).collect();

    let params = AttachParams::default().stdout(true).stderr(true).stdin(false);

    let start = std::time::Instant::now();
    let mut attached = api
        .exec(pod_name, cmd_refs, &params)
        .await
        .map_err(K8sError::Kube)?;

    let stdout_stream = attached
        .stdout()
        .ok_or_else(|| K8sError::ExecFailed("no stdout".to_string()))?;
    let stderr_stream = attached
        .stderr()
        .ok_or_else(|| K8sError::ExecFailed("no stderr".to_string()))?;

    let stdout_lines: Vec<String> = tokio_util::io::ReaderStream::new(stdout_stream)
        .map_ok(|b| String::from_utf8_lossy(&b).to_string())
        .try_collect::<Vec<_>>()
        .await
        .unwrap_or_default();

    let stderr_lines: Vec<String> = tokio_util::io::ReaderStream::new(stderr_stream)
        .map_ok(|b| String::from_utf8_lossy(&b).to_string())
        .try_collect::<Vec<_>>()
        .await
        .unwrap_or_default();

    let exit_code = if let Some(status_fut) = attached.take_status() {
        let status = status_fut.await;
        status.and_then(|s| s.code).unwrap_or(0) as i32
    } else {
        0
    };

    Ok(ExecResult {
        exit_code,
        stdout: stdout_lines.concat(),
        stderr: stderr_lines.concat(),
        duration_ms: start.elapsed().as_millis() as u64,
    })
}

/// Execute a command streaming stdout lines into a channel.
pub async fn exec_streaming(
    client: &Client,
    namespace: &str,
    pod_name: &str,
    command: &[String],
    workdir: Option<&str>,
) -> Result<mpsc::Receiver<String>, K8sError> {
    let api: Api<Pod> = Api::namespaced(client.clone(), namespace);

    let full_cmd: Vec<String> = if let Some(wd) = workdir {
        vec![
            "sh".to_string(),
            "-c".to_string(),
            format!("cd {} && {}", wd, command.join(" ")),
        ]
    } else {
        command.to_vec()
    };

    let cmd_refs: Vec<&str> = full_cmd.iter().map(|s| s.as_str()).collect();

    let params = AttachParams::default()
        .stdout(true)
        .stderr(false)
        .stdin(false);

    let mut attached = api
        .exec(pod_name, cmd_refs, &params)
        .await
        .map_err(K8sError::Kube)?;

    let (tx, rx) = mpsc::channel::<String>(256);

    let stdout = attached
        .stdout()
        .ok_or_else(|| K8sError::ExecFailed("no stdout".to_string()))?;

    tokio::spawn(async move {
        use tokio::io::AsyncBufReadExt;
        let mut lines = tokio::io::BufReader::new(stdout).lines();
        loop {
            match lines.next_line().await {
                Ok(Some(line)) => {
                    if tx.send(line).await.is_err() {
                        break;
                    }
                }
                _ => break,
            }
        }
    });

    Ok(rx)
}
