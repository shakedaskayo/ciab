use ciab_core::error::{CiabError, CiabResult};
use ciab_core::types::channel::{
    Channel, ChannelBinding, ChannelFilters, ChannelMessage, ChannelProvider,
    ChannelProviderConfig, ChannelRules, ChannelState, MessageDirection,
};
use uuid::Uuid;

use crate::Database;

impl Database {
    pub async fn insert_channel(&self, channel: &Channel) -> CiabResult<()> {
        let id = channel.id.to_string();
        let name = &channel.name;
        let description = channel.description.as_deref();
        let provider = channel.provider.to_string();
        let state = serde_json::to_value(&channel.state)?;
        let state_str = state.as_str().unwrap_or("inactive");
        let binding_json = serde_json::to_string(&channel.binding)?;
        let provider_config_json = serde_json::to_string(&channel.provider_config)?;
        let rules_json = serde_json::to_string(&channel.rules)?;
        let labels_json = serde_json::to_string(&channel.labels)?;
        let error_message = channel.error_message.as_deref();
        let qr_code = channel.qr_code.as_deref();
        let created_at = channel.created_at.to_rfc3339();
        let updated_at = channel.updated_at.to_rfc3339();

        sqlx::query(
            "INSERT INTO channels (id, name, description, provider, state, binding_json, provider_config_json, rules_json, labels_json, error_message, qr_code, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(name)
        .bind(description)
        .bind(&provider)
        .bind(state_str)
        .bind(&binding_json)
        .bind(&provider_config_json)
        .bind(&rules_json)
        .bind(&labels_json)
        .bind(error_message)
        .bind(qr_code)
        .bind(&created_at)
        .bind(&updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| CiabError::Database(e.to_string()))?;

        Ok(())
    }

    pub async fn get_channel(&self, id: &Uuid) -> CiabResult<Option<Channel>> {
        let id_str = id.to_string();

        let row: Option<(
            String, String, Option<String>, String, String, String, String, String, String,
            Option<String>, Option<String>, String, String,
        )> = sqlx::query_as(
            "SELECT id, name, description, provider, state, binding_json, provider_config_json, rules_json, labels_json, error_message, qr_code, created_at, updated_at
             FROM channels WHERE id = ?",
        )
        .bind(&id_str)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| CiabError::Database(e.to_string()))?;

        match row {
            Some(r) => Ok(Some(parse_channel_row(r)?)),
            None => Ok(None),
        }
    }

    pub async fn list_channels(&self, filters: &ChannelFilters) -> CiabResult<Vec<Channel>> {
        let mut query = String::from(
            "SELECT id, name, description, provider, state, binding_json, provider_config_json, rules_json, labels_json, error_message, qr_code, created_at, updated_at FROM channels WHERE 1=1",
        );
        let mut bind_values: Vec<String> = Vec::new();

        if let Some(ref provider) = filters.provider {
            query.push_str(" AND provider = ?");
            bind_values.push(provider.to_string());
        }
        if let Some(ref state) = filters.state {
            let sv = serde_json::to_value(state).unwrap_or_default();
            query.push_str(" AND state = ?");
            bind_values.push(sv.as_str().unwrap_or("inactive").to_string());
        }
        if let Some(ref name) = filters.name {
            query.push_str(" AND name LIKE ?");
            bind_values.push(format!("%{}%", name));
        }

        query.push_str(" ORDER BY created_at DESC");

        let mut q = sqlx::query_as::<
            _,
            (
                String,
                String,
                Option<String>,
                String,
                String,
                String,
                String,
                String,
                String,
                Option<String>,
                Option<String>,
                String,
                String,
            ),
        >(&query);
        for val in &bind_values {
            q = q.bind(val);
        }

        let rows = q
            .fetch_all(&self.pool)
            .await
            .map_err(|e| CiabError::Database(e.to_string()))?;

        let mut results = Vec::new();
        for row in rows {
            results.push(parse_channel_row(row)?);
        }
        Ok(results)
    }

