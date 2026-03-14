use ciab_core::error::{CiabError, CiabResult};
use ciab_core::types::session::Message;
use uuid::Uuid;

use crate::Database;

impl Database {
    pub async fn insert_message(&self, msg: &Message) -> CiabResult<()> {
        let id = msg.id.to_string();
        let session_id = msg.session_id.to_string();
        let role = serde_json::to_value(&msg.role)?
            .as_str()
            .unwrap_or("user")
            .to_string();
        let content_json = serde_json::to_string(&msg.content)?;
        let timestamp = msg.timestamp.to_rfc3339();

        sqlx::query(
            "INSERT INTO messages (id, session_id, role, content_json, timestamp)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(&session_id)
        .bind(&role)
        .bind(&content_json)
        .bind(&timestamp)
        .execute(&self.pool)
        .await
        .map_err(|e| CiabError::Database(e.to_string()))?;

        Ok(())
    }

    pub async fn delete_message(&self, id: &Uuid) -> CiabResult<()> {
        let id_str = id.to_string();
        sqlx::query("DELETE FROM messages WHERE id = ?")
            .bind(&id_str)
            .execute(&self.pool)
            .await
            .map_err(|e| CiabError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn delete_session_messages(&self, session_id: &Uuid) -> CiabResult<()> {
        let session_id_str = session_id.to_string();
        sqlx::query("DELETE FROM messages WHERE session_id = ?")
            .bind(&session_id_str)
            .execute(&self.pool)
            .await
            .map_err(|e| CiabError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn get_messages(
        &self,
        session_id: &Uuid,
        limit: Option<i64>,
    ) -> CiabResult<Vec<Message>> {
        let session_id_str = session_id.to_string();

        let rows: Vec<(String, String, String, String, String)> = if let Some(limit) = limit {
            sqlx::query_as(
                "SELECT id, session_id, role, content_json, timestamp FROM messages WHERE session_id = ? ORDER BY timestamp ASC LIMIT ?",
            )
            .bind(&session_id_str)
            .bind(limit)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| CiabError::Database(e.to_string()))?
        } else {
            sqlx::query_as(
                "SELECT id, session_id, role, content_json, timestamp FROM messages WHERE session_id = ? ORDER BY timestamp ASC",
            )
            .bind(&session_id_str)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| CiabError::Database(e.to_string()))?
        };

        let mut messages = Vec::new();
        for (id, session_id, role, content_json, timestamp) in rows {
            let msg = Message {
                id: id
                    .parse()
                    .map_err(|e: uuid::Error| CiabError::Database(e.to_string()))?,
                session_id: session_id
                    .parse()
                    .map_err(|e: uuid::Error| CiabError::Database(e.to_string()))?,
                role: serde_json::from_value(serde_json::Value::String(role))?,
                content: serde_json::from_str(&content_json)?,
                timestamp: chrono::DateTime::parse_from_rfc3339(&timestamp)
                    .map_err(|e| CiabError::Database(e.to_string()))?
                    .with_timezone(&chrono::Utc),
            };
            messages.push(msg);
        }

        Ok(messages)
    }
}
