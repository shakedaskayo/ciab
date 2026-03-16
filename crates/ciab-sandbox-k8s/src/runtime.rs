use std::collections::HashMap;

use async_trait::async_trait;
use chrono::Utc;
use dashmap::DashMap;
use k8s_openapi::api::core::v1::Pod;
use kube::{Api, Client};
use uuid::Uuid;

use ciab_core::error::{CiabError, CiabResult};
use ciab_core::traits::runtime::SandboxRuntime;
use ciab_core::types::sandbox::{
    ExecRequest, ExecResult, FileInfo, LogOptions, ResourceStats, SandboxInfo,
    SandboxPersistence, SandboxSpec, SandboxState,
};

use crate::config::KubernetesRuntimeConfig;
use crate::error::K8sError;
use crate::pod_builder::{build_pod, pod_name};
use crate::{exec, logs, pvc, rbac};

/// Metadata stored in-memory for each sandbox managed by this runtime.
#[derive(Debug, Clone)]
struct KubeSandboxMeta {
    spec: SandboxSpec,
    created_at: chrono::DateTime<Utc>,
}

pub struct KubernetesRuntime {
    client: Client,
    config: KubernetesRuntimeConfig,
    sandboxes: DashMap<Uuid, KubeSandboxMeta>,
}

impl KubernetesRuntime {
    /// Create a new KubernetesRuntime. Attempts in-cluster config first, then kubeconfig.
    pub async fn new(config: KubernetesRuntimeConfig) -> Result<Self, K8sError> {
        let client = if let Some(ref kc_path) = config.kubeconfig {
            let kube_config = kube::config::Kubeconfig::read_from(kc_path)
                .map_err(|e| K8sError::ExecFailed(format!("kubeconfig: {}", e)))?;
            let options = kube::config::KubeConfigOptions {
                context: config.context.clone(),
                ..Default::default()
            };
            let client_config = kube::Config::from_custom_kubeconfig(kube_config, &options)
                .await
                .map_err(|e| K8sError::ExecFailed(format!("kube config: {}", e)))?;
            Client::try_from(client_config)
                .map_err(|e| K8sError::ExecFailed(format!("kube client: {}", e)))?
        } else {
            Client::try_default()
                .await
                .map_err(|e| K8sError::ExecFailed(format!("kube client: {}", e)))?
        };

        let runtime = Self {
            client,
            config,
            sandboxes: DashMap::new(),
        };

        // Idempotently set up RBAC for agent Pods
        if let Err(e) = rbac::ensure_rbac(&runtime.client, &runtime.config.namespace).await {
            tracing::warn!(error = %e, "failed to ensure RBAC (continuing anyway)");
        }

        Ok(runtime)
    }

    /// Map K8s Pod phase to SandboxState.
    fn phase_to_state(phase: Option<&str>) -> SandboxState {
        match phase {
            Some("Running") => SandboxState::Running,
            Some("Succeeded") | Some("Failed") => SandboxState::Stopped,
            Some("Pending") => SandboxState::Creating,
            _ => SandboxState::Creating,
        }
    }

    async fn get_pod(&self, sandbox_id: &Uuid) -> Result<Option<Pod>, K8sError> {
        let api: Api<Pod> = Api::namespaced(self.client.clone(), &self.config.namespace);
        let name = pod_name(sandbox_id);
        Ok(api.get_opt(&name).await?)
    }

