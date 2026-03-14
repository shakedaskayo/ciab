use ciab_core::error::{CiabError, CiabResult};
use ciab_core::types::session::{Session, SessionState};
use uuid::Uuid;

use crate::Database;

impl Database {
    pub async fn insert_session(&self, session: &Session) -> CiabResult<()> {
        let id = session.id.to_string();
        let sandbox_id = session.sandbox_id.to_string();
        let state = serde_json::to_value(&session.state)?
            .as_str()
            .unwrap_or("active")
            .to_string();
        let metadata_json = serde_json::to_string(&session.metadata)?;
        let created_at = session.created_at.to_rfc3339();
        let updated_at = session.updated_at.to_rfc3339();

        sqlx::query(
            "INSERT INTO sessions (id, sandbox_id, state, metadata_json, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(&sandbox_id)
        .bind(&state)
        .bind(&metadata_json)
        .bind(&created_at)
        .bind(&updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| CiabError::Database(e.to_string()))?;

        Ok(())
    }

    pub async fn get_session(&self, id: &Uuid) -> CiabResult<Option<Session>> {
        let id_str = id.to_string();

        let row: Option<(String, String, String, String, String, String)> = sqlx::query_as(
            "SELECT id, sandbox_id, state, metadata_json, created_at, updated_at FROM sessions WHERE id = ?",
        )
        .bind(&id_str)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| CiabError::Database(e.to_string()))?;

        match row {
            Some((id, sandbox_id, state, metadata_json, created_at, updated_at)) => {
                let session = Session {
                    id: id
                        .parse()
                        .map_err(|e: uuid::Error| CiabError::Database(e.to_string()))?,
                    sandbox_id: sandbox_id
                        .parse()
                        .map_err(|e: uuid::Error| CiabError::Database(e.to_string()))?,
                    state: serde_json::from_value(serde_json::Value::String(state))?,
                    metadata: serde_json::from_str(&metadata_json)?,
                    created_at: chrono::DateTime::parse_from_rfc3339(&created_at)
                        .map_err(|e| CiabError::Database(e.to_string()))?
                        .with_timezone(&chrono::Utc),
                    updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at)
                        .map_err(|e| CiabError::Database(e.to_string()))?
                        .with_timezone(&chrono::Utc),
                };
                Ok(Some(session))
            }
            None => Ok(None),
        }
    }

    pub async fn list_sessions(&self, sandbox_id: &Uuid) -> CiabResult<Vec<Session>> {
        let sandbox_id_str = sandbox_id.to_string();

        let rows: Vec<(String, String, String, String, String, String)> = sqlx::query_as(
            "SELECT id, sandbox_id, state, metadata_json, created_at, updated_at FROM sessions WHERE sandbox_id = ? ORDER BY created_at DESC",
        )
        .bind(&sandbox_id_str)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CiabError::Database(e.to_string()))?;

        let mut sessions = Vec::new();
        for (id, sandbox_id, state, metadata_json, created_at, updated_at) in rows {
            let session = Session {
                id: id
                    .parse()
                    .map_err(|e: uuid::Error| CiabError::Database(e.to_string()))?,
                sandbox_id: sandbox_id
                    .parse()
                    .map_err(|e: uuid::Error| CiabError::Database(e.to_string()))?,
                state: serde_json::from_value(serde_json::Value::String(state))?,
                metadata: serde_json::from_str(&metadata_json)?,
                created_at: chrono::DateTime::parse_from_rfc3339(&created_at)
                    .map_err(|e| CiabError::Database(e.to_string()))?
                    .with_timezone(&chrono::Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at)
                    .map_err(|e| CiabError::Database(e.to_string()))?
                    .with_timezone(&chrono::Utc),
            };
            sessions.push(session);
        }

        Ok(sessions)
    }

    pub async fn update_session_metadata(
        &self,
        id: &Uuid,
        metadata: &std::collections::HashMap<String, serde_json::Value>,
    ) -> CiabResult<()> {
        let id_str = id.to_string();
        let metadata_json = serde_json::to_string(metadata)?;
        let updated_at = chrono::Utc::now().to_rfc3339();

        sqlx::query("UPDATE sessions SET metadata_json = ?, updated_at = ? WHERE id = ?")
            .bind(&metadata_json)
            .bind(&updated_at)
            .bind(&id_str)
            .execute(&self.pool)
            .await
            .map_err(|e| CiabError::Database(e.to_string()))?;

        Ok(())
    }

    pub async fn update_session_state(&self, id: &Uuid, state: &SessionState) -> CiabResult<()> {
        let id_str = id.to_string();
        let state_str = serde_json::to_value(state)?
            .as_str()
            .unwrap_or("active")
            .to_string();
        let updated_at = chrono::Utc::now().to_rfc3339();

        sqlx::query("UPDATE sessions SET state = ?, updated_at = ? WHERE id = ?")
            .bind(&state_str)
            .bind(&updated_at)
            .bind(&id_str)
            .execute(&self.pool)
            .await
            .map_err(|e| CiabError::Database(e.to_string()))?;

        Ok(())
    }
}
