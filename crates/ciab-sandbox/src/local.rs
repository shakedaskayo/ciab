use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use async_trait::async_trait;
use dashmap::DashMap;
use tokio::process::Command;
use tokio::sync::mpsc;
use uuid::Uuid;

use ciab_core::error::{CiabError, CiabResult};
use ciab_core::traits::runtime::SandboxRuntime;
use ciab_core::types::sandbox::{
    ExecRequest, ExecResult, FileInfo, LogOptions, ResourceStats, SandboxInfo, SandboxSpec,
    SandboxState,
};

/// Tracks a local sandbox (agent running as a child process).
struct LocalSandbox {
    id: Uuid,
    workdir: PathBuf,
    state: SandboxState,
    spec: SandboxSpec,
    created_at: chrono::DateTime<chrono::Utc>,
}

/// Runtime that runs agents as local processes — no Docker or containers needed.
pub struct LocalProcessRuntime {
    base_workdir: PathBuf,
    sandboxes: DashMap<Uuid, LocalSandbox>,
    max_processes: u32,
    process_count: AtomicU64,
    active_processes: DashMap<Uuid, tokio::sync::watch::Sender<bool>>,
}

impl LocalProcessRuntime {
    pub fn new(base_workdir: Option<String>, max_processes: Option<u32>) -> Self {
        let base_workdir = base_workdir.map(PathBuf::from).unwrap_or_else(|| {
            let tmp = std::env::temp_dir().join("ciab-sandboxes");
            let _ = std::fs::create_dir_all(&tmp);
            tmp
        });

        Self {
            base_workdir,
            sandboxes: DashMap::new(),
            max_processes: max_processes.unwrap_or(10),
            process_count: AtomicU64::new(0),
            active_processes: DashMap::new(),
        }
    }

    fn sandbox_dir(&self, id: &Uuid) -> PathBuf {
        self.base_workdir.join(id.to_string())
    }

    fn get_sandbox_ref(
        &self,
        id: &Uuid,
    ) -> CiabResult<dashmap::mapref::one::Ref<'_, Uuid, LocalSandbox>> {
        self.sandboxes
            .get(id)
            .ok_or_else(|| CiabError::SandboxNotFound(id.to_string()))
    }

    fn to_info(sb: &LocalSandbox) -> SandboxInfo {
        SandboxInfo {
            id: sb.id,
            name: sb.spec.name.clone(),
            state: sb.state.clone(),
            persistence: sb.spec.persistence.clone(),
            agent_provider: sb.spec.agent_provider.clone(),
            endpoint_url: None,
            resource_stats: None,
            labels: sb.spec.labels.clone(),
            created_at: sb.created_at,
            updated_at: chrono::Utc::now(),
            spec: sb.spec.clone(),
        }
    }
}

#[async_trait]
impl SandboxRuntime for LocalProcessRuntime {
    async fn create_sandbox(&self, spec: &SandboxSpec) -> CiabResult<SandboxInfo> {
        let count = self.process_count.load(Ordering::Relaxed);
        if count >= self.max_processes as u64 {
            return Err(CiabError::SandboxCreationFailed(format!(
                "max local process limit reached ({})",
                self.max_processes
            )));
        }

        let id = Uuid::new_v4();
        let workdir = self.sandbox_dir(&id);
        tokio::fs::create_dir_all(&workdir)
            .await
            .map_err(|e| CiabError::SandboxCreationFailed(e.to_string()))?;

        let now = chrono::Utc::now();
        let sandbox = LocalSandbox {
            id,
            workdir,
            state: SandboxState::Running,
            spec: spec.clone(),
            created_at: now,
        };

        let info = Self::to_info(&sandbox);
        self.sandboxes.insert(id, sandbox);
        self.process_count.fetch_add(1, Ordering::Relaxed);

        Ok(info)
    }

    async fn get_sandbox(&self, id: &Uuid) -> CiabResult<SandboxInfo> {
        let sb = self.get_sandbox_ref(id)?;
        Ok(Self::to_info(&sb))
    }

