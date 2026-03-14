use std::collections::HashMap;

use async_trait::async_trait;
use dashmap::DashMap;
use tokio::sync::mpsc;
use tracing;
use uuid::Uuid;

use ciab_core::error::{CiabError, CiabResult};
use ciab_core::traits::runtime::SandboxRuntime;
use ciab_core::types::sandbox::{
    ExecRequest, ExecResult, FileInfo, LogOptions, ResourceStats, SandboxInfo, SandboxPersistence,
    SandboxSpec, SandboxState,
};

use crate::client::{CreateSandboxRequest, OpenSandboxClient};
use crate::execd::ExecdClient;

pub struct OpenSandboxRuntime {
    client: OpenSandboxClient,
    execd_clients: DashMap<String, ExecdClient>,
}

impl OpenSandboxRuntime {
    pub fn new(opensandbox_url: String, api_key: Option<String>) -> Self {
        Self {
            client: OpenSandboxClient::new(opensandbox_url, api_key),
            execd_clients: DashMap::new(),
        }
    }

    fn get_or_create_execd(&self, sandbox_id: &str, endpoint_url: &str) -> ExecdClient {
        if let Some(client) = self.execd_clients.get(sandbox_id) {
            return client.clone();
        }
        let client = ExecdClient::new(endpoint_url.to_string());
        self.execd_clients
            .insert(sandbox_id.to_string(), client.clone());
        client
    }

    async fn resolve_execd(&self, id: &Uuid) -> CiabResult<ExecdClient> {
        let sandbox_id = id.to_string();
        if let Some(client) = self.execd_clients.get(&sandbox_id) {
            return Ok(client.clone());
        }
        let info = self.client.get_sandbox(&sandbox_id).await?;
        let endpoint_url = info.endpoint_url.ok_or_else(|| {
            CiabError::OpenSandboxError(format!("sandbox {} has no endpoint URL", sandbox_id))
        })?;
        Ok(self.get_or_create_execd(&sandbox_id, &endpoint_url))
    }

    fn map_status_to_state(status: &str) -> SandboxState {
        match status {
            "running" => SandboxState::Running,
            "paused" => SandboxState::Paused,
            "stopped" => SandboxState::Stopped,
            "creating" => SandboxState::Creating,
            "pending" => SandboxState::Pending,
            "failed" => SandboxState::Failed,
            "terminated" => SandboxState::Terminated,
            "pausing" => SandboxState::Pausing,
            "stopping" => SandboxState::Stopping,
            _ => SandboxState::Running,
        }
    }
}

#[async_trait]
impl SandboxRuntime for OpenSandboxRuntime {
    async fn create_sandbox(&self, spec: &SandboxSpec) -> CiabResult<SandboxInfo> {
        let request = CreateSandboxRequest {
            image: spec
                .image
                .clone()
                .unwrap_or_else(|| "ubuntu:latest".to_string()),
            cpu: spec.resource_limits.as_ref().map(|r| r.cpu_cores),
            memory_mb: spec.resource_limits.as_ref().map(|r| r.memory_mb),
            disk_mb: spec.resource_limits.as_ref().map(|r| r.disk_mb),
            env: spec.env_vars.clone(),
            ports: spec.ports.clone(),
            timeout_secs: spec.timeout_secs.map(|t| t as u64),
            labels: spec.labels.clone(),
        };

        let resp = self.client.create_sandbox(&request).await?;

        // Cache execd client if endpoint available
        if let Some(ref url) = resp.endpoint_url {
            self.get_or_create_execd(&resp.id, url);
        }

        let sandbox_id = Uuid::parse_str(&resp.id).unwrap_or_else(|_| Uuid::new_v4());
        let created_at = chrono::DateTime::parse_from_rfc3339(&resp.created_at)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(|_| chrono::Utc::now());

        Ok(SandboxInfo {
            id: sandbox_id,
            name: spec.name.clone(),
            state: Self::map_status_to_state(&resp.status),
            persistence: spec.persistence.clone(),
            agent_provider: spec.agent_provider.clone(),
            endpoint_url: resp.endpoint_url,
            resource_stats: None,
            labels: resp.labels,
            created_at,
            updated_at: created_at,
            spec: spec.clone(),
        })
    }

    async fn get_sandbox(&self, id: &Uuid) -> CiabResult<SandboxInfo> {
        let resp = self.client.get_sandbox(&id.to_string()).await?;
        let created_at = chrono::DateTime::parse_from_rfc3339(&resp.created_at)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(|_| chrono::Utc::now());

        Ok(SandboxInfo {
            id: *id,
            name: None,
            state: Self::map_status_to_state(&resp.status),
            persistence: SandboxPersistence::Ephemeral,
            agent_provider: "opensandbox".to_string(),
            endpoint_url: resp.endpoint_url,
            resource_stats: None,
            labels: resp.labels,
            created_at,
            updated_at: created_at,
            spec: SandboxSpec {
                name: None,
                agent_provider: "opensandbox".to_string(),
                image: None,
                resource_limits: None,
                persistence: SandboxPersistence::Ephemeral,
                network: None,
                env_vars: HashMap::new(),
                volumes: vec![],
                ports: vec![],
                git_repos: vec![],
                credentials: vec![],
                provisioning_scripts: vec![],
                labels: HashMap::new(),
                timeout_secs: None,
                agent_config: None,
                local_mounts: vec![],
                runtime_backend: None,
            },
        })
    }

