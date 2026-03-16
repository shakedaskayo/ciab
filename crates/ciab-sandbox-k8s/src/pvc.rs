use k8s_openapi::api::core::v1::{PersistentVolumeClaim, PersistentVolumeClaimSpec};
use k8s_openapi::apimachinery::pkg::api::resource::Quantity;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::{Api, Client};
use std::collections::BTreeMap;
use uuid::Uuid;

use crate::error::K8sError;

pub async fn create_pvc(
    client: &Client,
    namespace: &str,
    sandbox_id: &Uuid,
    size: &str,
    storage_class: Option<&str>,
) -> Result<(), K8sError> {
    let api: Api<PersistentVolumeClaim> = Api::namespaced(client.clone(), namespace);
    let name = pvc_name(sandbox_id);

    // Idempotent: if it already exists, skip
    if api.get_opt(&name).await?.is_some() {
        return Ok(());
    }

    let mut labels = BTreeMap::new();
    labels.insert("ciab/managed-by".to_string(), "ciab".to_string());
    labels.insert("ciab/sandbox-id".to_string(), sandbox_id.to_string());

    let mut requests = BTreeMap::new();
    requests.insert("storage".to_string(), Quantity(size.to_string()));

    let pvc = PersistentVolumeClaim {
        metadata: ObjectMeta {
            name: Some(name),
            namespace: Some(namespace.to_string()),
            labels: Some(labels),
            ..Default::default()
        },
        spec: Some(PersistentVolumeClaimSpec {
            access_modes: Some(vec!["ReadWriteOnce".to_string()]),
            storage_class_name: storage_class.map(|s| s.to_string()),
            resources: Some(k8s_openapi::api::core::v1::VolumeResourceRequirements {
                requests: Some(requests),
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    };

    api.create(&kube::api::PostParams::default(), &pvc).await?;
    Ok(())
}

pub async fn delete_pvc(
    client: &Client,
    namespace: &str,
    sandbox_id: &Uuid,
) -> Result<(), K8sError> {
    let api: Api<PersistentVolumeClaim> = Api::namespaced(client.clone(), namespace);
    let name = pvc_name(sandbox_id);
    match api.delete(&name, &kube::api::DeleteParams::default()).await {
        Ok(_) => Ok(()),
        Err(kube::Error::Api(e)) if e.code == 404 => Ok(()),
        Err(e) => Err(K8sError::Kube(e)),
    }
}

pub fn pvc_name(sandbox_id: &Uuid) -> String {
    format!("ciab-ws-{}", sandbox_id)
}
