use ciab_core::error::{CiabError, CiabResult};
use ciab_core::types::sandbox::{SandboxFilters, SandboxInfo, SandboxState};
use uuid::Uuid;

use crate::Database;

impl Database {
    pub async fn insert_sandbox(&self, info: &SandboxInfo) -> CiabResult<()> {
        let id = info.id.to_string();
        let name = info.name.clone();
        let state = serde_json::to_value(&info.state)?
            .as_str()
            .unwrap_or("pending")
            .to_string();
        let persistence = serde_json::to_value(&info.persistence)?
            .as_str()
            .unwrap_or("ephemeral")
            .to_string();
        let agent_provider = &info.agent_provider;
        let spec_json = serde_json::to_string(&info.spec)?;
        let info_json = serde_json::to_string(info)?;
        let created_at = info.created_at.to_rfc3339();
        let updated_at = info.updated_at.to_rfc3339();

        sqlx::query(
            "INSERT INTO sandboxes (id, name, state, persistence, agent_provider, spec_json, info_json, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(&name)
        .bind(&state)
        .bind(&persistence)
        .bind(agent_provider)
        .bind(&spec_json)
        .bind(&info_json)
        .bind(&created_at)
        .bind(&updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| CiabError::Database(e.to_string()))?;

        Ok(())
    }

    pub async fn get_sandbox(&self, id: &Uuid) -> CiabResult<Option<SandboxInfo>> {
        let id_str = id.to_string();

        let row: Option<(String,)> = sqlx::query_as("SELECT info_json FROM sandboxes WHERE id = ?")
            .bind(&id_str)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| CiabError::Database(e.to_string()))?;

        match row {
            Some((json,)) => {
                let info: SandboxInfo = serde_json::from_str(&json)?;
                Ok(Some(info))
            }
            None => Ok(None),
        }
    }

    pub async fn list_sandboxes(&self, filters: &SandboxFilters) -> CiabResult<Vec<SandboxInfo>> {
        let mut query = String::from("SELECT info_json FROM sandboxes WHERE 1=1");
        let mut bind_values: Vec<String> = Vec::new();

        if let Some(ref state) = filters.state {
            let state_str = serde_json::to_value(state)?
                .as_str()
                .unwrap_or("pending")
                .to_string();
            query.push_str(" AND state = ?");
            bind_values.push(state_str);
        }

        if let Some(ref provider) = filters.provider {
            query.push_str(" AND agent_provider = ?");
            bind_values.push(provider.clone());
        }

        query.push_str(" ORDER BY created_at DESC");

        // Build the query dynamically
        let mut q = sqlx::query_as::<_, (String,)>(&query);
        for val in &bind_values {
            q = q.bind(val);
        }

        let rows: Vec<(String,)> = q
            .fetch_all(&self.pool)
            .await
            .map_err(|e| CiabError::Database(e.to_string()))?;

        let mut results = Vec::new();
        for (json,) in rows {
            let info: SandboxInfo = serde_json::from_str(&json)?;
            // Apply label filters in-memory
            if !filters.labels.is_empty() {
                let matches = filters
                    .labels
                    .iter()
                    .all(|(k, v)| info.labels.get(k).is_some_and(|lv| lv == v));
                if !matches {
                    continue;
                }
            }
            results.push(info);
        }

        Ok(results)
    }

    pub async fn update_sandbox_state(&self, id: &Uuid, state: &SandboxState) -> CiabResult<()> {
        let id_str = id.to_string();
        let state_str = serde_json::to_value(state)?
            .as_str()
            .unwrap_or("pending")
            .to_string();
        let updated_at = chrono::Utc::now().to_rfc3339();

        // First update the state column
        sqlx::query("UPDATE sandboxes SET state = ?, updated_at = ? WHERE id = ?")
            .bind(&state_str)
            .bind(&updated_at)
            .bind(&id_str)
            .execute(&self.pool)
            .await
            .map_err(|e| CiabError::Database(e.to_string()))?;

        // Also update info_json to keep in sync
        let row: Option<(String,)> = sqlx::query_as("SELECT info_json FROM sandboxes WHERE id = ?")
            .bind(&id_str)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| CiabError::Database(e.to_string()))?;

        if let Some((json,)) = row {
            let mut info: SandboxInfo = serde_json::from_str(&json)?;
            info.state = state.clone();
            info.updated_at = chrono::Utc::now();
            let new_json = serde_json::to_string(&info)?;
            sqlx::query("UPDATE sandboxes SET info_json = ? WHERE id = ?")
                .bind(&new_json)
                .bind(&id_str)
                .execute(&self.pool)
                .await
                .map_err(|e| CiabError::Database(e.to_string()))?;
        }

        Ok(())
    }

    pub async fn update_sandbox_info(&self, id: &Uuid, info: &SandboxInfo) -> CiabResult<()> {
        let id_str = id.to_string();
        let state_str = serde_json::to_value(&info.state)?
            .as_str()
            .unwrap_or("pending")
            .to_string();
        let persistence_str = serde_json::to_value(&info.persistence)?
            .as_str()
            .unwrap_or("ephemeral")
            .to_string();
        let spec_json = serde_json::to_string(&info.spec)?;
        let info_json = serde_json::to_string(info)?;
        let updated_at = info.updated_at.to_rfc3339();

        sqlx::query(
            "UPDATE sandboxes SET name = ?, state = ?, persistence = ?, agent_provider = ?, spec_json = ?, info_json = ?, updated_at = ? WHERE id = ?"
        )
        .bind(&info.name)
        .bind(&state_str)
        .bind(&persistence_str)
        .bind(&info.agent_provider)
        .bind(&spec_json)
        .bind(&info_json)
        .bind(&updated_at)
        .bind(&id_str)
        .execute(&self.pool)
        .await
        .map_err(|e| CiabError::Database(e.to_string()))?;

        Ok(())
    }

    pub async fn delete_sandbox(&self, id: &Uuid) -> CiabResult<()> {
        let id_str = id.to_string();

        sqlx::query("DELETE FROM sandboxes WHERE id = ?")
            .bind(&id_str)
            .execute(&self.pool)
            .await
            .map_err(|e| CiabError::Database(e.to_string()))?;

        Ok(())
    }
}