    async fn wait_for_running(&self, sandbox_id: &Uuid, timeout_secs: u64) -> Result<(), K8sError> {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(timeout_secs);
        loop {
            if std::time::Instant::now() > deadline {
                return Err(K8sError::PodTimeout(sandbox_id.to_string()));
            }
            if let Some(pod) = self.get_pod(sandbox_id).await? {
                let phase = pod
                    .status
                    .as_ref()
                    .and_then(|s| s.phase.as_deref());
                match phase {
                    Some("Running") => return Ok(()),
                    Some("Failed") => {
                        return Err(K8sError::ExecFailed(format!(
                            "pod {} entered Failed state",
                            sandbox_id
                        )))
                    }
                    _ => {}
                }
            }
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
    }

    async fn build_sandbox_info(&self, sandbox_id: &Uuid) -> CiabResult<SandboxInfo> {
        let meta = self
            .sandboxes
            .get(sandbox_id)
            .ok_or_else(|| CiabError::SandboxNotFound(sandbox_id.to_string()))?;

        let state = match self.get_pod(sandbox_id).await {
            Ok(Some(pod)) => {
                let phase = pod.status.as_ref().and_then(|s| s.phase.as_deref());
                Self::phase_to_state(phase)
            }
            Ok(None) => SandboxState::Terminated,
            Err(e) => {
                tracing::warn!(error = %e, "failed to get pod state");
                SandboxState::Failed
            }
        };

        let now = Utc::now();
        Ok(SandboxInfo {
            id: *sandbox_id,
            name: meta.spec.name.clone(),
            state,
            persistence: meta.spec.persistence.clone(),
            agent_provider: meta.spec.agent_provider.clone(),
            endpoint_url: None,
            resource_stats: None,
            labels: meta.spec.labels.clone(),
            created_at: meta.created_at,
            updated_at: now,
            spec: meta.spec.clone(),
        })
    }
}

#[async_trait]
impl SandboxRuntime for KubernetesRuntime {
    async fn create_sandbox(
        &self,
        spec: &SandboxSpec,
    ) -> CiabResult<SandboxInfo> {
        let sandbox_id = Uuid::new_v4();
        let api: Api<Pod> = Api::namespaced(self.client.clone(), &self.config.namespace);

        // If persistent, create PVC first
        if matches!(spec.persistence, SandboxPersistence::Persistent) {
            pvc::create_pvc(
                &self.client,
                &self.config.namespace,
                &sandbox_id,
                &self.config.workspace_pvc_size,
                self.config.storage_class.as_deref(),
            )
            .await
            .map_err(|e| CiabError::RuntimeUnavailable(e.to_string()))?;
        }

        let pod = build_pod(&sandbox_id, spec, &self.config);
        api.create(&kube::api::PostParams::default(), &pod)
            .await
            .map_err(|e| CiabError::RuntimeUnavailable(e.to_string()))?;

        // Wait up to 5 minutes for Running
        self.wait_for_running(&sandbox_id, 300)
            .await
            .map_err(CiabError::from)?;

        self.sandboxes.insert(
            sandbox_id,
            KubeSandboxMeta {
                spec: spec.clone(),
                created_at: Utc::now(),
            },
        );

        self.build_sandbox_info(&sandbox_id).await
    }

    async fn get_sandbox(&self, id: &Uuid) -> CiabResult<SandboxInfo> {
        self.build_sandbox_info(id).await
    }

    async fn list_sandboxes(
        &self,
        state: Option<SandboxState>,
        provider: Option<&str>,
        labels: &HashMap<String, String>,
    ) -> CiabResult<Vec<SandboxInfo>> {
        let mut infos = Vec::new();
        for entry in self.sandboxes.iter() {
            match self.build_sandbox_info(entry.key()).await {
                Ok(info) => {
                    // Filter by state
                    if let Some(ref s) = state {
                        if &info.state != s {
                            continue;
                        }
                    }
                    // Filter by provider
                    if let Some(p) = provider {
                        if info.agent_provider != p {
                            continue;
                        }
                    }
                    // Filter by labels
                    if !labels.is_empty() {
                        let matches = labels
                            .iter()
                            .all(|(k, v)| info.labels.get(k).map(|lv| lv == v).unwrap_or(false));
                        if !matches {
                            continue;
                        }
                    }
                    infos.push(info);
                }
                Err(e) => tracing::warn!(error = %e, "failed to build sandbox info"),
            }
        }
        Ok(infos)
    }

    async fn terminate_sandbox(&self, id: &Uuid) -> CiabResult<()> {
        let api: Api<Pod> = Api::namespaced(self.client.clone(), &self.config.namespace);
        let name = pod_name(id);

        match api
            .delete(&name, &kube::api::DeleteParams::default())
            .await
        {
            Ok(_) => {}
            Err(kube::Error::Api(e)) if e.code == 404 => {}
            Err(e) => return Err(CiabError::RuntimeUnavailable(e.to_string())),
        }

        // Delete PVC regardless of persistence (cleanup)
        let _ = pvc::delete_pvc(&self.client, &self.config.namespace, id).await;

        self.sandboxes.remove(id);
        Ok(())
    }

    async fn stop_sandbox(&self, id: &Uuid) -> CiabResult<()> {
        let api: Api<Pod> = Api::namespaced(self.client.clone(), &self.config.namespace);
        let name = pod_name(id);

        match api
            .delete(&name, &kube::api::DeleteParams::default())
            .await
        {
            Ok(_) => {}
            Err(kube::Error::Api(e)) if e.code == 404 => {}
            Err(e) => return Err(CiabError::RuntimeUnavailable(e.to_string())),
        }
        // PVC is kept (stop != terminate)
        Ok(())
    }