    async fn list_sandboxes(
        &self,
        state: Option<SandboxState>,
        provider: Option<&str>,
        labels: &HashMap<String, String>,
    ) -> CiabResult<Vec<SandboxInfo>> {
        let sandboxes = self.client.list_sandboxes().await?;
        let mut results: Vec<SandboxInfo> = sandboxes
            .into_iter()
            .map(|resp| {
                let sandbox_id = Uuid::parse_str(&resp.id).unwrap_or_else(|_| Uuid::new_v4());
                let created_at = chrono::DateTime::parse_from_rfc3339(&resp.created_at)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(|_| chrono::Utc::now());

                SandboxInfo {
                    id: sandbox_id,
                    name: None,
                    state: Self::map_status_to_state(&resp.status),
                    persistence: SandboxPersistence::Ephemeral,
                    agent_provider: "opensandbox".to_string(),
                    endpoint_url: resp.endpoint_url,
                    resource_stats: None,
                    labels: resp.labels,
                    created_at,
                    updated_at: created_at,
                    spec: SandboxSpec {
                        name: None,
                        agent_provider: "opensandbox".to_string(),
                        image: None,
                        resource_limits: None,
                        persistence: SandboxPersistence::Ephemeral,
                        network: None,
                        env_vars: HashMap::new(),
                        volumes: vec![],
                        ports: vec![],
                        git_repos: vec![],
                        credentials: vec![],
                        provisioning_scripts: vec![],
                        labels: HashMap::new(),
                        timeout_secs: None,
                        agent_config: None,
                        local_mounts: vec![],
                        runtime_backend: None,
                    },
                }
            })
            .collect();

        // Apply filters
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
        // OpenSandbox auto-starts on create; resume if paused
        tracing::debug!("start_sandbox called for {}, attempting resume", id);
        self.client.resume_sandbox(&id.to_string()).await
    }

    async fn stop_sandbox(&self, id: &Uuid) -> CiabResult<()> {
        // OpenSandbox doesn't have a stop concept; map to pause
        tracing::debug!("stop_sandbox called for {}, mapping to pause", id);
        self.client.pause_sandbox(&id.to_string()).await
    }

    async fn pause_sandbox(&self, id: &Uuid) -> CiabResult<()> {
        self.client.pause_sandbox(&id.to_string()).await
    }

    async fn resume_sandbox(&self, id: &Uuid) -> CiabResult<()> {
        self.client.resume_sandbox(&id.to_string()).await
    }

    async fn terminate_sandbox(&self, id: &Uuid) -> CiabResult<()> {
        let sandbox_id = id.to_string();
        self.client.delete_sandbox(&sandbox_id).await?;
        self.execd_clients.remove(&sandbox_id);
        Ok(())
    }

    async fn exec(&self, id: &Uuid, request: &ExecRequest) -> CiabResult<ExecResult> {
        let execd = self.resolve_execd(id).await?;
        execd.run_command(request).await
    }

    async fn read_file(&self, id: &Uuid, path: &str) -> CiabResult<Vec<u8>> {
        let execd = self.resolve_execd(id).await?;
        execd.download_file(path).await
    }

    async fn write_file(&self, id: &Uuid, path: &str, content: &[u8]) -> CiabResult<()> {
        let execd = self.resolve_execd(id).await?;
        execd.upload_file(path, content, 0o644).await
    }

    async fn list_files(&self, id: &Uuid, path: &str) -> CiabResult<Vec<FileInfo>> {
        let execd = self.resolve_execd(id).await?;
        execd.list_files(path).await
    }

    async fn get_stats(&self, id: &Uuid) -> CiabResult<ResourceStats> {
        let execd = self.resolve_execd(id).await?;
        execd.get_metrics().await
    }

    async fn stream_logs(
        &self,
        id: &Uuid,
        options: &LogOptions,
    ) -> CiabResult<mpsc::Receiver<String>> {
        let (tx, rx) = mpsc::channel(256);
        let execd = self.resolve_execd(id).await?;

        let mut cmd = vec!["tail".to_string()];
        if options.follow {
            cmd.push("-f".to_string());
        }
        if let Some(tail_lines) = options.tail {
            cmd.push("-n".to_string());
            cmd.push(tail_lines.to_string());
        }
        cmd.push("/var/log/syslog".to_string());

        let request = ExecRequest {
            command: cmd,
            workdir: None,
            env: HashMap::new(),
            stdin: None,
            timeout_secs: None,
            tty: false,
        };

        let sandbox_id = *id;
        tokio::spawn(async move {
            let (stream_tx, mut stream_rx) =
                mpsc::channel::<ciab_core::types::stream::StreamEvent>(256);

            let stream_handle = {
                let execd = execd.clone();
                let request = request.clone();
                tokio::spawn(async move {
                    let _ = execd
                        .run_command_stream(&request, stream_tx, sandbox_id)
                        .await;
                })
            };

            while let Some(event) = stream_rx.recv().await {
                if let Some(text) = event.data.as_str() {
                    if tx.send(text.to_string()).await.is_err() {
                        break;
                    }
                } else {
                    let text = event.data.to_string();
                    if tx.send(text).await.is_err() {
                        break;
                    }
                }
            }

            let _ = stream_handle.await;
        });

        Ok(rx)
    }
}
