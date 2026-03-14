use bytes::Bytes;
use futures::StreamExt;
use reqwest::Client;
use tokio::sync::mpsc;
use uuid::Uuid;

use ciab_core::error::{CiabError, CiabResult};
use ciab_core::types::sandbox::{ExecRequest, ExecResult, FileInfo, ResourceStats};
use ciab_core::types::stream::{StreamEvent, StreamEventType};

#[derive(Clone)]
pub struct ExecdClient {
    endpoint_url: String,
    client: Client,
}

impl ExecdClient {
    pub fn new(endpoint_url: String) -> Self {
        Self {
            endpoint_url,
            client: Client::new(),
        }
    }

    pub async fn run_command(&self, req: &ExecRequest) -> CiabResult<ExecResult> {
        let url = format!("{}/exec", self.endpoint_url);
        let resp = self
            .client
            .post(&url)
            .json(req)
            .send()
            .await
            .map_err(|e| CiabError::OpenSandboxError(e.to_string()))?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(CiabError::ExecFailed(format!("({}): {}", status, body)));
        }

        resp.json::<ExecResult>()
            .await
            .map_err(|e| CiabError::OpenSandboxError(e.to_string()))
    }

    pub async fn run_command_stream(
        &self,
        req: &ExecRequest,
        tx: mpsc::Sender<StreamEvent>,
        sandbox_id: Uuid,
    ) -> CiabResult<i32> {
        let url = format!("{}/exec/stream", self.endpoint_url);
        let resp = self
            .client
            .post(&url)
            .json(req)
            .send()
            .await
            .map_err(|e| CiabError::OpenSandboxError(e.to_string()))?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(CiabError::ExecFailed(format!("({}): {}", status, body)));
        }

        let mut stream = resp.bytes_stream();
        let mut buffer = String::new();
        let mut exit_code: i32 = -1;

        while let Some(chunk_result) = stream.next().await {
            let chunk: Bytes =
                chunk_result.map_err(|e| CiabError::OpenSandboxError(e.to_string()))?;
            buffer.push_str(&String::from_utf8_lossy(&chunk));

            while let Some(pos) = buffer.find("\n\n") {
                let event_block = buffer[..pos].to_string();
                buffer = buffer[pos + 2..].to_string();

                let mut event_type_str: Option<String> = None;
                let mut data_str: Option<String> = None;

                for line in event_block.lines() {
                    if let Some(rest) = line.strip_prefix("event: ") {
                        event_type_str = Some(rest.trim().to_string());
                    } else if let Some(rest) = line.strip_prefix("data: ") {
                        data_str = Some(rest.trim().to_string());
                    }
                }

                if let (Some(evt_type), Some(data_raw)) = (event_type_str, data_str) {
                    let data: serde_json::Value =
                        serde_json::from_str(&data_raw).unwrap_or(serde_json::json!(data_raw));

                    let stream_event_type = match evt_type.as_str() {
                        "text_delta" => StreamEventType::TextDelta,
                        "text_complete" => StreamEventType::TextComplete,
                        "error" => StreamEventType::Error,
                        "stats" => StreamEventType::Stats,
                        "log_line" => StreamEventType::LogLine,
                        _ => StreamEventType::TextDelta,
                    };

                    // Extract exit code if present
                    if let Some(code) = data.get("exit_code").and_then(|v| v.as_i64()) {
                        exit_code = code as i32;
                    }

                    let event = StreamEvent {
                        id: uuid::Uuid::new_v4().to_string(),
                        sandbox_id,
                        session_id: None,
                        event_type: stream_event_type,
                        data,
                        timestamp: chrono::Utc::now(),
                    };

                    if tx.send(event).await.is_err() {
                        // Receiver dropped
                        break;
                    }
                }
            }
        }

        Ok(exit_code)
    }

    pub async fn upload_file(&self, path: &str, content: &[u8], mode: u32) -> CiabResult<()> {
        let url = format!("{}/files/{}", self.endpoint_url, path);
        let resp = self
            .client
            .put(&url)
            .header("X-File-Mode", mode.to_string())
            .body(content.to_vec())
            .send()
            .await
            .map_err(|e| CiabError::OpenSandboxError(e.to_string()))?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(CiabError::OpenSandboxError(format!(
                "upload file failed ({}): {}",
                status, body
            )));
        }

        Ok(())
    }

    pub async fn download_file(&self, path: &str) -> CiabResult<Vec<u8>> {
        let url = format!("{}/files/{}", self.endpoint_url, path);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| CiabError::OpenSandboxError(e.to_string()))?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(CiabError::FileNotFound(format!("({}): {}", status, body)));
        }

        resp.bytes()
            .await
            .map(|b| b.to_vec())
            .map_err(|e| CiabError::OpenSandboxError(e.to_string()))
    }

    pub async fn list_files(&self, pattern: &str) -> CiabResult<Vec<FileInfo>> {
        let url = format!("{}/files", self.endpoint_url);
        let resp = self
            .client
            .get(&url)
            .query(&[("pattern", pattern)])
            .send()
            .await
            .map_err(|e| CiabError::OpenSandboxError(e.to_string()))?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(CiabError::OpenSandboxError(format!(
                "list files failed ({}): {}",
                status, body
            )));
        }

        resp.json::<Vec<FileInfo>>()
            .await
            .map_err(|e| CiabError::OpenSandboxError(e.to_string()))
    }

    pub async fn get_metrics(&self) -> CiabResult<ResourceStats> {
        let url = format!("{}/metrics", self.endpoint_url);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| CiabError::OpenSandboxError(e.to_string()))?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(CiabError::OpenSandboxError(format!(
                "get metrics failed ({}): {}",
                status, body
            )));
        }

        resp.json::<ResourceStats>()
            .await
            .map_err(|e| CiabError::OpenSandboxError(e.to_string()))
    }
}