    async fn start_sandbox(&self, id: &Uuid) -> CiabResult<()> {
        let meta = self
            .sandboxes
            .get(id)
            .ok_or_else(|| CiabError::SandboxNotFound(id.to_string()))?
            .clone();

        let api: Api<Pod> = Api::namespaced(self.client.clone(), &self.config.namespace);
        let pod = build_pod(id, &meta.spec, &self.config);
        api.create(&kube::api::PostParams::default(), &pod)
            .await
            .map_err(|e| CiabError::RuntimeUnavailable(e.to_string()))?;

        self.wait_for_running(id, 300)
            .await
            .map_err(CiabError::from)?;

        Ok(())
    }

    async fn pause_sandbox(&self, id: &Uuid) -> CiabResult<()> {
        // K8s doesn't support native pause; treat as stop
        self.stop_sandbox(id).await
    }

    async fn resume_sandbox(&self, id: &Uuid) -> CiabResult<()> {
        self.start_sandbox(id).await
    }

    async fn exec(&self, id: &Uuid, request: &ExecRequest) -> CiabResult<ExecResult> {
        let pod = self
            .get_pod(id)
            .await
            .map_err(CiabError::from)?
            .ok_or_else(|| CiabError::SandboxNotFound(id.to_string()))?;

        let pod_name = pod
            .metadata
            .name
            .ok_or_else(|| CiabError::RuntimeUnavailable("pod has no name".to_string()))?;

        exec::exec_command(
            &self.client,
            &self.config.namespace,
            &pod_name,
            &request.command,
            request.workdir.as_deref(),
        )
        .await
        .map_err(CiabError::from)
    }

    async fn read_file(&self, id: &Uuid, path: &str) -> CiabResult<Vec<u8>> {
        use base64::{engine::general_purpose::STANDARD, Engine};

        let result = self
            .exec(
                id,
                &ExecRequest {
                    command: vec!["sh".to_string(), "-c".to_string(), format!("base64 < {}", path)],
                    workdir: None,
                    env: HashMap::new(),
                    stdin: None,
                    timeout_secs: None,
                    tty: false,
                },
            )
            .await?;

        if result.exit_code != 0 {
            return Err(CiabError::FileNotFound(path.to_string()));
        }

        STANDARD
            .decode(result.stdout.trim())
            .map_err(|e| CiabError::Internal(format!("base64 decode: {}", e)))
    }

    async fn write_file(&self, id: &Uuid, path: &str, content: &[u8]) -> CiabResult<()> {
        use base64::{engine::general_purpose::STANDARD, Engine};
        let encoded = STANDARD.encode(content);
        let script = format!(
            "mkdir -p $(dirname {path}) && echo '{encoded}' | base64 -d > {path}",
            path = path,
            encoded = encoded
        );
        let result = self
            .exec(
                id,
                &ExecRequest {
                    command: vec!["sh".to_string(), "-c".to_string(), script],
                    workdir: None,
                    env: HashMap::new(),
                    stdin: None,
                    timeout_secs: None,
                    tty: false,
                },
            )
            .await?;

        if result.exit_code != 0 {
            return Err(CiabError::ExecFailed(result.stderr));
        }
        Ok(())
    }

    async fn list_files(&self, id: &Uuid, path: &str) -> CiabResult<Vec<FileInfo>> {
        let result = self
            .exec(
                id,
                &ExecRequest {
                    command: vec!["sh".to_string(), "-c".to_string(), format!("ls -la {}", path)],
                    workdir: None,
                    env: HashMap::new(),
                    stdin: None,
                    timeout_secs: None,
                    tty: false,
                },
            )
            .await?;

        let mut files = Vec::new();
        for line in result.stdout.lines().skip(1) {
            // Parse ls -la output (simplified)
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 9 {
                continue;
            }
            let is_dir = parts[0].starts_with('d');
            let size: u64 = parts[4].parse().unwrap_or(0);
            let name = parts[8];
            if name == "." || name == ".." {
                continue;
            }
            files.push(FileInfo {
                path: format!("{}/{}", path.trim_end_matches('/'), name),
                size,
                is_dir,
                mode: 0o644,
                modified_at: None,
            });
        }
        Ok(files)
    }

    async fn get_stats(&self, _id: &Uuid) -> CiabResult<ResourceStats> {
        // Return zeroed stats — metrics-server integration is optional
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
        id: &Uuid,
        options: &LogOptions,
    ) -> CiabResult<tokio::sync::mpsc::Receiver<String>> {
        let pod = self
            .get_pod(id)
            .await
            .map_err(CiabError::from)?
            .ok_or_else(|| CiabError::SandboxNotFound(id.to_string()))?;

        let pod_name = pod
            .metadata
            .name
            .ok_or_else(|| CiabError::RuntimeUnavailable("pod has no name".to_string()))?;

        let _ = options; // LogOptions forwarding to kube LogParams can be extended later

        logs::stream_logs(&self.client, &self.config.namespace, &pod_name)
            .await
            .map_err(CiabError::from)
    }
}
