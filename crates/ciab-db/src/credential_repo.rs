use std::collections::HashMap;

use chrono::{DateTime, Utc};
use ciab_core::error::{CiabError, CiabResult};
use ciab_core::types::credentials::{CredentialSet, CredentialType, OAuthProvider};
use uuid::Uuid;

use crate::Database;

#[derive(Debug, Clone)]
pub struct CredentialRow {
    pub id: Uuid,
    pub name: String,
    pub credential_type: CredentialType,
    pub encrypted_data: Vec<u8>,
    pub labels: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl Database {
    pub async fn insert_credential(
        &self,
        id: &Uuid,
        name: &str,
        cred_type: &CredentialType,
        encrypted_data: &[u8],
        labels: &HashMap<String, String>,
        expires_at: Option<DateTime<Utc>>,
    ) -> CiabResult<()> {
        let id_str = id.to_string();
        let cred_type_str = serde_json::to_value(cred_type)?
            .as_str()
            .unwrap_or("api_key")
            .to_string();
        let labels_json = serde_json::to_string(labels)?;
        let created_at = Utc::now().to_rfc3339();
        let expires_at_str = expires_at.map(|dt| dt.to_rfc3339());

        sqlx::query(
            "INSERT INTO credentials (id, name, credential_type, encrypted_data, labels_json, created_at, expires_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id_str)
        .bind(name)
        .bind(&cred_type_str)
        .bind(encrypted_data)
        .bind(&labels_json)
        .bind(&created_at)
        .bind(&expires_at_str)
        .execute(&self.pool)
        .await
        .map_err(|e| CiabError::Database(e.to_string()))?;

        Ok(())
    }

    pub async fn get_credential(&self, id: &Uuid) -> CiabResult<Option<CredentialRow>> {
        let id_str = id.to_string();

        let row: Option<(String, String, String, Vec<u8>, String, String, Option<String>)> =
            sqlx::query_as(
                "SELECT id, name, credential_type, encrypted_data, labels_json, created_at, expires_at FROM credentials WHERE id = ?",
            )
            .bind(&id_str)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| CiabError::Database(e.to_string()))?;

        match row {
            Some((id, name, cred_type, encrypted_data, labels_json, created_at, expires_at)) => {
                let credential = CredentialRow {
                    id: id
                        .parse()
                        .map_err(|e: uuid::Error| CiabError::Database(e.to_string()))?,
                    name,
                    credential_type: serde_json::from_value(serde_json::Value::String(cred_type))?,
                    encrypted_data,
                    labels: serde_json::from_str(&labels_json)?,
                    created_at: chrono::DateTime::parse_from_rfc3339(&created_at)
                        .map_err(|e| CiabError::Database(e.to_string()))?
                        .with_timezone(&Utc),
                    expires_at: expires_at
                        .map(|s| {
                            chrono::DateTime::parse_from_rfc3339(&s)
                                .map(|dt| dt.with_timezone(&Utc))
                        })
                        .transpose()
                        .map_err(|e| CiabError::Database(e.to_string()))?,
                };
                Ok(Some(credential))
            }
            None => Ok(None),
        }
    }

    pub async fn list_credentials(&self) -> CiabResult<Vec<CredentialSet>> {
        let rows: Vec<(String, String, String, String, String, Option<String>)> = sqlx::query_as(
            "SELECT id, name, credential_type, labels_json, created_at, expires_at FROM credentials ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CiabError::Database(e.to_string()))?;

        let mut results = Vec::new();
        for (id, name, cred_type, labels_json, created_at, expires_at) in rows {
            let cred = CredentialSet {
                id: id
                    .parse()
                    .map_err(|e: uuid::Error| CiabError::Database(e.to_string()))?,
                name,
                credential_type: serde_json::from_value(serde_json::Value::String(cred_type))?,
                labels: serde_json::from_str(&labels_json)?,
                created_at: chrono::DateTime::parse_from_rfc3339(&created_at)
                    .map_err(|e| CiabError::Database(e.to_string()))?
                    .with_timezone(&Utc),
                expires_at: expires_at
                    .map(|s| {
                        chrono::DateTime::parse_from_rfc3339(&s).map(|dt| dt.with_timezone(&Utc))
                    })
                    .transpose()
                    .map_err(|e| CiabError::Database(e.to_string()))?,
            };
            results.push(cred);
        }

        Ok(results)
    }

    pub async fn delete_credential(&self, id: &Uuid) -> CiabResult<()> {
        let id_str = id.to_string();

        sqlx::query("DELETE FROM credentials WHERE id = ?")
            .bind(&id_str)
            .execute(&self.pool)
            .await
            .map_err(|e| CiabError::Database(e.to_string()))?;

        Ok(())
    }

    pub async fn insert_oauth_token(
        &self,
        id: &Uuid,
        provider: &OAuthProvider,
        credential_id: &Uuid,
        access_token_enc: &[u8],
        refresh_token_enc: Option<&[u8]>,
        expires_at: Option<DateTime<Utc>>,
    ) -> CiabResult<()> {
        let id_str = id.to_string();
        let provider_str = serde_json::to_string(provider)?;
        let credential_id_str = credential_id.to_string();
        let expires_at_str = expires_at.map(|dt| dt.to_rfc3339());

        sqlx::query(
            "INSERT INTO oauth_tokens (id, provider, credential_id, access_token_enc, refresh_token_enc, expires_at)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&id_str)
        .bind(&provider_str)
        .bind(&credential_id_str)
        .bind(access_token_enc)
        .bind(refresh_token_enc)
        .bind(&expires_at_str)
        .execute(&self.pool)
        .await
        .map_err(|e| CiabError::Database(e.to_string()))?;

        Ok(())
    }

    pub async fn get_oauth_token(
        &self,
        credential_id: &Uuid,
    ) -> CiabResult<Option<(Vec<u8>, Option<Vec<u8>>, Option<DateTime<Utc>>)>> {
        let credential_id_str = credential_id.to_string();

        let row: Option<(Vec<u8>, Option<Vec<u8>>, Option<String>)> = sqlx::query_as(
            "SELECT access_token_enc, refresh_token_enc, expires_at FROM oauth_tokens WHERE credential_id = ?",
        )
        .bind(&credential_id_str)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| CiabError::Database(e.to_string()))?;

        match row {
            Some((access_token, refresh_token, expires_at)) => {
                let expires = expires_at
                    .map(|s| {
                        chrono::DateTime::parse_from_rfc3339(&s).map(|dt| dt.with_timezone(&Utc))
                    })
                    .transpose()
                    .map_err(|e| CiabError::Database(e.to_string()))?;
                Ok(Some((access_token, refresh_token, expires)))
            }
            None => Ok(None),
        }
    }
}
