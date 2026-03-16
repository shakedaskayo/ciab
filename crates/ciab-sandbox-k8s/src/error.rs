use ciab_core::error::CiabError;

#[derive(Debug, thiserror::Error)]
pub enum K8sError {
    #[error("kube error: {0}")]
    Kube(#[from] kube::Error),
    #[error("pod not found: {0}")]
    PodNotFound(String),
    #[error("exec failed: {0}")]
    ExecFailed(String),
    #[error("timeout waiting for pod: {0}")]
    PodTimeout(String),
}

impl From<K8sError> for CiabError {
    fn from(e: K8sError) -> Self {
        match e {
            K8sError::PodNotFound(s) => CiabError::KubernetesPodNotFound(s),
            K8sError::PodTimeout(s) => CiabError::SandboxTimeout(s),
            K8sError::ExecFailed(s) => CiabError::ExecFailed(s),
            K8sError::Kube(inner) => CiabError::KubernetesError(inner.to_string()),
        }
    }
}
