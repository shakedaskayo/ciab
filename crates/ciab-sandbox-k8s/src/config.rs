use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KubernetesRuntimeConfig {
    /// Path to kubeconfig file. None = in-cluster config.
    #[serde(default)]
    pub kubeconfig: Option<String>,
    /// kubeconfig context name.
    #[serde(default)]
    pub context: Option<String>,
    /// Kubernetes namespace for agent Pods.
    #[serde(default = "default_namespace")]
    pub namespace: String,
    /// Container image to use for agent Pods.
    #[serde(default = "default_agent_image")]
    pub agent_image: String,
    /// RuntimeClass for microvm isolation (e.g. "kata-containers", "kata-qemu").
    /// This is the sole Kata Containers integration point.
    #[serde(default)]
    pub runtime_class: Option<String>,
    /// Node selector labels.
    #[serde(default)]
    pub node_selector: HashMap<String, String>,
    /// Pod tolerations for tainted nodes.
    #[serde(default)]
    pub tolerations: Vec<KubeToleration>,
    /// Image pull secrets (names of K8s secrets).
    #[serde(default)]
    pub image_pull_secrets: Vec<String>,
    /// Storage class for PVCs.
    #[serde(default)]
    pub storage_class: Option<String>,
    /// PVC size for workspace persistence.
    #[serde(default = "default_pvc_size")]
    pub workspace_pvc_size: String,
    /// Service account for agent Pods.
    #[serde(default)]
    pub service_account: Option<String>,
    /// Whether to create a NetworkPolicy isolating agent Pods.
    #[serde(default = "default_true")]
    pub create_network_policy: bool,
    /// Run containers as non-root.
    #[serde(default = "default_true")]
    pub run_as_non_root: bool,
    /// Drop all Linux capabilities from agent containers.
    #[serde(default = "default_true")]
    pub drop_all_capabilities: bool,
    /// Default CPU request (e.g. "500m").
    #[serde(default)]
    pub default_cpu_request: Option<String>,
    /// Default CPU limit (e.g. "2").
    #[serde(default)]
    pub default_cpu_limit: Option<String>,
    /// Default memory request (e.g. "256Mi").
    #[serde(default)]
    pub default_memory_request: Option<String>,
    /// Default memory limit (e.g. "2Gi").
    #[serde(default)]
    pub default_memory_limit: Option<String>,
}

impl Default for KubernetesRuntimeConfig {
    fn default() -> Self {
        Self {
            kubeconfig: None,
            context: None,
            namespace: default_namespace(),
            agent_image: default_agent_image(),
            runtime_class: None,
            node_selector: HashMap::new(),
            tolerations: Vec::new(),
            image_pull_secrets: Vec::new(),
            storage_class: None,
            workspace_pvc_size: default_pvc_size(),
            service_account: None,
            create_network_policy: true,
            run_as_non_root: true,
            drop_all_capabilities: true,
            default_cpu_request: None,
            default_cpu_limit: None,
            default_memory_request: None,
            default_memory_limit: None,
        }
    }
}

fn default_namespace() -> String {
    "ciab-agents".to_string()
}

fn default_agent_image() -> String {
    "ghcr.io/shakedaskayo/ciab-claude:latest".to_string()
}

fn default_pvc_size() -> String {
    "10Gi".to_string()
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KubeToleration {
    pub key: String,
    #[serde(default = "default_exists")]
    pub operator: String,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub effect: Option<String>,
}

fn default_exists() -> String {
    "Exists".to_string()
}
