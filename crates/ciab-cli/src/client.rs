use anyhow::{anyhow, Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde_json::Value;

pub struct CiabClient {
    base_url: String,
    client: reqwest::Client,
}

impl CiabClient {
    pub fn new(base_url: &str, api_key: Option<&str>) -> Self {
        let mut default_headers = HeaderMap::new();
        if let Some(key) = api_key {
            if let Ok(val) = HeaderValue::from_str(&format!("Bearer {}", key)) {
                default_headers.insert(AUTHORIZATION, val);
            }
        }

        let client = reqwest::Client::builder()
            .default_headers(default_headers)
            .build()
            .expect("failed to build reqwest client");

        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client,
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    async fn check_response(&self, resp: reqwest::Response) -> Result<Value> {
        let status = resp.status();
        if status.is_success() {
            // Handle 204 No Content
            if status == reqwest::StatusCode::NO_CONTENT {
                return Ok(serde_json::json!({"status": "ok"}));
            }
            let body = resp.text().await.context("reading response body")?;
            if body.is_empty() {
                return Ok(serde_json::json!({"status": "ok"}));
            }
            serde_json::from_str(&body).context("parsing JSON response")
        } else {
            let body = resp
                .text()
                .await
                .unwrap_or_else(|_| "unknown error".to_string());
            Err(anyhow!("API error ({}): {}", status, body))
        }
    }

    // -----------------------------------------------------------------------
    // Health
    // -----------------------------------------------------------------------

    #[allow(dead_code)]
    pub async fn health(&self) -> Result<Value> {
        let resp = self
            .client
            .get(self.url("/health"))
            .send()
            .await
            .context("health request")?;
        self.check_response(resp).await
    }

    // -----------------------------------------------------------------------
    // Sandboxes
    // -----------------------------------------------------------------------

    pub async fn create_sandbox(&self, spec: &Value) -> Result<Value> {
        let resp = self
            .client
            .post(self.url("/api/v1/sandboxes"))
            .json(spec)
            .send()
            .await
            .context("create sandbox")?;
        self.check_response(resp).await
    }

    pub async fn list_sandboxes(
        &self,
        state: Option<&str>,
        provider: Option<&str>,
    ) -> Result<Value> {
        let mut url = self.url("/api/v1/sandboxes");
        let mut params = Vec::new();
        if let Some(s) = state {
            params.push(format!("state={}", s));
        }
        if let Some(p) = provider {
            params.push(format!("provider={}", p));
        }
        if !params.is_empty() {
            url = format!("{}?{}", url, params.join("&"));
        }

        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .context("list sandboxes")?;
        self.check_response(resp).await
    }

    pub async fn get_sandbox(&self, id: &str) -> Result<Value> {
        let resp = self
            .client
            .get(self.url(&format!("/api/v1/sandboxes/{}", id)))
            .send()
            .await
            .context("get sandbox")?;
        self.check_response(resp).await
    }

    pub async fn delete_sandbox(&self, id: &str) -> Result<Value> {
        let resp = self
            .client
            .delete(self.url(&format!("/api/v1/sandboxes/{}", id)))
            .send()
            .await
            .context("delete sandbox")?;
        self.check_response(resp).await
    }

    pub async fn start_sandbox(&self, id: &str) -> Result<Value> {
        let resp = self
            .client
            .post(self.url(&format!("/api/v1/sandboxes/{}/start", id)))
            .send()
            .await
            .context("start sandbox")?;
        self.check_response(resp).await
    }

    pub async fn stop_sandbox(&self, id: &str) -> Result<Value> {
        let resp = self
            .client
            .post(self.url(&format!("/api/v1/sandboxes/{}/stop", id)))
            .send()
            .await
            .context("stop sandbox")?;
        self.check_response(resp).await
    }

    pub async fn pause_sandbox(&self, id: &str) -> Result<Value> {
        let resp = self
            .client
            .post(self.url(&format!("/api/v1/sandboxes/{}/pause", id)))
            .send()
            .await
            .context("pause sandbox")?;
        self.check_response(resp).await
    }

    pub async fn resume_sandbox(&self, id: &str) -> Result<Value> {
        let resp = self
            .client
            .post(self.url(&format!("/api/v1/sandboxes/{}/resume", id)))
            .send()
            .await
            .context("resume sandbox")?;
        self.check_response(resp).await
    }

    pub async fn sandbox_stats(&self, id: &str) -> Result<Value> {
        let resp = self
            .client
            .get(self.url(&format!("/api/v1/sandboxes/{}/stats", id)))
            .send()
            .await
            .context("sandbox stats")?;
        self.check_response(resp).await
    }

    pub async fn sandbox_logs(&self, id: &str, follow: bool, tail: Option<u32>) -> Result<Value> {
        let mut url = format!("/api/v1/sandboxes/{}/logs", id);
        let mut params = Vec::new();
        if follow {
            params.push("follow=true".to_string());
        }
        if let Some(t) = tail {
            params.push(format!("tail={}", t));
        }
        if !params.is_empty() {
            url = format!("{}?{}", url, params.join("&"));
        }

        let resp = self
            .client
            .get(self.url(&url))
            .send()
            .await
            .context("sandbox logs")?;
        self.check_response(resp).await
    }

    pub async fn exec_command(&self, id: &str, req: &Value) -> Result<Value> {
        let resp = self
            .client
            .post(self.url(&format!("/api/v1/sandboxes/{}/exec", id)))
            .json(req)
            .send()
            .await
            .context("exec command")?;
        self.check_response(resp).await
    }

    // -----------------------------------------------------------------------
    // Sessions
    // -----------------------------------------------------------------------

    pub async fn create_session(&self, sandbox_id: &str) -> Result<Value> {
        let resp = self
            .client
            .post(self.url(&format!("/api/v1/sandboxes/{}/sessions", sandbox_id)))
            .json(&serde_json::json!({}))
            .send()
            .await
            .context("create session")?;
        self.check_response(resp).await
    }

    pub async fn list_sessions(&self, sandbox_id: &str) -> Result<Value> {
        let resp = self
            .client
            .get(self.url(&format!("/api/v1/sandboxes/{}/sessions", sandbox_id)))
            .send()
            .await
            .context("list sessions")?;
        self.check_response(resp).await
    }

    pub async fn get_session(&self, id: &str) -> Result<Value> {
        let resp = self
            .client
            .get(self.url(&format!("/api/v1/sessions/{}", id)))
            .send()
            .await
            .context("get session")?;
        self.check_response(resp).await
    }

    pub async fn send_message(&self, session_id: &str, message: &str) -> Result<Value> {
        let resp = self
            .client
            .post(self.url(&format!("/api/v1/sessions/{}/messages", session_id)))
            .json(&serde_json::json!({"message": message}))
            .send()
            .await
            .context("send message")?;
        self.check_response(resp).await
    }

    pub async fn interrupt_session(&self, id: &str) -> Result<Value> {
        let resp = self
            .client
            .post(self.url(&format!("/api/v1/sessions/{}/interrupt", id)))
            .send()
            .await
            .context("interrupt session")?;
        self.check_response(resp).await
    }

    pub async fn stream_session(&self, session_id: &str) -> Result<reqwest::Response> {
        let resp = self
            .client
            .get(self.url(&format!("/api/v1/sessions/{}/stream", session_id)))
            .send()
            .await
            .context("stream session")?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp
                .text()
                .await
                .unwrap_or_else(|_| "unknown error".to_string());
            return Err(anyhow!("API error ({}): {}", status, body));
        }
        Ok(resp)
    }

    // -----------------------------------------------------------------------
    // Files
    // -----------------------------------------------------------------------

    pub async fn list_files(&self, sandbox_id: &str, path: Option<&str>) -> Result<Value> {
        let mut url = format!("/api/v1/sandboxes/{}/files", sandbox_id);
        if let Some(p) = path {
            url = format!("{}?path={}", url, p);
        }
        let resp = self
            .client
            .get(self.url(&url))
            .send()
            .await
            .context("list files")?;
        self.check_response(resp).await
    }

    pub async fn upload_file(
        &self,
        sandbox_id: &str,
        remote_path: &str,
        content: Vec<u8>,
    ) -> Result<Value> {
        let path = remote_path.trim_start_matches('/');
        let resp = self
            .client
            .put(self.url(&format!("/api/v1/sandboxes/{}/files/{}", sandbox_id, path)))
            .body(content)
            .send()
            .await
            .context("upload file")?;
        self.check_response(resp).await
    }

    pub async fn download_file(&self, sandbox_id: &str, remote_path: &str) -> Result<Vec<u8>> {
        let path = remote_path.trim_start_matches('/');
        let resp = self
            .client
            .get(self.url(&format!("/api/v1/sandboxes/{}/files/{}", sandbox_id, path)))
            .send()
            .await
            .context("download file")?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp
                .text()
                .await
                .unwrap_or_else(|_| "unknown error".to_string());
            return Err(anyhow!("API error ({}): {}", status, body));
        }
        let bytes = resp.bytes().await.context("reading file bytes")?;
        Ok(bytes.to_vec())
    }

    pub async fn delete_file(&self, sandbox_id: &str, remote_path: &str) -> Result<Value> {
        let path = remote_path.trim_start_matches('/');
        let resp = self
            .client
            .delete(self.url(&format!("/api/v1/sandboxes/{}/files/{}", sandbox_id, path)))
            .send()
            .await
            .context("delete file")?;
        self.check_response(resp).await
    }

    // -----------------------------------------------------------------------
    // Credentials
    // -----------------------------------------------------------------------

    pub async fn create_credential(
        &self,
        name: &str,
        cred_type: &str,
        value: &str,
    ) -> Result<Value> {
        let resp = self
            .client
            .post(self.url("/api/v1/credentials"))
            .json(&serde_json::json!({
                "name": name,
                "credential_type": cred_type,
                "value": value,
            }))
            .send()
            .await
            .context("create credential")?;
        self.check_response(resp).await
    }

    pub async fn list_credentials(&self) -> Result<Value> {
        let resp = self
            .client
            .get(self.url("/api/v1/credentials"))
            .send()
            .await
            .context("list credentials")?;
        self.check_response(resp).await
    }

    pub async fn get_credential(&self, id: &str) -> Result<Value> {
        let resp = self
            .client
            .get(self.url(&format!("/api/v1/credentials/{}", id)))
            .send()
            .await
            .context("get credential")?;
        self.check_response(resp).await
    }

    pub async fn delete_credential(&self, id: &str) -> Result<Value> {
        let resp = self
            .client
            .delete(self.url(&format!("/api/v1/credentials/{}", id)))
            .send()
            .await
            .context("delete credential")?;
        self.check_response(resp).await
    }

    // -----------------------------------------------------------------------
    // OAuth
    // -----------------------------------------------------------------------

    pub async fn oauth_authorize(&self, provider: &str) -> Result<Value> {
        let resp = self
            .client
            .get(self.url(&format!("/api/v1/oauth/{}/authorize", provider)))
            .send()
            .await
            .context("oauth authorize")?;
        self.check_response(resp).await
    }

    pub async fn oauth_device_code(&self, provider: &str) -> Result<Value> {
        let resp = self
            .client
            .get(self.url(&format!("/api/v1/oauth/{}/device-code", provider)))
            .send()
            .await
            .context("oauth device code")?;
        self.check_response(resp).await
    }

    pub async fn oauth_device_poll(&self, provider: &str, device_code: &str) -> Result<Value> {
        let resp = self
            .client
            .post(self.url(&format!("/api/v1/oauth/{}/device-poll", provider)))
            .json(&serde_json::json!({"device_code": device_code}))
            .send()
            .await
            .context("oauth device poll")?;
        self.check_response(resp).await
    }

    pub async fn oauth_refresh(&self, provider: &str) -> Result<Value> {
        let resp = self
            .client
            .post(self.url(&format!("/api/v1/oauth/{}/refresh", provider)))
            .send()
            .await
            .context("oauth refresh")?;
        self.check_response(resp).await
    }

    // -----------------------------------------------------------------------
    // Workspaces
    // -----------------------------------------------------------------------

    pub async fn create_workspace(&self, body: &Value) -> Result<Value> {
        let resp = self
            .client
            .post(self.url("/api/v1/workspaces"))
            .json(body)
            .send()
            .await
            .context("create workspace")?;
        self.check_response(resp).await
    }

    pub async fn list_workspaces(&self, name: Option<&str>) -> Result<Value> {
        let mut url = self.url("/api/v1/workspaces");
        if let Some(n) = name {
            url = format!("{}?name={}", url, n);
        }
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .context("list workspaces")?;
        self.check_response(resp).await
    }

    pub async fn get_workspace(&self, id: &str) -> Result<Value> {
        let resp = self
            .client
            .get(self.url(&format!("/api/v1/workspaces/{}", id)))
            .send()
            .await
            .context("get workspace")?;
        self.check_response(resp).await
    }

    pub async fn update_workspace(&self, id: &str, body: &Value) -> Result<Value> {
        let resp = self
            .client
            .put(self.url(&format!("/api/v1/workspaces/{}", id)))
            .json(body)
            .send()
            .await
            .context("update workspace")?;
        self.check_response(resp).await
    }

    pub async fn delete_workspace(&self, id: &str) -> Result<Value> {
        let resp = self
            .client
            .delete(self.url(&format!("/api/v1/workspaces/{}", id)))
            .send()
            .await
            .context("delete workspace")?;
        self.check_response(resp).await
    }

    pub async fn launch_workspace(&self, id: &str) -> Result<Value> {
        let resp = self
            .client
            .post(self.url(&format!("/api/v1/workspaces/{}/launch", id)))
            .send()
            .await
            .context("launch workspace")?;
        self.check_response(resp).await
    }

    pub async fn list_workspace_sandboxes(&self, id: &str) -> Result<Value> {
        let resp = self
            .client
            .get(self.url(&format!("/api/v1/workspaces/{}/sandboxes", id)))
            .send()
            .await
            .context("list workspace sandboxes")?;
        self.check_response(resp).await
    }

    pub async fn import_workspace_toml(&self, toml_content: &str) -> Result<Value> {
        let resp = self
            .client
            .post(self.url("/api/v1/workspaces/import"))
            .header("content-type", "text/plain")
            .body(toml_content.to_string())
            .send()
            .await
            .context("import workspace toml")?;
        self.check_response(resp).await
    }

    // -----------------------------------------------------------------------
    // Gateway
    // -----------------------------------------------------------------------

    pub async fn gateway_status(&self) -> Result<Value> {
        let resp = self
            .client
            .get(self.url("/api/v1/gateway/status"))
            .send()
            .await
            .context("gateway status")?;
        self.check_response(resp).await
    }

    pub async fn gateway_discover(&self) -> Result<Value> {
        let resp = self
            .client
            .get(self.url("/api/v1/gateway/discover"))
            .send()
            .await
            .context("gateway discover")?;
        self.check_response(resp).await
    }

    pub async fn gateway_expose(&self, body: &Value) -> Result<Value> {
        let resp = self
            .client
            .post(self.url("/api/v1/gateway/expose"))
            .json(body)
            .send()
            .await
            .context("gateway expose")?;
        self.check_response(resp).await
    }

    pub async fn gateway_create_token(&self, body: &Value) -> Result<Value> {
        let resp = self
            .client
            .post(self.url("/api/v1/gateway/tokens"))
            .json(body)
            .send()
            .await
            .context("gateway create token")?;
        self.check_response(resp).await
    }

    pub async fn gateway_list_tokens(&self) -> Result<Value> {
        let resp = self
            .client
            .get(self.url("/api/v1/gateway/tokens"))
            .send()
            .await
            .context("gateway list tokens")?;
        self.check_response(resp).await
    }

    pub async fn gateway_revoke_token(&self, id: &str) -> Result<Value> {
        let resp = self
            .client
            .delete(self.url(&format!("/api/v1/gateway/tokens/{}", id)))
            .send()
            .await
            .context("gateway revoke token")?;
        self.check_response(resp).await
    }

    pub async fn gateway_create_tunnel(&self, body: &Value) -> Result<Value> {
        let resp = self
            .client
            .post(self.url("/api/v1/gateway/tunnels"))
            .json(body)
            .send()
            .await
            .context("gateway create tunnel")?;
        self.check_response(resp).await
    }

    pub async fn gateway_list_tunnels(&self) -> Result<Value> {
        let resp = self
            .client
            .get(self.url("/api/v1/gateway/tunnels"))
            .send()
            .await
            .context("gateway list tunnels")?;
        self.check_response(resp).await
    }

    pub async fn gateway_delete_tunnel(&self, id: &str) -> Result<Value> {
        let resp = self
            .client
            .delete(self.url(&format!("/api/v1/gateway/tunnels/{}", id)))
            .send()
            .await
            .context("gateway delete tunnel")?;
        self.check_response(resp).await
    }

    pub async fn gateway_prepare_provider(&self, body: &Value) -> Result<Value> {
        let resp = self
            .client
            .post(self.url("/api/v1/gateway/providers/prepare"))
            .json(body)
            .send()
            .await
            .context("gateway prepare provider")?;
        self.check_response(resp).await
    }

    // -----------------------------------------------------------------------
    // Images
    // -----------------------------------------------------------------------

    pub async fn build_image(&self, body: &Value) -> Result<Value> {
        let resp = self
            .client
            .post(self.url("/api/v1/images/build"))
            .json(body)
            .send()
            .await
            .context("build image")?;
        self.check_response(resp).await
    }

    pub async fn list_images(&self) -> Result<Value> {
        let resp = self
            .client
            .get(self.url("/api/v1/images"))
            .send()
            .await
            .context("list images")?;
        self.check_response(resp).await
    }

    pub async fn get_build_status(&self, build_id: &str) -> Result<Value> {
        let resp = self
            .client
            .get(self.url(&format!("/api/v1/images/builds/{}", build_id)))
            .send()
            .await
            .context("get build status")?;
        self.check_response(resp).await
    }

    pub async fn delete_image(&self, image_id: &str) -> Result<Value> {
        let resp = self
            .client
            .delete(self.url(&format!("/api/v1/images/{}", image_id)))
            .send()
            .await
            .context("delete image")?;
        self.check_response(resp).await
    }

    pub async fn export_workspace_toml(&self, id: &str) -> Result<String> {
        let resp = self
            .client
            .get(self.url(&format!("/api/v1/workspaces/{}/export", id)))
            .send()
            .await
            .context("export workspace toml")?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp
                .text()
                .await
                .unwrap_or_else(|_| "unknown error".to_string());
            return Err(anyhow!("API error ({}): {}", status, body));
        }
        resp.text().await.context("reading toml body")
    }
}
