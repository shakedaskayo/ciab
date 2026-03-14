use ciab_core::error::{CiabError, CiabResult};

use crate::Database;

impl Database {
    pub async fn run_migrations(&self) -> CiabResult<()> {
        let statements = [
            "CREATE TABLE IF NOT EXISTS sandboxes (
                id TEXT PRIMARY KEY,
                name TEXT,
                state TEXT NOT NULL DEFAULT 'pending',
                persistence TEXT NOT NULL DEFAULT 'ephemeral',
                agent_provider TEXT NOT NULL,
                spec_json TEXT NOT NULL,
                info_json TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            "CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                sandbox_id TEXT NOT NULL REFERENCES sandboxes(id),
                state TEXT NOT NULL DEFAULT 'active',
                metadata_json TEXT NOT NULL DEFAULT '{}',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            "CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL REFERENCES sessions(id),
                role TEXT NOT NULL,
                content_json TEXT NOT NULL,
                timestamp TEXT NOT NULL
            )",
            "CREATE TABLE IF NOT EXISTS credentials (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                credential_type TEXT NOT NULL,
                encrypted_data BLOB NOT NULL,
                labels_json TEXT NOT NULL DEFAULT '{}',
                created_at TEXT NOT NULL,
                expires_at TEXT
            )",
            "CREATE TABLE IF NOT EXISTS oauth_tokens (
                id TEXT PRIMARY KEY,
                provider TEXT NOT NULL,
                credential_id TEXT NOT NULL REFERENCES credentials(id),
                access_token_enc BLOB NOT NULL,
                refresh_token_enc BLOB,
                expires_at TEXT
            )",
            "CREATE TABLE IF NOT EXISTS workspaces (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                spec_json TEXT NOT NULL,
                labels_json TEXT NOT NULL DEFAULT '{}',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            "CREATE TABLE IF NOT EXISTS workspace_sandboxes (
                workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
                sandbox_id TEXT NOT NULL REFERENCES sandboxes(id) ON DELETE CASCADE,
                created_at TEXT NOT NULL,
                PRIMARY KEY (workspace_id, sandbox_id)
            )",
            "CREATE TABLE IF NOT EXISTS template_sources (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                url TEXT NOT NULL,
                branch TEXT NOT NULL DEFAULT 'main',
                templates_path TEXT NOT NULL DEFAULT '.ciab/templates',
                last_synced_at TEXT,
                template_count INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            "CREATE TABLE IF NOT EXISTS gateway_tunnels (
                id TEXT PRIMARY KEY,
                sandbox_id TEXT REFERENCES sandboxes(id) ON DELETE CASCADE,
                tunnel_type TEXT NOT NULL,
                public_url TEXT NOT NULL,
                local_port INTEGER NOT NULL,
                state TEXT NOT NULL DEFAULT 'active',
                config_json TEXT NOT NULL DEFAULT '{}',
                error_message TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            "CREATE TABLE IF NOT EXISTS client_tokens (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                token_hash TEXT NOT NULL UNIQUE,
                scopes_json TEXT NOT NULL,
                expires_at TEXT,
                last_used_at TEXT,
                created_at TEXT NOT NULL,
                revoked_at TEXT
            )",
            "CREATE INDEX IF NOT EXISTS idx_client_tokens_hash ON client_tokens(token_hash)",
            "CREATE INDEX IF NOT EXISTS idx_gateway_tunnels_sandbox ON gateway_tunnels(sandbox_id)",
            "CREATE TABLE IF NOT EXISTS channels (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                provider TEXT NOT NULL,
                state TEXT NOT NULL DEFAULT 'inactive',
                binding_json TEXT NOT NULL,
                provider_config_json TEXT NOT NULL,
                rules_json TEXT NOT NULL DEFAULT '{}',
                labels_json TEXT NOT NULL DEFAULT '{}',
                error_message TEXT,
                qr_code TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            "CREATE TABLE IF NOT EXISTS channel_messages (
                id TEXT PRIMARY KEY,
                channel_id TEXT NOT NULL REFERENCES channels(id) ON DELETE CASCADE,
                direction TEXT NOT NULL,
                sender_id TEXT NOT NULL,
                sender_name TEXT,
                sandbox_id TEXT,
                session_id TEXT,
                content TEXT NOT NULL,
                platform_metadata_json TEXT NOT NULL DEFAULT '{}',
                timestamp TEXT NOT NULL
            )",
            "CREATE INDEX IF NOT EXISTS idx_channel_messages_channel ON channel_messages(channel_id)",
            "CREATE INDEX IF NOT EXISTS idx_channel_messages_sender ON channel_messages(channel_id, sender_id)",
            "CREATE TABLE IF NOT EXISTS llm_providers (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                kind TEXT NOT NULL,
                enabled INTEGER NOT NULL DEFAULT 1,
                base_url TEXT,
                api_key_credential_id TEXT,
                default_model TEXT,
                is_local INTEGER NOT NULL DEFAULT 0,
                auto_detected INTEGER NOT NULL DEFAULT 0,
                extra_json TEXT NOT NULL DEFAULT '{}',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            "CREATE TABLE IF NOT EXISTS llm_models (
                id TEXT NOT NULL,
                provider_id TEXT NOT NULL REFERENCES llm_providers(id) ON DELETE CASCADE,
                name TEXT NOT NULL,
                context_window INTEGER,
                supports_tools INTEGER NOT NULL DEFAULT 0,
                supports_vision INTEGER NOT NULL DEFAULT 0,
                is_local INTEGER NOT NULL DEFAULT 0,
                size_bytes INTEGER,
                family TEXT,
                fetched_at TEXT NOT NULL,
                PRIMARY KEY (id, provider_id)
            )",
            "CREATE INDEX IF NOT EXISTS idx_llm_models_provider ON llm_models(provider_id)",
        ];

        for sql in &statements {
            sqlx::query(sql)
                .execute(&self.pool)
                .await
                .map_err(|e| CiabError::Database(e.to_string()))?;
        }

        tracing::info!("Database migrations completed successfully");
        Ok(())
    }
}
