use chrono::{DateTime, Utc};
use ciab_core::error::{CiabError, CiabResult};
use ciab_core::types::llm_provider::{LlmModel, LlmProvider, LlmProviderKind};
use uuid::Uuid;

use crate::Database;

impl Database {
    pub async fn insert_llm_provider(&self, provider: &LlmProvider) -> CiabResult<()> {
        let id_str = provider.id.to_string();
        let kind_str = provider.kind.to_string();
        let api_key_cred_str = provider.api_key_credential_id.map(|u| u.to_string());
        let extra_json = serde_json::to_string(&provider.extra)?;
        let created_at = provider.created_at.to_rfc3339();
        let updated_at = provider.updated_at.to_rfc3339();

        sqlx::query(
            "INSERT INTO llm_providers (id, name, kind, enabled, base_url, api_key_credential_id, default_model, is_local, auto_detected, extra_json, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id_str)
        .bind(&provider.name)
        .bind(&kind_str)
        .bind(provider.enabled)
        .bind(&provider.base_url)
        .bind(&api_key_cred_str)
        .bind(&provider.default_model)
        .bind(provider.is_local)
        .bind(provider.auto_detected)
        .bind(&extra_json)
        .bind(&created_at)
        .bind(&updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| CiabError::Database(e.to_string()))?;

        Ok(())
    }

    pub async fn get_llm_provider(&self, id: &Uuid) -> CiabResult<Option<LlmProvider>> {
        let id_str = id.to_string();

        let row: Option<(
            String,
            String,
            String,
            bool,
            Option<String>,
            Option<String>,
            Option<String>,
            bool,
            bool,
            String,
            String,
            String,
        )> = sqlx::query_as(
            "SELECT id, name, kind, enabled, base_url, api_key_credential_id, default_model, is_local, auto_detected, extra_json, created_at, updated_at FROM llm_providers WHERE id = ?",
        )
        .bind(&id_str)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| CiabError::Database(e.to_string()))?;

        match row {
            Some(r) => Ok(Some(parse_llm_provider_row(r)?)),
            None => Ok(None),
        }
    }

