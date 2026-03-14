use std::collections::HashMap;

use async_trait::async_trait;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::error::CiabResult;
use crate::types::sandbox::{
    ExecRequest, ExecResult, FileInfo, LogOptions, ResourceStats, SandboxInfo, SandboxSpec,
    SandboxState,
};

#[async_trait]
pub trait SandboxRuntime: Send + Sync {
    /// Create a new sandbox from the given spec.
    async fn create_sandbox(&self, spec: &SandboxSpec) -> CiabResult<SandboxInfo>;

    /// Get info about a sandbox by ID.
    async fn get_sandbox(&self, id: &Uuid) -> CiabResult<SandboxInfo>;

    /// List all sandboxes, optionally filtered by state, provider, and labels.
    async fn list_sandboxes(
        &self,
        state: Option<SandboxState>,
        provider: Option<&str>,
        labels: &HashMap<String, String>,
    ) -> CiabResult<Vec<SandboxInfo>>;

    /// Start a sandbox.
    async fn start_sandbox(&self, id: &Uuid) -> CiabResult<()>;

    /// Stop a sandbox.
    async fn stop_sandbox(&self, id: &Uuid) -> CiabResult<()>;

    /// Pause a sandbox.
    async fn pause_sandbox(&self, id: &Uuid) -> CiabResult<()>;

    /// Resume a paused sandbox.
    async fn resume_sandbox(&self, id: &Uuid) -> CiabResult<()>;

    /// Terminate and remove a sandbox.
    async fn terminate_sandbox(&self, id: &Uuid) -> CiabResult<()>;

    /// Execute a command inside a sandbox.
    async fn exec(&self, id: &Uuid, request: &ExecRequest) -> CiabResult<ExecResult>;

    /// Execute a command inside a sandbox, streaming stdout lines as they arrive.
    /// Returns a receiver of output lines and a join handle for the final ExecResult.
    ///
    /// Default implementation falls back to `exec()` and sends all lines at once.
    async fn exec_streaming(
        &self,
        id: &Uuid,
        request: &ExecRequest,
    ) -> CiabResult<(
        mpsc::Receiver<String>,
        tokio::task::JoinHandle<CiabResult<ExecResult>>,
    )> {
        let result = self.exec(id, request).await?;
        let (tx, rx) = mpsc::channel::<String>(256);
        let stdout = result.stdout.clone();
        let handle = tokio::spawn(async move {
            for line in stdout.lines() {
                let _ = tx.send(line.to_string()).await;
            }
            Ok(result)
        });
        Ok((rx, handle))
    }

    /// Execute a command with bidirectional streaming.
    /// Returns (stdout lines receiver, stdin sender, join handle).
    /// Send lines to the stdin sender to write to the process's stdin.
    async fn exec_streaming_interactive(
        &self,
        id: &Uuid,
        request: &ExecRequest,
    ) -> CiabResult<(
        mpsc::Receiver<String>,
        mpsc::Sender<String>,
        tokio::task::JoinHandle<CiabResult<ExecResult>>,
    )> {
        // Default: fall back to non-interactive exec_streaming, return a dummy stdin sender.
        let (rx, handle) = self.exec_streaming(id, request).await?;
        let (stdin_tx, _stdin_rx) = mpsc::channel::<String>(16);
        Ok((rx, stdin_tx, handle))
    }

    /// Read a file from a sandbox.
    async fn read_file(&self, id: &Uuid, path: &str) -> CiabResult<Vec<u8>>;

    /// Write a file to a sandbox.
    async fn write_file(&self, id: &Uuid, path: &str, content: &[u8]) -> CiabResult<()>;

    /// List files in a directory inside a sandbox.
    async fn list_files(&self, id: &Uuid, path: &str) -> CiabResult<Vec<FileInfo>>;

    /// Get resource stats for a sandbox.
    async fn get_stats(&self, id: &Uuid) -> CiabResult<ResourceStats>;

    /// Stream logs from a sandbox.
    async fn stream_logs(
        &self,
        id: &Uuid,
        options: &LogOptions,
    ) -> CiabResult<mpsc::Receiver<String>>;

    /// Kill an active exec process for a sandbox.
    async fn kill_exec(&self, id: &Uuid) -> CiabResult<()> {
        let _ = id;
        Ok(())
    }
}
