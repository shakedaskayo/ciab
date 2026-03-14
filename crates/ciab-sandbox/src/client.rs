use std::collections::HashMap;

use reqwest::Client;
use serde::{Deserialize, Serialize};

use ciab_core::error::{CiabError, CiabResult};
use ciab_core::types::sandbox::PortMapping;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSandboxRequest {
    pub image: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpu: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory_mb: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disk_mb: Option<u32>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ports: Vec<PortMapping>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_secs: Option<u64>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub labels: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenSandboxResponse {
    pub id: String,
    pub status: String,
    #[serde(default)]
    pub endpoint_url: Option<String>,
    pub created_at: String,
    #[serde(default)]
    pub labels: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize)]
struct RenewRequest {
    duration_secs: u64,
}

#[derive(Clone)]
pub struct OpenSandboxClient {
    base_url: String,
    api_key: Option<String>,
    client: Client,
}

impl OpenSandboxClient {
    pub fn new(base_url: String, api_key: Option<String>) -> Self {
        Self {
            base_url,
            api_key,
            client: Client::new(),
        }
    }

    fn request(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        let url = format!("{}{}", self.base_url, path);
        let mut builder = self.client.request(method, &url);
        if let Some(ref key) = self.api_key {
            builder = builder.header("Authorization", format!("Bearer {}", key));
        }
        builder
    }

    pub async fn create_sandbox(
        &self,
        request: &CreateSandboxRequest,
    ) -> CiabResult<OpenSandboxResponse> {
        let resp = self
            .request(reqwest::Method::POST, "/api/v1/sandboxes")
            .json(request)
            .send()
            .await
            .map_err(|e| CiabError::OpenSandboxError(e.to_string()))?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(CiabError::OpenSandboxError(format!(
                "create sandbox failed ({}): {}",
                status, body
            )));
        }

        resp.json::<OpenSandboxResponse>()
            .await
            .map_err(|e| CiabError::OpenSandboxError(e.to_string()))
    }

    pub async fn get_sandbox(&self, sandbox_id: &str) -> CiabResult<OpenSandboxResponse> {
        let path = format!("/api/v1/sandboxes/{}", sandbox_id);
        let resp = self
            .request(reqwest::Method::GET, &path)
            .send()
            .await
            .map_err(|e| CiabError::OpenSandboxError(e.to_string()))?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(CiabError::OpenSandboxError(format!(
                "get sandbox failed ({}): {}",
                status, body
            )));
        }

        resp.json::<OpenSandboxResponse>()
            .await
            .map_err(|e| CiabError::OpenSandboxError(e.to_string()))
    }

    pub async fn list_sandboxes(&self) -> CiabResult<Vec<OpenSandboxResponse>> {
        let resp = self
            .request(reqwest::Method::GET, "/api/v1/sandboxes")
            .send()
            .await
            .map_err(|e| CiabError::OpenSandboxError(e.to_string()))?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(CiabError::OpenSandboxError(format!(
                "list sandboxes failed ({}): {}",
                status, body
            )));
        }

        resp.json::<Vec<OpenSandboxResponse>>()
            .await
            .map_err(|e| CiabError::OpenSandboxError(e.to_string()))
    }

    pub async fn delete_sandbox(&self, sandbox_id: &str) -> CiabResult<()> {
        let path = format!("/api/v1/sandboxes/{}", sandbox_id);
        let resp = self
            .request(reqwest::Method::DELETE, &path)
            .send()
            .await
            .map_err(|e| CiabError::OpenSandboxError(e.to_string()))?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(CiabError::OpenSandboxError(format!(
                "delete sandbox failed ({}): {}",
                status, body
            )));
        }

        Ok(())
    }

    pub async fn pause_sandbox(&self, sandbox_id: &str) -> CiabResult<()> {
        let path = format!("/api/v1/sandboxes/{}/pause", sandbox_id);
        let resp = self
            .request(reqwest::Method::POST, &path)
            .send()
            .await
            .map_err(|e| CiabError::OpenSandboxError(e.to_string()))?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(CiabError::OpenSandboxError(format!(
                "pause sandbox failed ({}): {}",
                status, body
            )));
        }

        Ok(())
    }

    pub async fn resume_sandbox(&self, sandbox_id: &str) -> CiabResult<()> {
        let path = format!("/api/v1/sandboxes/{}/resume", sandbox_id);
        let resp = self
            .request(reqwest::Method::POST, &path)
            .send()
            .await
            .map_err(|e| CiabError::OpenSandboxError(e.to_string()))?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(CiabError::OpenSandboxError(format!(
                "resume sandbox failed ({}): {}",
                status, body
            )));
        }

        Ok(())
    }

    pub async fn renew_expiration(&self, sandbox_id: &str, duration_secs: u64) -> CiabResult<()> {
        let path = format!("/api/v1/sandboxes/{}/renew", sandbox_id);
        let resp = self
            .request(reqwest::Method::POST, &path)
            .json(&RenewRequest { duration_secs })
            .send()
            .await
            .map_err(|e| CiabError::OpenSandboxError(e.to_string()))?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(CiabError::OpenSandboxError(format!(
                "renew sandbox failed ({}): {}",
                status, body
            )));
        }

        Ok(())
    }
}