    pub async fn list_llm_providers(&self) -> CiabResult<Vec<LlmProvider>> {
        let rows: Vec<(
            String,
            String,
            String,
            bool,
            Option<String>,
            Option<String>,
            Option<String>,
            bool,
            bool,
            String,
            String,
            String,
        )> = sqlx::query_as(
            "SELECT id, name, kind, enabled, base_url, api_key_credential_id, default_model, is_local, auto_detected, extra_json, created_at, updated_at FROM llm_providers ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CiabError::Database(e.to_string()))?;

        let mut results = Vec::new();
        for row in rows {
            results.push(parse_llm_provider_row(row)?);
        }
        Ok(results)
    }

    pub async fn update_llm_provider(&self, provider: &LlmProvider) -> CiabResult<()> {
        let id_str = provider.id.to_string();
        let kind_str = provider.kind.to_string();
        let api_key_cred_str = provider.api_key_credential_id.map(|u| u.to_string());
        let extra_json = serde_json::to_string(&provider.extra)?;
        let updated_at = Utc::now().to_rfc3339();

        sqlx::query(
            "UPDATE llm_providers SET name = ?, kind = ?, enabled = ?, base_url = ?, api_key_credential_id = ?, default_model = ?, is_local = ?, auto_detected = ?, extra_json = ?, updated_at = ? WHERE id = ?",
        )
        .bind(&provider.name)
        .bind(&kind_str)
        .bind(provider.enabled)
        .bind(&provider.base_url)
        .bind(&api_key_cred_str)
        .bind(&provider.default_model)
        .bind(provider.is_local)
        .bind(provider.auto_detected)
        .bind(&extra_json)
        .bind(&updated_at)
        .bind(&id_str)
        .execute(&self.pool)
        .await
        .map_err(|e| CiabError::Database(e.to_string()))?;

        Ok(())
    }

    pub async fn delete_llm_provider(&self, id: &Uuid) -> CiabResult<()> {
        let id_str = id.to_string();

        sqlx::query("DELETE FROM llm_providers WHERE id = ?")
            .bind(&id_str)
            .execute(&self.pool)
            .await
            .map_err(|e| CiabError::Database(e.to_string()))?;

        Ok(())
    }

    pub async fn insert_llm_models(
        &self,
        provider_id: &Uuid,
        models: &[LlmModel],
    ) -> CiabResult<()> {
        let provider_id_str = provider_id.to_string();
        let fetched_at = Utc::now().to_rfc3339();

        for model in models {
            sqlx::query(
                "INSERT OR REPLACE INTO llm_models (id, provider_id, name, context_window, supports_tools, supports_vision, is_local, size_bytes, family, fetched_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&model.id)
            .bind(&provider_id_str)
            .bind(&model.name)
            .bind(model.context_window.map(|v| v as i64))
            .bind(model.supports_tools)
            .bind(model.supports_vision)
            .bind(model.is_local)
            .bind(model.size_bytes.map(|v| v as i64))
            .bind(&model.family)
            .bind(&fetched_at)
            .execute(&self.pool)
            .await
            .map_err(|e| CiabError::Database(e.to_string()))?;
        }

        Ok(())
    }

    pub async fn list_llm_models(&self, provider_id: &Uuid) -> CiabResult<Vec<LlmModel>> {
        let provider_id_str = provider_id.to_string();

        let rows: Vec<(
            String,
            String,
            String,
            Option<i64>,
            bool,
            bool,
            bool,
            Option<i64>,
            Option<String>,
        )> = sqlx::query_as(
            "SELECT id, provider_id, name, context_window, supports_tools, supports_vision, is_local, size_bytes, family FROM llm_models WHERE provider_id = ? ORDER BY name",
        )
        .bind(&provider_id_str)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CiabError::Database(e.to_string()))?;

        let mut results = Vec::new();
        for (
            id,
            provider_id,
            name,
            context_window,
            supports_tools,
            supports_vision,
            is_local,
            size_bytes,
            family,
        ) in rows
        {
            results.push(LlmModel {
                id,
                name,
                provider_id: provider_id
                    .parse()
                    .map_err(|e: uuid::Error| CiabError::Database(e.to_string()))?,
                context_window: context_window.map(|v| v as u64),
                supports_tools,
                supports_vision,
                is_local,
                size_bytes: size_bytes.map(|v| v as u64),
                family,
            });
        }

        Ok(results)
    }

    pub async fn delete_llm_models_by_provider(&self, provider_id: &Uuid) -> CiabResult<()> {
        let provider_id_str = provider_id.to_string();

        sqlx::query("DELETE FROM llm_models WHERE provider_id = ?")
            .bind(&provider_id_str)
            .execute(&self.pool)
            .await
            .map_err(|e| CiabError::Database(e.to_string()))?;

        Ok(())
    }
}

fn parse_llm_provider_row(
    row: (
        String,
        String,
        String,
        bool,
        Option<String>,
        Option<String>,
        Option<String>,
        bool,
        bool,
        String,
        String,
        String,
    ),
) -> CiabResult<LlmProvider> {
    let (
        id,
        name,
        kind,
        enabled,
        base_url,
        api_key_credential_id,
        default_model,
        is_local,
        auto_detected,
        extra_json,
        created_at,
        updated_at,
    ) = row;

    Ok(LlmProvider {
        id: id
            .parse()
            .map_err(|e: uuid::Error| CiabError::Database(e.to_string()))?,
        name,
        kind: kind
            .parse::<LlmProviderKind>()
            .map_err(CiabError::Database)?,
        enabled,
        base_url,
        api_key_credential_id: api_key_credential_id
            .map(|s| s.parse::<Uuid>())
            .transpose()
            .map_err(|e| CiabError::Database(e.to_string()))?,
        default_model,
        is_local,
        auto_detected,
        extra: serde_json::from_str(&extra_json).unwrap_or_default(),
        created_at: DateTime::parse_from_rfc3339(&created_at)
            .map_err(|e| CiabError::Database(e.to_string()))?
            .with_timezone(&Utc),
        updated_at: DateTime::parse_from_rfc3339(&updated_at)
            .map_err(|e| CiabError::Database(e.to_string()))?
            .with_timezone(&Utc),
    })
}
