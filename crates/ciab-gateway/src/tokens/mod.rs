use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use rand::RngCore;
use sha2::{Digest, Sha256};

/// Generate a new client token string: `ciab_<base64url(32 random bytes)>`.
pub fn generate_token() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    let encoded = URL_SAFE_NO_PAD.encode(bytes);
    format!("ciab_{}", encoded)
}

/// Compute SHA-256 hash of a raw token, returned as hex string.
pub fn hash_token(raw_token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw_token.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

/// Tiny re-export of hex encoding (inline implementation to avoid extra dep).
mod hex {
    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        bytes
            .as_ref()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_format() {
        let token = generate_token();
        assert!(token.starts_with("ciab_"));
        // base64url of 32 bytes = 43 chars, plus "ciab_" prefix = 48
        assert_eq!(token.len(), 48);
    }

    #[test]
    fn hash_deterministic() {
        let token = "ciab_test_token_value";
        let h1 = hash_token(token);
        let h2 = hash_token(token);
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64); // SHA-256 hex = 64 chars
    }

    #[test]
    fn different_tokens_different_hashes() {
        let t1 = generate_token();
        let t2 = generate_token();
        assert_ne!(hash_token(&t1), hash_token(&t2));
    }
}
