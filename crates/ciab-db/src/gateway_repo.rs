use ciab_core::error::{CiabError, CiabResult};

use crate::Database;

// Re-use the gateway types from ciab-gateway would create a circular dep,
// so we use intermediate row types and let the caller convert.
// However, since ciab-db doesn't depend on ciab-gateway, we accept/return
// the types as raw fields and provide convenience methods.

/// Intermediate row for gateway tunnels.
#[derive(Debug, Clone)]
pub struct GatewayTunnelRow {
    pub id: String,
    pub sandbox_id: Option<String>,
    pub tunnel_type: String,
    pub public_url: String,
    pub local_port: i64,
    pub state: String,
    pub config_json: String,
    pub error_message: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Intermediate row for client tokens.
#[derive(Debug, Clone)]
pub struct ClientTokenRow {
    pub id: String,
    pub name: String,
    pub token_hash: String,
    pub scopes_json: String,
    pub expires_at: Option<String>,
    pub last_used_at: Option<String>,
    pub created_at: String,
    pub revoked_at: Option<String>,
}

impl Database {
    // -----------------------------------------------------------------------
    // Gateway Tunnels
    // -----------------------------------------------------------------------

    pub async fn insert_gateway_tunnel_row(&self, row: &GatewayTunnelRow) -> CiabResult<()> {
        sqlx::query(
            "INSERT INTO gateway_tunnels (id, sandbox_id, tunnel_type, public_url, local_port, state, config_json, error_message, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&row.id)
        .bind(&row.sandbox_id)
        .bind(&row.tunnel_type)
        .bind(&row.public_url)
        .bind(row.local_port)
        .bind(&row.state)
        .bind(&row.config_json)
        .bind(&row.error_message)
        .bind(&row.created_at)
        .bind(&row.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| CiabError::Database(e.to_string()))?;

        Ok(())
    }

    pub async fn get_gateway_tunnel_row(&self, id: &str) -> CiabResult<Option<GatewayTunnelRow>> {
        let row: Option<(
            String,
            Option<String>,
            String,
            String,
            i64,
            String,
            String,
            Option<String>,
            String,
            String,
        )> = sqlx::query_as(
            "SELECT id, sandbox_id, tunnel_type, public_url, local_port, state, config_json, error_message, created_at, updated_at
             FROM gateway_tunnels WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| CiabError::Database(e.to_string()))?;

        Ok(row.map(
            |(
                id,
                sandbox_id,
                tunnel_type,
                public_url,
                local_port,
                state,
                config_json,
                error_message,
                created_at,
                updated_at,
            )| {
                GatewayTunnelRow {
                    id,
                    sandbox_id,
                    tunnel_type,
                    public_url,
                    local_port,
                    state,
                    config_json,
                    error_message,
                    created_at,
                    updated_at,
                }
            },
        ))
    }

    pub async fn list_gateway_tunnel_rows(&self) -> CiabResult<Vec<GatewayTunnelRow>> {
        let rows: Vec<(
            String,
            Option<String>,
            String,
            String,
            i64,
            String,
            String,
            Option<String>,
            String,
            String,
        )> = sqlx::query_as(
            "SELECT id, sandbox_id, tunnel_type, public_url, local_port, state, config_json, error_message, created_at, updated_at
             FROM gateway_tunnels ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CiabError::Database(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(
                |(
                    id,
                    sandbox_id,
                    tunnel_type,
                    public_url,
                    local_port,
                    state,
                    config_json,
                    error_message,
                    created_at,
                    updated_at,
                )| {
                    GatewayTunnelRow {
                        id,
                        sandbox_id,
                        tunnel_type,
                        public_url,
                        local_port,
                        state,
                        config_json,
                        error_message,
                        created_at,
                        updated_at,
                    }
                },
            )
            .collect())
    }

    pub async fn delete_gateway_tunnel_row(&self, id: &str) -> CiabResult<()> {
        sqlx::query("DELETE FROM gateway_tunnels WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| CiabError::Database(e.to_string()))?;

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Client Tokens
    // -----------------------------------------------------------------------

    pub async fn insert_client_token_row(&self, row: &ClientTokenRow) -> CiabResult<()> {
        sqlx::query(
            "INSERT INTO client_tokens (id, name, token_hash, scopes_json, expires_at, last_used_at, created_at, revoked_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&row.id)
        .bind(&row.name)
        .bind(&row.token_hash)
        .bind(&row.scopes_json)
        .bind(&row.expires_at)
        .bind(&row.last_used_at)
        .bind(&row.created_at)
        .bind(&row.revoked_at)
        .execute(&self.pool)
        .await
        .map_err(|e| CiabError::Database(e.to_string()))?;

        Ok(())
    }

    pub async fn get_client_token_row(&self, id: &str) -> CiabResult<Option<ClientTokenRow>> {
        let row: Option<(
            String,
            String,
            String,
            String,
            Option<String>,
            Option<String>,
            String,
            Option<String>,
        )> = sqlx::query_as(
            "SELECT id, name, token_hash, scopes_json, expires_at, last_used_at, created_at, revoked_at
             FROM client_tokens WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| CiabError::Database(e.to_string()))?;

        Ok(row.map(
            |(
                id,
                name,
                token_hash,
                scopes_json,
                expires_at,
                last_used_at,
                created_at,
                revoked_at,
            )| {
                ClientTokenRow {
                    id,
                    name,
                    token_hash,
                    scopes_json,
                    expires_at,
                    last_used_at,
                    created_at,
                    revoked_at,
                }
            },
        ))
    }

    pub async fn get_client_token_row_by_hash(
        &self,
        token_hash: &str,
    ) -> CiabResult<Option<ClientTokenRow>> {
        let row: Option<(
            String,
            String,
            String,
            String,
            Option<String>,
            Option<String>,
            String,
            Option<String>,
        )> = sqlx::query_as(
            "SELECT id, name, token_hash, scopes_json, expires_at, last_used_at, created_at, revoked_at
             FROM client_tokens WHERE token_hash = ?",
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| CiabError::Database(e.to_string()))?;

        Ok(row.map(
            |(
                id,
                name,
                token_hash,
                scopes_json,
                expires_at,
                last_used_at,
                created_at,
                revoked_at,
            )| {
                ClientTokenRow {
                    id,
                    name,
                    token_hash,
                    scopes_json,
                    expires_at,
                    last_used_at,
                    created_at,
                    revoked_at,
                }
            },
        ))
    }

    pub async fn list_client_token_rows(&self) -> CiabResult<Vec<ClientTokenRow>> {
        let rows: Vec<(
            String,
            String,
            String,
            String,
            Option<String>,
            Option<String>,
            String,
            Option<String>,
        )> = sqlx::query_as(
            "SELECT id, name, token_hash, scopes_json, expires_at, last_used_at, created_at, revoked_at
             FROM client_tokens ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CiabError::Database(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(
                |(
                    id,
                    name,
                    token_hash,
                    scopes_json,
                    expires_at,
                    last_used_at,
                    created_at,
                    revoked_at,
                )| {
                    ClientTokenRow {
                        id,
                        name,
                        token_hash,
                        scopes_json,
                        expires_at,
                        last_used_at,
                        created_at,
                        revoked_at,
                    }
                },
            )
            .collect())
    }

    pub async fn revoke_client_token_row(&self, id: &str) -> CiabResult<()> {
        let revoked_at = chrono::Utc::now().to_rfc3339();
        sqlx::query("UPDATE client_tokens SET revoked_at = ? WHERE id = ?")
            .bind(&revoked_at)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| CiabError::Database(e.to_string()))?;

        Ok(())
    }

    pub async fn touch_client_token_row(&self, id: &str) -> CiabResult<()> {
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query("UPDATE client_tokens SET last_used_at = ? WHERE id = ?")
            .bind(&now)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| CiabError::Database(e.to_string()))?;

        Ok(())
    }
}
