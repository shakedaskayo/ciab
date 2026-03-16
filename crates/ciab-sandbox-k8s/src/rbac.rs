use k8s_openapi::api::core::v1::ServiceAccount;
use k8s_openapi::api::rbac::v1::{PolicyRule, Role, RoleBinding, RoleRef, Subject};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::{Api, Client};

use crate::error::K8sError;

/// Idempotently create ServiceAccount, Role, and RoleBinding for agent Pods.
pub async fn ensure_rbac(client: &Client, namespace: &str) -> Result<(), K8sError> {
    ensure_service_account(client, namespace).await?;
    ensure_role(client, namespace).await?;
    ensure_role_binding(client, namespace).await?;
    Ok(())
}

async fn ensure_service_account(client: &Client, namespace: &str) -> Result<(), K8sError> {
    let api: Api<ServiceAccount> = Api::namespaced(client.clone(), namespace);
    if api.get_opt("ciab-agent").await?.is_none() {
        let sa = ServiceAccount {
            metadata: ObjectMeta {
                name: Some("ciab-agent".to_string()),
                namespace: Some(namespace.to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        api.create(&kube::api::PostParams::default(), &sa).await?;
    }
    Ok(())
}

async fn ensure_role(client: &Client, namespace: &str) -> Result<(), K8sError> {
    let api: Api<Role> = Api::namespaced(client.clone(), namespace);
    if api.get_opt("ciab-agent").await?.is_none() {
        let role = Role {
            metadata: ObjectMeta {
                name: Some("ciab-agent".to_string()),
                namespace: Some(namespace.to_string()),
                ..Default::default()
            },
            rules: Some(vec![PolicyRule {
                api_groups: Some(vec!["".to_string()]),
                resources: Some(vec!["pods".to_string()]),
                verbs: vec!["get".to_string(), "list".to_string()],
                ..Default::default()
            }]),
        };
        api.create(&kube::api::PostParams::default(), &role).await?;
    }
    Ok(())
}

async fn ensure_role_binding(client: &Client, namespace: &str) -> Result<(), K8sError> {
    let api: Api<RoleBinding> = Api::namespaced(client.clone(), namespace);
    if api.get_opt("ciab-agent").await?.is_none() {
        let rb = RoleBinding {
            metadata: ObjectMeta {
                name: Some("ciab-agent".to_string()),
                namespace: Some(namespace.to_string()),
                ..Default::default()
            },
            role_ref: RoleRef {
                api_group: "rbac.authorization.k8s.io".to_string(),
                kind: "Role".to_string(),
                name: "ciab-agent".to_string(),
            },
            subjects: Some(vec![Subject {
                kind: "ServiceAccount".to_string(),
                name: "ciab-agent".to_string(),
                namespace: Some(namespace.to_string()),
                ..Default::default()
            }]),
        };
        api.create(&kube::api::PostParams::default(), &rb).await?;
    }
    Ok(())
}
