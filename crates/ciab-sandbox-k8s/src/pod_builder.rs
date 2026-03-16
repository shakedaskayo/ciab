use std::collections::BTreeMap;

use k8s_openapi::api::core::v1::{
    Capabilities, Container, EmptyDirVolumeSource, EnvVar, LocalObjectReference,
    PersistentVolumeClaimVolumeSource, Pod, PodSpec, ResourceRequirements, SecurityContext,
    Toleration, Volume, VolumeMount,
};
use k8s_openapi::apimachinery::pkg::api::resource::Quantity;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use uuid::Uuid;

use crate::config::KubernetesRuntimeConfig;
use ciab_core::types::sandbox::{SandboxPersistence, SandboxSpec};

/// Sanitize a UUID into a valid DNS label for use as a Pod name.
pub fn pod_name(sandbox_id: &Uuid) -> String {
    format!("ciab-{}", sandbox_id)
}

/// Build a Kubernetes Pod object from a SandboxSpec.
pub fn build_pod(sandbox_id: &Uuid, spec: &SandboxSpec, config: &KubernetesRuntimeConfig) -> Pod {
    let name = pod_name(sandbox_id);

    let mut labels = BTreeMap::new();
    labels.insert("ciab/sandbox-id".to_string(), sandbox_id.to_string());
    labels.insert("ciab/managed-by".to_string(), "ciab".to_string());

    // Environment variables
    let env: Vec<EnvVar> = spec
        .env_vars
        .iter()
        .map(|(k, v)| EnvVar {
            name: k.clone(),
            value: Some(v.clone()),
            ..Default::default()
        })
        .collect();

    // Resource requests/limits
    let resources = build_resources(spec, config);

    // Security context for the container
    let container_security_context = if config.drop_all_capabilities {
        Some(SecurityContext {
            capabilities: Some(Capabilities {
                drop: Some(vec!["ALL".to_string()]),
                ..Default::default()
            }),
            allow_privilege_escalation: Some(false),
            read_only_root_filesystem: Some(false),
            ..Default::default()
        })
    } else {
        None
    };

    // Pod-level security context
    let pod_security_context = if config.run_as_non_root {
        Some(k8s_openapi::api::core::v1::PodSecurityContext {
            run_as_non_root: Some(true),
            run_as_user: Some(1000),
            fs_group: Some(1000),
            ..Default::default()
        })
    } else {
        None
    };

    // Volumes and volume mounts
    let (volumes, volume_mounts) = build_volumes(sandbox_id, spec, config);

    // Image: prefer spec.image, then config.agent_image
    let image = spec
        .image
        .clone()
        .unwrap_or_else(|| config.agent_image.clone());

    let container = Container {
        name: "agent".to_string(),
        image: Some(image),
        env: if env.is_empty() { None } else { Some(env) },
        resources: Some(resources),
        security_context: container_security_context,
        volume_mounts: if volume_mounts.is_empty() {
            None
        } else {
            Some(volume_mounts)
        },
        working_dir: Some("/workspace".to_string()),
        ..Default::default()
    };

    // Tolerations
    let tolerations: Vec<Toleration> = config
        .tolerations
        .iter()
        .map(|t| Toleration {
            key: Some(t.key.clone()),
            operator: Some(t.operator.clone()),
            value: t.value.clone(),
            effect: t.effect.clone(),
            ..Default::default()
        })
        .collect();

    // Node selector
    let node_selector: BTreeMap<String, String> =
        config.node_selector.clone().into_iter().collect();

    // Image pull secrets
    let image_pull_secrets: Vec<LocalObjectReference> = config
        .image_pull_secrets
        .iter()
        .map(|s| LocalObjectReference { name: s.clone() })
        .collect();

    let mut pod_spec = PodSpec {
        containers: vec![container],
        restart_policy: Some("Never".to_string()),
        security_context: pod_security_context,
        tolerations: if tolerations.is_empty() {
            None
        } else {
            Some(tolerations)
        },
        node_selector: if node_selector.is_empty() {
            None
        } else {
            Some(node_selector)
        },
        image_pull_secrets: if image_pull_secrets.is_empty() {
            None
        } else {
            Some(image_pull_secrets)
        },
        service_account_name: config.service_account.clone(),
        volumes: if volumes.is_empty() {
            None
        } else {
            Some(volumes)
        },
        ..Default::default()
    };

    // microvm / Kata Containers integration: the sole integration point
    if let Some(ref rc) = config.runtime_class {
        pod_spec.runtime_class_name = Some(rc.clone());
    }

    Pod {
        metadata: ObjectMeta {
            name: Some(name),
            namespace: Some(config.namespace.clone()),
            labels: Some(labels),
            ..Default::default()
        },
        spec: Some(pod_spec),
        ..Default::default()
    }
}

fn build_resources(spec: &SandboxSpec, config: &KubernetesRuntimeConfig) -> ResourceRequirements {
    let mut requests = BTreeMap::new();
    let mut limits = BTreeMap::new();

    // CPU
    let cpu_req = spec
        .resource_limits
        .as_ref()
        .map(|r| format!("{}", r.cpu_cores))
        .or_else(|| config.default_cpu_request.clone());
    let cpu_lim = config.default_cpu_limit.clone().or_else(|| cpu_req.clone());

    if let Some(v) = cpu_req {
        requests.insert("cpu".to_string(), Quantity(v));
    }
    if let Some(v) = cpu_lim {
        limits.insert("cpu".to_string(), Quantity(v));
    }

    // Memory
    let mem_req = spec
        .resource_limits
        .as_ref()
        .map(|r| format!("{}Mi", r.memory_mb))
        .or_else(|| config.default_memory_request.clone());
    let mem_lim = config
        .default_memory_limit
        .clone()
        .or_else(|| mem_req.clone());

    if let Some(v) = mem_req {
        requests.insert("memory".to_string(), Quantity(v));
    }
    if let Some(v) = mem_lim {
        limits.insert("memory".to_string(), Quantity(v));
    }

    ResourceRequirements {
        requests: if requests.is_empty() {
            None
        } else {
            Some(requests)
        },
        limits: if limits.is_empty() {
            None
        } else {
            Some(limits)
        },
        ..Default::default()
    }
}

fn build_volumes(
    sandbox_id: &Uuid,
    spec: &SandboxSpec,
    config: &KubernetesRuntimeConfig,
) -> (Vec<Volume>, Vec<VolumeMount>) {
    let mut volumes = Vec::new();
    let mut mounts = Vec::new();

    let is_persistent = matches!(spec.persistence, SandboxPersistence::Persistent);

    let workspace_volume = if is_persistent {
        Volume {
            name: "workspace".to_string(),
            persistent_volume_claim: Some(PersistentVolumeClaimVolumeSource {
                claim_name: crate::pvc::pvc_name(sandbox_id),
                read_only: Some(false),
            }),
            ..Default::default()
        }
    } else {
        Volume {
            name: "workspace".to_string(),
            empty_dir: Some(EmptyDirVolumeSource {
                medium: None,
                size_limit: None,
            }),
            ..Default::default()
        }
    };

    volumes.push(workspace_volume);
    mounts.push(VolumeMount {
        name: "workspace".to_string(),
        mount_path: "/workspace".to_string(),
        ..Default::default()
    });

    let _ = config; // suppress unused warning

    (volumes, mounts)
}
