use std::collections::HashMap;
use std::sync::Arc;

use aes_gcm::aead::rand_core::RngCore;
use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::Aes256Gcm;
use chrono::{DateTime, Utc};
use ciab_core::error::{CiabError, CiabResult};
use ciab_core::types::credentials::{CredentialSet, CredentialType, OAuthProvider, OAuthToken};
use uuid::Uuid;

use ciab_db::Database;

pub struct CredentialStore {
    db: Arc<Database>,
    encryption_key: [u8; 32],
}

impl CredentialStore {
    pub fn new(db: Arc<Database>, encryption_key_hex: &str) -> CiabResult<Self> {
        let key_bytes = Self::parse_key(encryption_key_hex)?;
        Ok(Self {
            db,
            encryption_key: key_bytes,
        })
    }

    fn parse_key(hex_str: &str) -> CiabResult<[u8; 32]> {
        let hex_str = hex_str.trim();
        // Try to decode as hex first
        let decoded: Vec<u8> = (0..hex_str.len())
            .step_by(2)
            .filter_map(|i| {
                hex_str
                    .get(i..i + 2)
                    .and_then(|byte_str| u8::from_str_radix(byte_str, 16).ok())
            })
            .collect();

        let mut key = [0u8; 32];
        if decoded.len() >= 32 {
            // Truncate to 32 bytes
            key.copy_from_slice(&decoded[..32]);
        } else if decoded.len() > 0 && decoded.len() == hex_str.len() / 2 {
            // Valid hex but shorter than 32 bytes: hash it by XOR-folding
            // Simple approach: repeat the bytes to fill 32 bytes
            for (i, &b) in decoded.iter().cycle().take(32).enumerate() {
                key[i] = b;
            }
            // Mix in length to differentiate keys of different lengths
            key[31] ^= decoded.len() as u8;
        } else {
            // Not valid hex, treat as raw passphrase: simple hash by folding
            let bytes = hex_str.as_bytes();
            for (i, &b) in bytes.iter().cycle().take(32).enumerate() {
                key[i] = b;
            }
            // Additional mixing pass
            for i in 1..32 {
                key[i] = key[i].wrapping_add(key[i - 1]).wrapping_mul(31);
            }
        }
        Ok(key)
    }

    pub fn encrypt(&self, plaintext: &[u8]) -> CiabResult<Vec<u8>> {
        let cipher = Aes256Gcm::new_from_slice(&self.encryption_key)
            .map_err(|e| CiabError::Internal(format!("cipher init failed: {e}")))?;

        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = aes_gcm::Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| CiabError::Internal(format!("encryption failed: {e}")))?;

        let mut result = Vec::with_capacity(12 + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }

    pub fn decrypt(&self, ciphertext: &[u8]) -> CiabResult<Vec<u8>> {
        if ciphertext.len() < 12 {
            return Err(CiabError::DecryptionFailed(
                "ciphertext too short".to_string(),
            ));
        }

        let cipher = Aes256Gcm::new_from_slice(&self.encryption_key)
            .map_err(|e| CiabError::Internal(format!("cipher init failed: {e}")))?;

        let nonce = aes_gcm::Nonce::from_slice(&ciphertext[..12]);
        let plaintext = cipher
            .decrypt(nonce, &ciphertext[12..])
            .map_err(|e| CiabError::DecryptionFailed(format!("decryption failed: {e}")))?;

        Ok(plaintext)
    }

    pub async fn store_credential(
        &self,
        name: &str,
        cred_type: CredentialType,
        data: &[u8],
        labels: HashMap<String, String>,
        expires_at: Option<DateTime<Utc>>,
    ) -> CiabResult<CredentialSet> {
        let id = Uuid::new_v4();
        let encrypted_data = self.encrypt(data)?;

        self.db
            .insert_credential(&id, name, &cred_type, &encrypted_data, &labels, expires_at)
            .await?;

        Ok(CredentialSet {
            id,
            name: name.to_string(),
            credential_type: cred_type,
            labels,
            created_at: Utc::now(),
            expires_at,
        })
    }

    pub async fn get_credential(&self, id: &Uuid) -> CiabResult<(CredentialSet, Vec<u8>)> {
        let row = self
            .db
            .get_credential(id)
            .await?
            .ok_or_else(|| CiabError::CredentialNotFound(id.to_string()))?;

        let plaintext = self.decrypt(&row.encrypted_data)?;

        let cred_set = CredentialSet {
            id: row.id,
            name: row.name,
            credential_type: row.credential_type,
            labels: row.labels,
            created_at: row.created_at,
            expires_at: row.expires_at,
        };

        Ok((cred_set, plaintext))
    }

    pub async fn list_credentials(&self) -> CiabResult<Vec<CredentialSet>> {
        self.db.list_credentials().await
    }

    pub async fn delete_credential(&self, id: &Uuid) -> CiabResult<()> {
        self.db.delete_credential(id).await
    }

    pub async fn store_oauth_token(
        &self,
        provider: &str,
        credential_id: &Uuid,
        token: &OAuthToken,
    ) -> CiabResult<()> {
        let access_token_enc = self.encrypt(token.access_token.as_bytes())?;
        let refresh_token_enc = match &token.refresh_token {
            Some(rt) => Some(self.encrypt(rt.as_bytes())?),
            None => None,
        };

        let oauth_provider: OAuthProvider =
            serde_json::from_value(serde_json::Value::String(provider.to_string()))
                .unwrap_or(OAuthProvider::Custom(provider.to_string()));

        let id = Uuid::new_v4();
        self.db
            .insert_oauth_token(
                &id,
                &oauth_provider,
                credential_id,
                &access_token_enc,
                refresh_token_enc.as_deref(),
                token.expires_at,
            )
            .await
    }

    pub async fn get_oauth_token(&self, provider: &str) -> CiabResult<Option<OAuthToken>> {
        // We need a credential_id to look up the token. The provider string is used
        // to find the credential by convention: look for a credential with name matching
        // the provider, then look up the oauth token by credential_id.
        let credentials = self.db.list_credentials().await?;
        let cred = credentials.iter().find(|c| {
            c.credential_type == CredentialType::OAuthToken
                && (c.name.to_lowercase() == provider.to_lowercase()
                    || c.labels
                        .get("provider")
                        .map(|p| p.to_lowercase() == provider.to_lowercase())
                        .unwrap_or(false))
        });

        let cred = match cred {
            Some(c) => c,
            None => return Ok(None),
        };

        let row = self.db.get_oauth_token(&cred.id).await?;

        match row {
            Some((access_token_enc, refresh_token_enc, expires_at)) => {
                let access_token = String::from_utf8(self.decrypt(&access_token_enc)?)
                    .map_err(|e| CiabError::DecryptionFailed(e.to_string()))?;
                let refresh_token = match refresh_token_enc {
                    Some(enc) => {
                        let rt = String::from_utf8(self.decrypt(&enc)?)
                            .map_err(|e| CiabError::DecryptionFailed(e.to_string()))?;
                        Some(rt)
                    }
                    None => None,
                };
                Ok(Some(OAuthToken {
                    access_token,
                    refresh_token,
                    token_type: "bearer".to_string(),
                    expires_at,
                    scopes: Vec::new(),
                }))
            }
            None => Ok(None),
        }
    }
}
