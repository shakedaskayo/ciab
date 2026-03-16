use futures::{AsyncBufReadExt, StreamExt};
use k8s_openapi::api::core::v1::Pod;
use kube::{Api, Client};
use tokio::sync::mpsc;

use crate::error::K8sError;

pub async fn stream_logs(
    client: &Client,
    namespace: &str,
    pod_name: &str,
) -> Result<mpsc::Receiver<String>, K8sError> {
    let api: Api<Pod> = Api::namespaced(client.clone(), namespace);
    let (tx, rx) = mpsc::channel::<String>(256);

    let params = kube::api::LogParams {
        follow: true,
        tail_lines: None,
        ..Default::default()
    };

    let log_stream = api.log_stream(pod_name, &params).await?;
    let pod_name_owned = pod_name.to_string();

    tokio::spawn(async move {
        let mut lines = log_stream.lines();
        loop {
            match lines.next().await {
                Some(Ok(line)) => {
                    if tx.send(line).await.is_err() {
                        return;
                    }
                }
                Some(Err(e)) => {
                    tracing::warn!(pod = %pod_name_owned, error = %e, "log stream error");
                    return;
                }
                None => return,
            }
        }
    });

    Ok(rx)
}
