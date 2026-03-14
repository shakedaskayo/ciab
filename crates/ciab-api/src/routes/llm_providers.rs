use std::collections::HashMap;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use chrono::Utc;
use ciab_core::error::CiabError;
use ciab_core::types::llm_provider::{LlmModel, LlmProvider, LlmProviderKind};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ollama::OllamaClient;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Request / Response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateLlmProviderRequest {
    pub name: String,
    pub kind: LlmProviderKind,
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub default_model: Option<String>,
    #[serde(default)]
    pub is_local: Option<bool>,
    #[serde(default)]
    pub extra: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateLlmProviderRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub kind: Option<LlmProviderKind>,
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub base_url: Option<Option<String>>,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub default_model: Option<Option<String>>,
    #[serde(default)]
    pub is_local: Option<bool>,
    #[serde(default)]
    pub extra: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Serialize)]
pub struct TestResult {
    pub success: bool,
    pub message: String,
    pub latency_ms: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct DetectResult {
    pub detected: Vec<DetectedProvider>,
}

#[derive(Debug, Serialize)]
pub struct DetectedProvider {
    pub kind: LlmProviderKind,
    pub name: String,
    pub base_url: String,
    pub version: Option<String>,
    pub already_registered: bool,
}

#[derive(Debug, Deserialize)]
pub struct PullModelRequest {
    pub model: String,
    #[serde(default)]
    pub base_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CompatibilityEntry {
    pub agent_provider: String,
    pub llm_provider_kind: LlmProviderKind,
    pub supports_model_override: bool,
    pub notes: Option<String>,
}

// ---------------------------------------------------------------------------
// list_llm_providers
// ---------------------------------------------------------------------------

pub async fn list_llm_providers(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, CiabError> {
    let providers = state.db.list_llm_providers().await?;
    Ok(Json(providers))
}

// ---------------------------------------------------------------------------
// create_llm_provider
// ---------------------------------------------------------------------------

pub async fn create_llm_provider(
    State(state): State<AppState>,
    Json(body): Json<CreateLlmProviderRequest>,
) -> Result<impl IntoResponse, CiabError> {
    let mut api_key_credential_id = None;

    // If an API key is provided, store it as a credential
    if let Some(ref api_key) = body.api_key {
        let cred = state
            .credentials
            .store_credential(
                &format!("llm-{}-key", body.name),
                ciab_core::types::credentials::CredentialType::ApiKey,
                api_key.as_bytes(),
                HashMap::new(),
                None,
            )
            .await?;
        api_key_credential_id = Some(cred.id);
    }

    let is_local = body.is_local.unwrap_or(body.kind == LlmProviderKind::Ollama);
    let now = Utc::now();
    let provider = LlmProvider {
        id: Uuid::new_v4(),
        name: body.name,
        kind: body.kind,
        enabled: body.enabled.unwrap_or(true),
        base_url: body.base_url,
        api_key_credential_id,
        default_model: body.default_model,
        is_local,
        auto_detected: false,
        extra: body.extra.unwrap_or_default(),
        created_at: now,
        updated_at: now,
    };

    state.db.insert_llm_provider(&provider).await?;
    Ok((StatusCode::CREATED, Json(provider)))
}

// ---------------------------------------------------------------------------
// get_llm_provider
// ---------------------------------------------------------------------------

pub async fn get_llm_provider(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, CiabError> {
    let provider = state
        .db
        .get_llm_provider(&id)
        .await?
        .ok_or_else(|| CiabError::Internal(format!("LLM provider {} not found", id)))?;
    Ok(Json(provider))
}

// ---------------------------------------------------------------------------
// update_llm_provider
// ---------------------------------------------------------------------------

pub async fn update_llm_provider(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateLlmProviderRequest>,
) -> Result<impl IntoResponse, CiabError> {
    let mut provider = state
        .db
        .get_llm_provider(&id)
        .await?
        .ok_or_else(|| CiabError::Internal(format!("LLM provider {} not found", id)))?;

    if let Some(name) = body.name {
        provider.name = name;
    }
    if let Some(kind) = body.kind {
        provider.kind = kind;
    }
    if let Some(enabled) = body.enabled {
        provider.enabled = enabled;
    }
    if let Some(base_url) = body.base_url {
        provider.base_url = base_url;
    }
    if let Some(default_model) = body.default_model {
        provider.default_model = default_model;
    }
    if let Some(is_local) = body.is_local {
        provider.is_local = is_local;
    }
    if let Some(extra) = body.extra {
        provider.extra = extra;
    }

    // If a new API key is provided, update the credential
    if let Some(ref api_key) = body.api_key {
        // Delete old credential if exists
        if let Some(old_cred_id) = provider.api_key_credential_id {
            let _ = state.credentials.delete_credential(&old_cred_id).await;
        }
        let cred = state
            .credentials
            .store_credential(
                &format!("llm-{}-key", provider.name),
                ciab_core::types::credentials::CredentialType::ApiKey,
                api_key.as_bytes(),
                HashMap::new(),
                None,
            )
            .await?;
        provider.api_key_credential_id = Some(cred.id);
    }

    provider.updated_at = Utc::now();
    state.db.update_llm_provider(&provider).await?;
    Ok(Json(provider))
}

// ---------------------------------------------------------------------------
// delete_llm_provider
// ---------------------------------------------------------------------------

pub async fn delete_llm_provider(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, CiabError> {
    // Delete associated credential if exists
    if let Some(provider) = state.db.get_llm_provider(&id).await? {
        if let Some(cred_id) = provider.api_key_credential_id {
            let _ = state.credentials.delete_credential(&cred_id).await;
        }
    }
    state.db.delete_llm_provider(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// list_models
// ---------------------------------------------------------------------------

pub async fn list_models(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, CiabError> {
    let models = state.db.list_llm_models(&id).await?;
    Ok(Json(models))
}

// ---------------------------------------------------------------------------
// refresh_models
// ---------------------------------------------------------------------------

pub async fn refresh_models(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, CiabError> {
    let provider = state
        .db
        .get_llm_provider(&id)
        .await?
        .ok_or_else(|| CiabError::Internal(format!("LLM provider {} not found", id)))?;

    let models = fetch_models_for_provider(&state, &provider).await?;

    // Clear old models and insert new ones
    state.db.delete_llm_models_by_provider(&id).await?;
    state.db.insert_llm_models(&id, &models).await?;

    Ok(Json(models))
}

// ---------------------------------------------------------------------------
// test_provider
// ---------------------------------------------------------------------------

pub async fn test_provider(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, CiabError> {
    let provider = state
        .db
        .get_llm_provider(&id)
        .await?
        .ok_or_else(|| CiabError::Internal(format!("LLM provider {} not found", id)))?;

    let start = std::time::Instant::now();
    let result = test_provider_connectivity(&state, &provider).await;
    let latency = start.elapsed().as_millis() as u64;

    match result {
        Ok(msg) => Ok(Json(TestResult {
            success: true,
            message: msg,
            latency_ms: Some(latency),
        })),
        Err(e) => Ok(Json(TestResult {
            success: false,
            message: e,
            latency_ms: Some(latency),
        })),
    }
}

// ---------------------------------------------------------------------------
// detect_providers
// ---------------------------------------------------------------------------

pub async fn detect_providers(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, CiabError> {
    let mut detected = Vec::new();

    // Check Ollama
    if let Some(client) = OllamaClient::detect().await {
        let version = client.version().await.ok();
        let existing = state.db.list_llm_providers().await?;
        let already_registered = existing
            .iter()
            .any(|p| p.kind == LlmProviderKind::Ollama);

        detected.push(DetectedProvider {
            kind: LlmProviderKind::Ollama,
            name: "Ollama (local)".to_string(),
            base_url: client.base_url().to_string(),
            version,
            already_registered,
        });
    }

    Ok(Json(DetectResult { detected }))
}

// ---------------------------------------------------------------------------
// ollama_pull
// ---------------------------------------------------------------------------

pub async fn ollama_pull(
    State(_state): State<AppState>,
    Json(body): Json<PullModelRequest>,
) -> Result<impl IntoResponse, CiabError> {
    let base_url = body
        .base_url
        .unwrap_or_else(|| "http://localhost:11434".to_string());
    let client = OllamaClient::new(base_url);

    let (tx, mut rx) = tokio::sync::mpsc::channel(100);
    let model_name = body.model.clone();

    // Spawn pull in background
    tokio::spawn(async move {
        if let Err(e) = client.pull_model(&model_name, tx).await {
            tracing::error!(model = %model_name, error = %e, "ollama pull failed");
        }
    });

    // Collect final status
    let mut last_status = "started".to_string();
    while let Some(progress) = rx.recv().await {
        last_status = progress.status;
    }

    Ok(Json(serde_json::json!({
        "model": body.model,
        "status": last_status,
    })))
}

// ---------------------------------------------------------------------------
// compatibility
// ---------------------------------------------------------------------------

pub async fn compatibility(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, CiabError> {
    let mut entries = Vec::new();

    for agent in state.agents.values() {
        for compat in agent.supported_llm_providers() {
            entries.push(CompatibilityEntry {
                agent_provider: compat.agent_provider,
                llm_provider_kind: compat.llm_provider_kind,
                supports_model_override: compat.supports_model_override,
                notes: compat.notes,
            });
        }
    }

    Ok(Json(entries))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

async fn fetch_models_for_provider(
    state: &AppState,
    provider: &LlmProvider,
) -> CiabResult<Vec<LlmModel>> {
    match provider.kind {
        LlmProviderKind::Ollama => {
            let base_url = provider
                .base_url
                .clone()
                .unwrap_or_else(|| "http://localhost:11434".to_string());
            let client = OllamaClient::new(base_url);
            let ollama_models = client
                .list_models()
                .await
                .map_err(|e| CiabError::Internal(format!("Ollama API error: {}", e)))?;

            Ok(ollama_models
                .into_iter()
                .map(|m| LlmModel {
                    id: m.name.clone(),
                    name: m.name,
                    provider_id: provider.id,
                    context_window: None,
                    supports_tools: false,
                    supports_vision: false,
                    is_local: true,
                    size_bytes: Some(m.size),
                    family: if m.details.family.is_empty() {
                        None
                    } else {
                        Some(m.details.family)
                    },
                })
                .collect())
        }
        LlmProviderKind::Anthropic => Ok(hardcoded_anthropic_models(provider.id)),
        LlmProviderKind::Google => Ok(hardcoded_google_models(provider.id)),
        LlmProviderKind::OpenAi => {
            // Try fetching from API; fall back to hardcoded
            match fetch_openai_models(state, provider).await {
                Ok(models) => Ok(models),
                Err(_) => Ok(hardcoded_openai_models(provider.id)),
            }
        }
        LlmProviderKind::OpenRouter => {
            match fetch_openrouter_models(provider).await {
                Ok(models) => Ok(models),
                Err(_) => Ok(vec![]),
            }
        }
        LlmProviderKind::Custom => Ok(vec![]),
    }
}

use ciab_core::error::CiabResult;

async fn fetch_openai_models(
    state: &AppState,
    provider: &LlmProvider,
) -> Result<Vec<LlmModel>, String> {
    let api_key = resolve_api_key(state, provider).await?;
    let base_url = provider
        .base_url
        .clone()
        .unwrap_or_else(|| "https://api.openai.com".to_string());

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}/v1/models", base_url))
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let data = json
        .get("data")
        .and_then(|d| d.as_array())
        .cloned()
        .unwrap_or_default();

    Ok(data
        .into_iter()
        .filter_map(|m| {
            let id = m.get("id")?.as_str()?.to_string();
            Some(LlmModel {
                id: id.clone(),
                name: id,
                provider_id: provider.id,
                context_window: None,
                supports_tools: true,
                supports_vision: false,
                is_local: false,
                size_bytes: None,
                family: None,
            })
        })
        .collect())
}

async fn fetch_openrouter_models(provider: &LlmProvider) -> Result<Vec<LlmModel>, String> {
    let client = reqwest::Client::new();
    let resp = client
        .get("https://openrouter.ai/api/v1/models")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let data = json
        .get("data")
        .and_then(|d| d.as_array())
        .cloned()
        .unwrap_or_default();

    Ok(data
        .into_iter()
        .filter_map(|m| {
            let id = m.get("id")?.as_str()?.to_string();
            let name = m
                .get("name")
                .and_then(|n| n.as_str())
                .unwrap_or(&id)
                .to_string();
            let context_length = m
                .get("context_length")
                .and_then(|c| c.as_u64());

            Some(LlmModel {
                id: id.clone(),
                name,
                provider_id: provider.id,
                context_window: context_length,
                supports_tools: true,
                supports_vision: false,
                is_local: false,
                size_bytes: None,
                family: None,
            })
        })
        .collect())
}

async fn resolve_api_key(state: &AppState, provider: &LlmProvider) -> Result<String, String> {
    if let Some(cred_id) = provider.api_key_credential_id {
        let (_cred, data) = state
            .credentials
            .get_credential(&cred_id)
            .await
            .map_err(|e| e.to_string())?;
        Ok(String::from_utf8_lossy(&data).to_string())
    } else {
        Err("No API key configured for this provider".to_string())
    }
}

async fn test_provider_connectivity(
    state: &AppState,
    provider: &LlmProvider,
) -> Result<String, String> {
    match provider.kind {
        LlmProviderKind::Ollama => {
            let base_url = provider
                .base_url
                .clone()
                .unwrap_or_else(|| "http://localhost:11434".to_string());
            let client = OllamaClient::new(base_url);
            let version = client.version().await.map_err(|e| e.to_string())?;
            Ok(format!("Ollama v{} is running", version))
        }
        LlmProviderKind::Anthropic => {
            let api_key = resolve_api_key(state, provider).await?;
            let base_url = provider
                .base_url
                .clone()
                .unwrap_or_else(|| "https://api.anthropic.com".to_string());
            let client = reqwest::Client::new();
            let resp = client
                .get(format!("{}/v1/models", base_url))
                .header("x-api-key", &api_key)
                .header("anthropic-version", "2023-06-01")
                .send()
                .await
                .map_err(|e| e.to_string())?;
            if resp.status().is_success() {
                Ok("Anthropic API connected".to_string())
            } else {
                Err(format!("Anthropic API returned {}", resp.status()))
            }
        }
        LlmProviderKind::OpenAi | LlmProviderKind::OpenRouter => {
            let api_key = resolve_api_key(state, provider).await?;
            let base_url = provider.base_url.clone().unwrap_or_else(|| {
                if provider.kind == LlmProviderKind::OpenRouter {
                    "https://openrouter.ai/api".to_string()
                } else {
                    "https://api.openai.com".to_string()
                }
            });
            let client = reqwest::Client::new();
            let resp = client
                .get(format!("{}/v1/models", base_url))
                .header("Authorization", format!("Bearer {}", api_key))
                .send()
                .await
                .map_err(|e| e.to_string())?;
            if resp.status().is_success() {
                Ok(format!("{} API connected", provider.kind))
            } else {
                Err(format!("API returned {}", resp.status()))
            }
        }
        LlmProviderKind::Google => {
            let api_key = resolve_api_key(state, provider).await?;
            let client = reqwest::Client::new();
            let resp = client
                .get(format!(
                    "https://generativelanguage.googleapis.com/v1beta/models?key={}",
                    api_key
                ))
                .send()
                .await
                .map_err(|e| e.to_string())?;
            if resp.status().is_success() {
                Ok("Google AI API connected".to_string())
            } else {
                Err(format!("Google API returned {}", resp.status()))
            }
        }
        LlmProviderKind::Custom => {
            if let Some(ref base_url) = provider.base_url {
                let client = reqwest::Client::new();
                let resp = client
                    .get(format!("{}/v1/models", base_url))
                    .send()
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(format!("Custom endpoint responded with {}", resp.status()))
            } else {
                Err("No base URL configured".to_string())
            }
        }
    }
}

fn hardcoded_anthropic_models(provider_id: Uuid) -> Vec<LlmModel> {
    vec![
        LlmModel {
            id: "claude-opus-4-20250514".to_string(),
            name: "Claude Opus 4".to_string(),
            provider_id,
            context_window: Some(200000),
            supports_tools: true,
            supports_vision: true,
            is_local: false,
            size_bytes: None,
            family: Some("claude-4".to_string()),
        },
        LlmModel {
            id: "claude-sonnet-4-20250514".to_string(),
            name: "Claude Sonnet 4".to_string(),
            provider_id,
            context_window: Some(200000),
            supports_tools: true,
            supports_vision: true,
            is_local: false,
            size_bytes: None,
            family: Some("claude-4".to_string()),
        },
        LlmModel {
            id: "claude-haiku-4-20250514".to_string(),
            name: "Claude Haiku 4".to_string(),
            provider_id,
            context_window: Some(200000),
            supports_tools: true,
            supports_vision: true,
            is_local: false,
            size_bytes: None,
            family: Some("claude-4".to_string()),
        },
        LlmModel {
            id: "claude-3-5-sonnet-20241022".to_string(),
            name: "Claude 3.5 Sonnet".to_string(),
            provider_id,
            context_window: Some(200000),
            supports_tools: true,
            supports_vision: true,
            is_local: false,
            size_bytes: None,
            family: Some("claude-3.5".to_string()),
        },
        LlmModel {
            id: "claude-3-5-haiku-20241022".to_string(),
            name: "Claude 3.5 Haiku".to_string(),
            provider_id,
            context_window: Some(200000),
            supports_tools: true,
            supports_vision: true,
            is_local: false,
            size_bytes: None,
            family: Some("claude-3.5".to_string()),
        },
    ]
}

fn hardcoded_openai_models(provider_id: Uuid) -> Vec<LlmModel> {
    vec![
        LlmModel {
            id: "gpt-4o".to_string(),
            name: "GPT-4o".to_string(),
            provider_id,
            context_window: Some(128000),
            supports_tools: true,
            supports_vision: true,
            is_local: false,
            size_bytes: None,
            family: Some("gpt-4".to_string()),
        },
        LlmModel {
            id: "gpt-4o-mini".to_string(),
            name: "GPT-4o Mini".to_string(),
            provider_id,
            context_window: Some(128000),
            supports_tools: true,
            supports_vision: true,
            is_local: false,
            size_bytes: None,
            family: Some("gpt-4".to_string()),
        },
        LlmModel {
            id: "o3".to_string(),
            name: "o3".to_string(),
            provider_id,
            context_window: Some(200000),
            supports_tools: true,
            supports_vision: true,
            is_local: false,
            size_bytes: None,
            family: Some("o3".to_string()),
        },
        LlmModel {
            id: "o3-mini".to_string(),
            name: "o3-mini".to_string(),
            provider_id,
            context_window: Some(200000),
            supports_tools: true,
            supports_vision: false,
            is_local: false,
            size_bytes: None,
            family: Some("o3".to_string()),
        },
    ]
}

fn hardcoded_google_models(provider_id: Uuid) -> Vec<LlmModel> {
    vec![
        LlmModel {
            id: "gemini-2.5-pro".to_string(),
            name: "Gemini 2.5 Pro".to_string(),
            provider_id,
            context_window: Some(1000000),
            supports_tools: true,
            supports_vision: true,
            is_local: false,
            size_bytes: None,
            family: Some("gemini-2.5".to_string()),
        },
        LlmModel {
            id: "gemini-2.5-flash".to_string(),
            name: "Gemini 2.5 Flash".to_string(),
            provider_id,
            context_window: Some(1000000),
            supports_tools: true,
            supports_vision: true,
            is_local: false,
            size_bytes: None,
            family: Some("gemini-2.5".to_string()),
        },
        LlmModel {
            id: "gemini-2.0-flash".to_string(),
            name: "Gemini 2.0 Flash".to_string(),
            provider_id,
            context_window: Some(1000000),
            supports_tools: true,
            supports_vision: true,
            is_local: false,
            size_bytes: None,
            family: Some("gemini-2.0".to_string()),
        },
    ]
}