    pub async fn update_channel(&self, id: &Uuid, channel: &Channel) -> CiabResult<()> {
        let id_str = id.to_string();
        let state = serde_json::to_value(&channel.state)?;
        let state_str = state.as_str().unwrap_or("inactive");
        let binding_json = serde_json::to_string(&channel.binding)?;
        let provider_config_json = serde_json::to_string(&channel.provider_config)?;
        let rules_json = serde_json::to_string(&channel.rules)?;
        let labels_json = serde_json::to_string(&channel.labels)?;
        let updated_at = channel.updated_at.to_rfc3339();

        sqlx::query(
            "UPDATE channels SET name = ?, description = ?, state = ?, binding_json = ?, provider_config_json = ?, rules_json = ?, labels_json = ?, error_message = ?, qr_code = ?, updated_at = ? WHERE id = ?",
        )
        .bind(&channel.name)
        .bind(channel.description.as_deref())
        .bind(state_str)
        .bind(&binding_json)
        .bind(&provider_config_json)
        .bind(&rules_json)
        .bind(&labels_json)
        .bind(channel.error_message.as_deref())
        .bind(channel.qr_code.as_deref())
        .bind(&updated_at)
        .bind(&id_str)
        .execute(&self.pool)
        .await
        .map_err(|e| CiabError::Database(e.to_string()))?;

        Ok(())
    }

    pub async fn update_channel_state(
        &self,
        id: &Uuid,
        state: &ChannelState,
        error_message: Option<&str>,
    ) -> CiabResult<()> {
        let id_str = id.to_string();
        let sv = serde_json::to_value(state)?;
        let state_str = sv.as_str().unwrap_or("inactive");
        let updated_at = chrono::Utc::now().to_rfc3339();

        sqlx::query(
            "UPDATE channels SET state = ?, error_message = ?, updated_at = ? WHERE id = ?",
        )
        .bind(state_str)
        .bind(error_message)
        .bind(&updated_at)
        .bind(&id_str)
        .execute(&self.pool)
        .await
        .map_err(|e| CiabError::Database(e.to_string()))?;

        Ok(())
    }

    pub async fn delete_channel(&self, id: &Uuid) -> CiabResult<()> {
        let id_str = id.to_string();

        sqlx::query("DELETE FROM channels WHERE id = ?")
            .bind(&id_str)
            .execute(&self.pool)
            .await
            .map_err(|e| CiabError::Database(e.to_string()))?;

        Ok(())
    }

