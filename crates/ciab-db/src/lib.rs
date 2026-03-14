mod channel_repo;
mod credential_repo;
mod gateway_repo;
mod llm_provider_repo;
mod message_repo;
mod migrations;
mod sandbox_repo;
mod session_repo;
mod workspace_repo;

use ciab_core::error::{CiabError, CiabResult};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::str::FromStr;

pub use credential_repo::CredentialRow;
pub use gateway_repo::{ClientTokenRow, GatewayTunnelRow};

#[derive(Debug, Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new(database_url: &str) -> CiabResult<Self> {
        let options = SqliteConnectOptions::from_str(database_url)
            .map_err(|e| CiabError::Database(e.to_string()))?
            .create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
            .map_err(|e| CiabError::Database(e.to_string()))?;

        let db = Self { pool };
        db.run_migrations().await?;
        Ok(db)
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}
