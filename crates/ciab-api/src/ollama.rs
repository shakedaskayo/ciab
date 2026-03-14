use serde::{Deserialize, Serialize};

/// Lightweight HTTP client for the Ollama REST API.
pub struct OllamaClient {
    base_url: String,
    client: reqwest::Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaModelInfo {
    pub name: String,
    pub model: String,
    #[serde(default)]
    pub size: u64,
    #[serde(default)]
    pub digest: String,
    #[serde(default)]
    pub details: OllamaModelDetails,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OllamaModelDetails {
    #[serde(default)]
    pub format: String,
    #[serde(default)]
    pub family: String,
    #[serde(default)]
    pub parameter_size: String,
    #[serde(default)]
    pub quantization_level: String,
}

#[derive(Debug, Deserialize)]
struct OllamaTagsResponse {
    #[serde(default)]
    models: Vec<OllamaModelInfo>,
}

#[derive(Debug, Deserialize)]
struct OllamaVersionResponse {
    #[serde(default)]
    version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaModelDetail {
    #[serde(default)]
    pub modelfile: String,
    #[serde(default)]
    pub parameters: String,
    #[serde(default)]
    pub template: String,
    #[serde(default)]
    pub details: OllamaModelDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullProgress {
    pub status: String,
    #[serde(default)]
    pub digest: String,
    #[serde(default)]
    pub total: u64,
    #[serde(default)]
    pub completed: u64,
}

impl OllamaClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(5))
                .build()
                .unwrap_or_default(),
        }
    }

    /// Try to detect a running Ollama instance at the default URL.
    pub async fn detect() -> Option<OllamaClient> {
        let client = OllamaClient::new("http://localhost:11434".to_string());
        match client.version().await {
            Ok(_) => Some(client),
            Err(_) => None,
        }
    }

    /// Get Ollama version.
    pub async fn version(&self) -> Result<String, reqwest::Error> {
        let resp: OllamaVersionResponse = self
            .client
            .get(format!("{}/api/version", self.base_url))
            .send()
            .await?
            .json()
            .await?;
        Ok(resp.version)
    }

    /// List locally available models.
    pub async fn list_models(&self) -> Result<Vec<OllamaModelInfo>, reqwest::Error> {
        let resp: OllamaTagsResponse = self
            .client
            .get(format!("{}/api/tags", self.base_url))
            .send()
            .await?
            .json()
            .await?;
        Ok(resp.models)
    }

    /// Get detailed info about a specific model.
    pub async fn show_model(&self, name: &str) -> Result<OllamaModelDetail, reqwest::Error> {
        let resp: OllamaModelDetail = self
            .client
            .post(format!("{}/api/show", self.base_url))
            .json(&serde_json::json!({ "name": name }))
            .send()
            .await?
            .json()
            .await?;
        Ok(resp)
    }

    /// Pull a model, sending progress events to the channel.
    pub async fn pull_model(
        &self,
        name: &str,
        tx: tokio::sync::mpsc::Sender<PullProgress>,
    ) -> Result<(), String> {
        let pull_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(3600))
            .build()
            .map_err(|e| e.to_string())?;

        let resp = pull_client
            .post(format!("{}/api/pull", self.base_url))
            .json(&serde_json::json!({ "name": name, "stream": true }))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let mut stream = resp.bytes_stream();
        use futures::StreamExt;
        let mut buf = String::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| e.to_string())?;
            buf.push_str(&String::from_utf8_lossy(&chunk));

            // Parse NDJSON lines
            while let Some(newline_pos) = buf.find('\n') {
                let line = buf[..newline_pos].trim().to_string();
                buf = buf[newline_pos + 1..].to_string();

                if line.is_empty() {
                    continue;
                }

                if let Ok(progress) = serde_json::from_str::<PullProgress>(&line) {
                    let _ = tx.send(progress).await;
                }
            }
        }

        Ok(())
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}