    pub async fn insert_channel_message(&self, msg: &ChannelMessage) -> CiabResult<()> {
        let id = msg.id.to_string();
        let channel_id = msg.channel_id.to_string();
        let direction = serde_json::to_value(&msg.direction)?;
        let direction_str = direction.as_str().unwrap_or("inbound");
        let sandbox_id = msg.sandbox_id.map(|s| s.to_string());
        let session_id = msg.session_id.map(|s| s.to_string());
        let metadata_json = serde_json::to_string(&msg.platform_metadata)?;
        let timestamp = msg.timestamp.to_rfc3339();

        sqlx::query(
            "INSERT INTO channel_messages (id, channel_id, direction, sender_id, sender_name, sandbox_id, session_id, content, platform_metadata_json, timestamp)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(&channel_id)
        .bind(direction_str)
        .bind(&msg.sender_id)
        .bind(msg.sender_name.as_deref())
        .bind(sandbox_id.as_deref())
        .bind(session_id.as_deref())
        .bind(&msg.content)
        .bind(&metadata_json)
        .bind(&timestamp)
        .execute(&self.pool)
        .await
        .map_err(|e| CiabError::Database(e.to_string()))?;

        Ok(())
    }

    pub async fn list_channel_messages(
        &self,
        channel_id: &Uuid,
        limit: Option<u32>,
        sender_id: Option<&str>,
    ) -> CiabResult<Vec<ChannelMessage>> {
        let channel_id_str = channel_id.to_string();
        let mut query = String::from(
            "SELECT id, channel_id, direction, sender_id, sender_name, sandbox_id, session_id, content, platform_metadata_json, timestamp
             FROM channel_messages WHERE channel_id = ?",
        );
        let mut bind_values: Vec<String> = vec![channel_id_str];

        if let Some(sid) = sender_id {
            query.push_str(" AND sender_id = ?");
            bind_values.push(sid.to_string());
        }

        query.push_str(" ORDER BY timestamp DESC");

        if let Some(lim) = limit {
            query.push_str(&format!(" LIMIT {}", lim));
        }

        let mut q = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                String,
                Option<String>,
                Option<String>,
                Option<String>,
                String,
                String,
                String,
            ),
        >(&query);
        for val in &bind_values {
            q = q.bind(val);
        }

        let rows = q
            .fetch_all(&self.pool)
            .await
            .map_err(|e| CiabError::Database(e.to_string()))?;

        let mut results = Vec::new();
        for (
            id,
            channel_id,
            direction,
            sender_id,
            sender_name,
            sandbox_id,
            session_id,
            content,
            metadata_json,
            timestamp,
        ) in rows
        {
            let direction: MessageDirection = serde_json::from_str(&format!("\"{}\"", direction))
                .unwrap_or(MessageDirection::Inbound);
            let platform_metadata = serde_json::from_str(&metadata_json).unwrap_or_default();
            let timestamp = chrono::DateTime::parse_from_rfc3339(&timestamp)
                .map_err(|e| CiabError::Database(e.to_string()))?
                .with_timezone(&chrono::Utc);

            results.push(ChannelMessage {
                id: Uuid::parse_str(&id).map_err(|e| CiabError::Database(e.to_string()))?,
                channel_id: Uuid::parse_str(&channel_id)
                    .map_err(|e| CiabError::Database(e.to_string()))?,
                direction,
                sender_id,
                sender_name,
                sandbox_id: sandbox_id
                    .map(|s| Uuid::parse_str(&s))
                    .transpose()
                    .map_err(|e| CiabError::Database(e.to_string()))?,
                session_id: session_id
                    .map(|s| Uuid::parse_str(&s))
                    .transpose()
                    .map_err(|e| CiabError::Database(e.to_string()))?,
                content,
                platform_metadata,
                timestamp,
            });
        }

        Ok(results)
    }
}

fn parse_channel_row(
    row: (
        String,
        String,
        Option<String>,
        String,
        String,
        String,
        String,
        String,
        String,
        Option<String>,
        Option<String>,
        String,
        String,
    ),
) -> CiabResult<Channel> {
    let (
        id,
        name,
        description,
        provider,
        state,
        binding_json,
        provider_config_json,
        rules_json,
        labels_json,
        error_message,
        qr_code,
        created_at,
        updated_at,
    ) = row;

    let provider: ChannelProvider = serde_json::from_str(&format!("\"{}\"", provider))
        .map_err(|e| CiabError::Database(format!("invalid provider: {}", e)))?;
    let state: ChannelState =
        serde_json::from_str(&format!("\"{}\"", state)).unwrap_or(ChannelState::Inactive);
    let binding: ChannelBinding = serde_json::from_str(&binding_json)?;
    let provider_config: ChannelProviderConfig = serde_json::from_str(&provider_config_json)?;
    let rules: ChannelRules = serde_json::from_str(&rules_json).unwrap_or_default();
    let labels = serde_json::from_str(&labels_json).unwrap_or_default();
    let created_at = chrono::DateTime::parse_from_rfc3339(&created_at)
        .map_err(|e| CiabError::Database(e.to_string()))?
        .with_timezone(&chrono::Utc);
    let updated_at = chrono::DateTime::parse_from_rfc3339(&updated_at)
        .map_err(|e| CiabError::Database(e.to_string()))?
        .with_timezone(&chrono::Utc);

    Ok(Channel {
        id: Uuid::parse_str(&id).map_err(|e| CiabError::Database(e.to_string()))?,
        name,
        description,
        provider,
        state,
        binding,
        provider_config,
        rules,
        labels,
        error_message,
        qr_code,
        created_at,
        updated_at,
    })
}
