use std::collections::HashMap;

use ciab_core::error::{CiabError, CiabResult};
use ciab_core::types::credentials::CredentialType;
use uuid::Uuid;

use crate::vault::CredentialStore;

pub struct CredentialInjector;

impl CredentialInjector {
    /// For each credential ID, fetch and decrypt. Based on type, produce env vars.
    /// - ApiKey: return as single env var (name from credential name)
    /// - EnvVars: deserialize decrypted data as HashMap<String,String>, merge all
    /// - GitToken: set GIT_TOKEN env var
    /// - OAuthToken: set access_token as env var
    /// - SshKey/File: skip (handled by resolve_files)
    pub async fn resolve_env_vars(
        store: &CredentialStore,
        credential_ids: &[String],
    ) -> CiabResult<HashMap<String, String>> {
        let mut env_vars = HashMap::new();

        for id_str in credential_ids {
            let id = id_str
                .parse::<Uuid>()
                .map_err(|e| CiabError::Internal(format!("invalid credential id: {e}")))?;

            let (cred_set, plaintext) = store.get_credential(&id).await?;

            match cred_set.credential_type {
                CredentialType::ApiKey => {
                    let value = String::from_utf8(plaintext)
                        .map_err(|e| CiabError::Internal(format!("invalid utf8: {e}")))?;
                    // Use the credential name as the env var key (uppercased)
                    let env_key = cred_set.name.to_uppercase().replace(['-', ' '], "_");
                    env_vars.insert(env_key, value);
                }
                CredentialType::EnvVars => {
                    let vars: HashMap<String, String> = serde_json::from_slice(&plaintext)?;
                    env_vars.extend(vars);
                }
                CredentialType::GitToken => {
                    let value = String::from_utf8(plaintext)
                        .map_err(|e| CiabError::Internal(format!("invalid utf8: {e}")))?;
                    env_vars.insert("GIT_TOKEN".to_string(), value);
                }
                CredentialType::OAuthToken => {
                    let token: ciab_core::types::credentials::OAuthToken =
                        serde_json::from_slice(&plaintext)?;
                    let env_key = format!(
                        "{}_ACCESS_TOKEN",
                        cred_set.name.to_uppercase().replace(['-', ' '], "_")
                    );
                    env_vars.insert(env_key, token.access_token);
                }
                CredentialType::SshKey | CredentialType::File => {
                    // Handled by resolve_files
                }
            }
        }

        Ok(env_vars)
    }

    /// For SshKey/File types, return (path, content) pairs for upload to sandbox.
    pub async fn resolve_files(
        store: &CredentialStore,
        credential_ids: &[String],
    ) -> CiabResult<Vec<(String, Vec<u8>)>> {
        let mut files = Vec::new();

        for id_str in credential_ids {
            let id = id_str
                .parse::<Uuid>()
                .map_err(|e| CiabError::Internal(format!("invalid credential id: {e}")))?;

            let (cred_set, plaintext) = store.get_credential(&id).await?;

            match cred_set.credential_type {
                CredentialType::SshKey => {
                    let path = cred_set
                        .labels
                        .get("path")
                        .cloned()
                        .unwrap_or_else(|| format!("/root/.ssh/{}", cred_set.name));
                    files.push((path, plaintext));
                }
                CredentialType::File => {
                    let path = cred_set
                        .labels
                        .get("path")
                        .cloned()
                        .unwrap_or_else(|| format!("/tmp/{}", cred_set.name));
                    files.push((path, plaintext));
                }
                _ => {
                    // Not a file type, skip
                }
            }
        }

        Ok(files)
    }
}