    async fn list_sandboxes(
        &self,
        state: Option<SandboxState>,
        provider: Option<&str>,
        labels: &HashMap<String, String>,
    ) -> CiabResult<Vec<SandboxInfo>> {
        let mut results: Vec<SandboxInfo> = self
            .sandboxes
            .iter()
            .map(|entry| Self::to_info(entry.value()))
            .collect();

        if let Some(ref filter_state) = state {
            results.retain(|s| &s.state == filter_state);
        }
        if let Some(filter_provider) = provider {
            results.retain(|s| s.agent_provider == filter_provider);
        }
        if !labels.is_empty() {
            results.retain(|s| {
                labels
                    .iter()
                    .all(|(k, v)| s.labels.get(k).map(|sv| sv == v).unwrap_or(false))
            });
        }

        Ok(results)
    }

    async fn start_sandbox(&self, id: &Uuid) -> CiabResult<()> {
        let mut sb = self
            .sandboxes
            .get_mut(id)
            .ok_or_else(|| CiabError::SandboxNotFound(id.to_string()))?;
        sb.state = SandboxState::Running;
        Ok(())
    }

    async fn stop_sandbox(&self, id: &Uuid) -> CiabResult<()> {
        let mut sb = self
            .sandboxes
            .get_mut(id)
            .ok_or_else(|| CiabError::SandboxNotFound(id.to_string()))?;
        sb.state = SandboxState::Stopped;
        Ok(())
    }

    async fn pause_sandbox(&self, id: &Uuid) -> CiabResult<()> {
        let mut sb = self
            .sandboxes
            .get_mut(id)
            .ok_or_else(|| CiabError::SandboxNotFound(id.to_string()))?;
        sb.state = SandboxState::Paused;
        Ok(())
    }

    async fn resume_sandbox(&self, id: &Uuid) -> CiabResult<()> {
        let mut sb = self
            .sandboxes
            .get_mut(id)
            .ok_or_else(|| CiabError::SandboxNotFound(id.to_string()))?;
        sb.state = SandboxState::Running;
        Ok(())
    }

    async fn terminate_sandbox(&self, id: &Uuid) -> CiabResult<()> {
        if let Some((_, _sb)) = self.sandboxes.remove(id) {
            self.process_count.fetch_sub(1, Ordering::Relaxed);
            // Clean up the sandbox directory
            let workdir = self.sandbox_dir(id);
            if workdir.exists() {
                let _ = tokio::fs::remove_dir_all(&workdir).await;
            }
        }
        Ok(())
    }

    async fn exec(&self, id: &Uuid, request: &ExecRequest) -> CiabResult<ExecResult> {
        let sb = self.get_sandbox_ref(id)?;
        if sb.state != SandboxState::Running {
            return Err(CiabError::SandboxInvalidState {
                current: sb.state.to_string(),
                expected: "running".to_string(),
            });
        }

        let workdir = request
            .workdir
            .as_ref()
            .map(PathBuf::from)
            .filter(|p| p.exists())
            .unwrap_or_else(|| sb.workdir.clone());

        if request.command.is_empty() {
            return Err(CiabError::ExecFailed("empty command".to_string()));
        }

        let program = &request.command[0];
        let args = &request.command[1..];

        let start = std::time::Instant::now();
        let mut cmd = Command::new(program);
        cmd.args(args)
            .current_dir(&workdir)
            .envs(&request.env)
            .envs(&sb.spec.env_vars)
            .env_remove("CLAUDECODE");

        if let Some(ref stdin_data) = request.stdin {
            let mut child = cmd
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .map_err(|e| CiabError::ExecFailed(e.to_string()))?;

            if let Some(ref mut child_stdin) = child.stdin {
                use tokio::io::AsyncWriteExt;
                let _ = child_stdin.write_all(stdin_data.as_bytes()).await;
                let _ = child_stdin.shutdown().await;
            }
            // Drop stdin to signal EOF
            child.stdin.take();

            let output = child
                .wait_with_output()
                .await
                .map_err(|e| CiabError::ExecFailed(e.to_string()))?;

            let duration = start.elapsed();
            return Ok(ExecResult {
                exit_code: output.status.code().unwrap_or(-1),
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                duration_ms: duration.as_millis() as u64,
            });
        }

        cmd.stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        let output = if let Some(timeout_secs) = request.timeout_secs {
            tokio::time::timeout(
                std::time::Duration::from_secs(timeout_secs as u64),
                cmd.output(),
            )
            .await
            .map_err(|_| CiabError::Timeout("exec command timed out".to_string()))?
            .map_err(|e| CiabError::ExecFailed(e.to_string()))?
        } else {
            cmd.output()
                .await
                .map_err(|e| CiabError::ExecFailed(e.to_string()))?
        };

        let duration = start.elapsed();
        Ok(ExecResult {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            duration_ms: duration.as_millis() as u64,
        })
    }

    async fn exec_streaming(
        &self,
        id: &Uuid,
        request: &ExecRequest,
    ) -> CiabResult<(
        mpsc::Receiver<String>,
        tokio::task::JoinHandle<CiabResult<ExecResult>>,
    )> {
        let sb = self.get_sandbox_ref(id)?;
        if sb.state != SandboxState::Running {
            return Err(CiabError::SandboxInvalidState {
                current: sb.state.to_string(),
                expected: "running".to_string(),
            });
        }

        let workdir = request
            .workdir
            .as_ref()
            .map(PathBuf::from)
            .filter(|p| p.exists())
            .unwrap_or_else(|| sb.workdir.clone());

        if request.command.is_empty() {
            return Err(CiabError::ExecFailed("empty command".to_string()));
        }

        let program = request.command[0].clone();
        let args: Vec<String> = request.command[1..].to_vec();
        let env_vars: HashMap<String, String> = request.env.clone();
        let sandbox_env: HashMap<String, String> = sb.spec.env_vars.clone();
        let timeout_secs = request.timeout_secs;

        let (tx, rx) = mpsc::channel::<String>(256);

        let (cancel_tx, mut cancel_rx) = tokio::sync::watch::channel(false);
        self.active_processes.insert(*id, cancel_tx);

        let handle = tokio::spawn(async move {
            use tokio::io::{AsyncBufReadExt, BufReader};

            let start = std::time::Instant::now();
            let mut cmd = Command::new(&program);
            cmd.args(&args)
                .current_dir(&workdir)
                .envs(&env_vars)
                .envs(&sandbox_env)
                // Remove CLAUDECODE env var so agent CLIs (e.g. Claude Code)
                // don't think they're running in a nested session.
                .env_remove("CLAUDECODE")
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped());

            let mut child = cmd
                .spawn()
                .map_err(|e| CiabError::ExecFailed(e.to_string()))?;

            let stdout = child.stdout.take();
            let stderr = child.stderr.take();

            let tx_out = tx.clone();
            let stdout_handle = tokio::spawn(async move {
                let mut all = String::new();
                if let Some(stdout) = stdout {
                    let mut reader = BufReader::new(stdout);
                    let mut line = String::new();
                    loop {
                        line.clear();
                        match reader.read_line(&mut line).await {
                            Ok(0) => break,
                            Ok(_) => {
                                let trimmed = line.trim_end_matches('\n').to_string();
                                all.push_str(&trimmed);
                                all.push('\n');
                                let _ = tx_out.send(trimmed).await;
                            }
                            Err(_) => break,
                        }
                    }
                }
                all
            });

            let stderr_handle = tokio::spawn(async move {
                let mut all = String::new();
                if let Some(stderr) = stderr {
                    let mut reader = BufReader::new(stderr);
                    let mut line = String::new();
                    loop {
                        line.clear();
                        match reader.read_line(&mut line).await {
                            Ok(0) => break,
                            Ok(_) => {
                                all.push_str(&line);
                            }
                            Err(_) => break,
                        }
                    }
                }
                all
            });

            let wait_result = tokio::select! {
                result = async {
                    if let Some(secs) = timeout_secs {
                        tokio::time::timeout(
                            std::time::Duration::from_secs(secs as u64),
                            child.wait(),
                        )
                        .await
                        .map_err(|_| CiabError::Timeout("exec command timed out".to_string()))?
                        .map_err(|e| CiabError::ExecFailed(e.to_string()))
                    } else {
                        child.wait().await.map_err(|e| CiabError::ExecFailed(e.to_string()))
                    }
                } => result?,
                _ = async {
                    loop {
                        if cancel_rx.changed().await.is_err() {
                            // Sender dropped — no cancellation.
                            futures::future::pending::<()>().await;
                        }
                        if *cancel_rx.borrow() {
                            break;
                        }
                    }
                } => {
                    let _ = child.kill().await;
                    return Err(CiabError::ExecFailed("process cancelled".to_string()));
                }
            };

            let stdout_text = stdout_handle.await.unwrap_or_default();
            let stderr_text = stderr_handle.await.unwrap_or_default();
            let duration = start.elapsed();

            Ok(ExecResult {
                exit_code: wait_result.code().unwrap_or(-1),
                stdout: stdout_text,
                stderr: stderr_text,
                duration_ms: duration.as_millis() as u64,
            })
        });

        Ok((rx, handle))
    }

    async fn exec_streaming_interactive(
        &self,
        id: &Uuid,
        request: &ExecRequest,
    ) -> CiabResult<(
        mpsc::Receiver<String>,
        mpsc::Sender<String>,
        tokio::task::JoinHandle<CiabResult<ExecResult>>,
    )> {
        let sb = self.get_sandbox_ref(id)?;
        if sb.state != SandboxState::Running {
            return Err(CiabError::SandboxInvalidState {
                current: sb.state.to_string(),
                expected: "running".to_string(),
            });
        }

        let workdir = request
            .workdir
            .as_ref()
            .map(PathBuf::from)
            .filter(|p| p.exists())
            .unwrap_or_else(|| sb.workdir.clone());

        if request.command.is_empty() {
            return Err(CiabError::ExecFailed("empty command".to_string()));
        }

        let program = request.command[0].clone();
        let args: Vec<String> = request.command[1..].to_vec();
        let env_vars: HashMap<String, String> = request.env.clone();
        let sandbox_env: HashMap<String, String> = sb.spec.env_vars.clone();
        let timeout_secs = request.timeout_secs;

        let (stdout_tx, stdout_rx) = mpsc::channel::<String>(256);
        let (stdin_tx, mut stdin_rx) = mpsc::channel::<String>(64);

        let (cancel_tx, mut cancel_rx) = tokio::sync::watch::channel(false);
        self.active_processes.insert(*id, cancel_tx);

        let handle = tokio::spawn(async move {
            use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

            let start = std::time::Instant::now();
            let mut cmd = Command::new(&program);
            cmd.args(&args)
                .current_dir(&workdir)
                .envs(&env_vars)
                .envs(&sandbox_env)
                // Remove CLAUDECODE env var so agent CLIs (e.g. Claude Code)
                // don't think they're running in a nested session.
                .env_remove("CLAUDECODE")
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped());

            let mut child = cmd
                .spawn()
                .map_err(|e| CiabError::ExecFailed(e.to_string()))?;

            let child_stdin = child.stdin.take();
            let stdout = child.stdout.take();
            let stderr = child.stderr.take();

            // Stdin writer task: reads from stdin_rx and writes to child's stdin.
            let stdin_handle = tokio::spawn(async move {
                if let Some(mut stdin) = child_stdin {
                    while let Some(line) = stdin_rx.recv().await {
                        let data = format!("{}\n", line);
                        if stdin.write_all(data.as_bytes()).await.is_err() {
                            break;
                        }
                        if stdin.flush().await.is_err() {
                            break;
                        }
                    }
                }
            });

            // Stdout reader task
            let tx_out = stdout_tx.clone();
            let stdout_handle = tokio::spawn(async move {
                let mut all = String::new();
                if let Some(stdout) = stdout {
                    let mut reader = BufReader::new(stdout);
                    let mut line = String::new();
                    loop {
                        line.clear();
                        match reader.read_line(&mut line).await {
                            Ok(0) => break,
                            Ok(_) => {
                                let trimmed = line.trim_end_matches('\n').to_string();
                                all.push_str(&trimmed);
                                all.push('\n');
                                let _ = tx_out.send(trimmed).await;
                            }
                            Err(_) => break,
                        }
                    }
                }
                all
            });

            // Stderr reader task
            let stderr_handle = tokio::spawn(async move {
                let mut all = String::new();
                if let Some(stderr) = stderr {
                    let mut reader = BufReader::new(stderr);
                    let mut line = String::new();
                    loop {
                        line.clear();
                        match reader.read_line(&mut line).await {
                            Ok(0) => break,
                            Ok(_) => {
                                all.push_str(&line);
                            }
                            Err(_) => break,
                        }
                    }
                }
                all
            });

            let wait_result = tokio::select! {
                result = async {
                    if let Some(secs) = timeout_secs {
                        tokio::time::timeout(
                            std::time::Duration::from_secs(secs as u64),
                            child.wait(),
                        )
                        .await
                        .map_err(|_| CiabError::Timeout("exec command timed out".to_string()))?
                        .map_err(|e| CiabError::ExecFailed(e.to_string()))
                    } else {
                        child.wait().await.map_err(|e| CiabError::ExecFailed(e.to_string()))
                    }
                } => result?,
                _ = async {
                    loop {
                        if cancel_rx.changed().await.is_err() {
                            futures::future::pending::<()>().await;
                        }
                        if *cancel_rx.borrow() {
                            break;
                        }
                    }
                } => {
                    let _ = child.kill().await;
                    stdin_handle.abort();
                    return Err(CiabError::ExecFailed("process cancelled".to_string()));
                }
            };

            stdin_handle.abort(); // Clean up stdin writer
            let stdout_text = stdout_handle.await.unwrap_or_default();
            let stderr_text = stderr_handle.await.unwrap_or_default();
            let duration = start.elapsed();

            Ok(ExecResult {
                exit_code: wait_result.code().unwrap_or(-1),
                stdout: stdout_text,
                stderr: stderr_text,
                duration_ms: duration.as_millis() as u64,
            })
        });

        Ok((stdout_rx, stdin_tx, handle))
    }

    async fn read_file(&self, id: &Uuid, path: &str) -> CiabResult<Vec<u8>> {
        let sb = self.get_sandbox_ref(id)?;
        let file_path = resolve_path(&sb.workdir, path);
        tokio::fs::read(&file_path)
            .await
            .map_err(|e| CiabError::FileNotFound(format!("{}: {}", path, e)))
    }

    async fn write_file(&self, id: &Uuid, path: &str, content: &[u8]) -> CiabResult<()> {
        let sb = self.get_sandbox_ref(id)?;
        let file_path = resolve_path(&sb.workdir, path);
        if let Some(parent) = file_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| CiabError::Internal(e.to_string()))?;
        }
        tokio::fs::write(&file_path, content)
            .await
            .map_err(|e| CiabError::Internal(format!("write file {}: {}", path, e)))
    }

    async fn list_files(&self, id: &Uuid, path: &str) -> CiabResult<Vec<FileInfo>> {
        let sb = self.get_sandbox_ref(id)?;
        let dir_path = resolve_path(&sb.workdir, path);

        let mut entries = tokio::fs::read_dir(&dir_path)
            .await
            .map_err(|e| CiabError::FileNotFound(format!("{}: {}", path, e)))?;

        let mut files = Vec::new();
        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| CiabError::Internal(e.to_string()))?
        {
            let metadata = entry
                .metadata()
                .await
                .map_err(|e| CiabError::Internal(e.to_string()))?;

            let modified_at = metadata
                .modified()
                .ok()
                .and_then(|t| {
                    t.duration_since(std::time::UNIX_EPOCH)
                        .ok()
                        .map(|d| chrono::DateTime::from_timestamp(d.as_secs() as i64, 0))
                })
                .flatten();

            files.push(FileInfo {
                path: entry
                    .path()
                    .strip_prefix(&sb.workdir)
                    .unwrap_or(entry.path().as_path())
                    .to_string_lossy()
                    .to_string(),
                size: metadata.len(),
                is_dir: metadata.is_dir(),
                mode: 0o644,
                modified_at,
            });
        }

        Ok(files)
    }

    async fn get_stats(&self, id: &Uuid) -> CiabResult<ResourceStats> {
        let _sb = self.get_sandbox_ref(id)?;
        // For local processes, return approximate stats
        Ok(ResourceStats {
            cpu_usage_percent: 0.0,
            memory_used_mb: 0,
            memory_limit_mb: 0,
            disk_used_mb: 0,
            disk_limit_mb: 0,
            network_rx_bytes: 0,
            network_tx_bytes: 0,
        })
    }

    async fn stream_logs(
        &self,
        _id: &Uuid,
        _options: &LogOptions,
    ) -> CiabResult<mpsc::Receiver<String>> {
        let (tx, rx) = mpsc::channel(256);
        // Local processes don't have a unified log stream;
        // send a placeholder message
        tokio::spawn(async move {
            let _ = tx.send("[local runtime] Log streaming not available for local processes. Use exec to run log commands.".to_string()).await;
        });
        Ok(rx)
    }

    async fn kill_exec(&self, id: &Uuid) -> CiabResult<()> {
        if let Some((_, tx)) = self.active_processes.remove(id) {
            let _ = tx.send(true);
        }
        Ok(())
    }
}

/// Resolve a path relative to the sandbox workdir, preventing path traversal.
fn resolve_path(workdir: &Path, path: &str) -> PathBuf {
    let clean = path.trim_start_matches('/');
    let resolved = workdir.join(clean);
    // Basic path traversal prevention
    if resolved.starts_with(workdir) {
        resolved
    } else {
        workdir.join(clean.replace("..", "_"))
    }
}
