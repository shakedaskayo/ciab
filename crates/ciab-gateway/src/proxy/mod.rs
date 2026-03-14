use uuid::Uuid;

use crate::types::TokenScope;

/// Extract sandbox ID from a path-based proxy URL (`/sandbox/<id>/...`).
pub fn extract_sandbox_id_from_path(path: &str) -> Option<Uuid> {
    let parts: Vec<&str> = path.trim_start_matches('/').splitn(3, '/').collect();
    if parts.len() >= 2 && parts[0] == "sandbox" {
        Uuid::parse_str(parts[1]).ok()
    } else {
        None
    }
}

/// Strip the `/sandbox/<id>` prefix from a path, returning the remainder.
pub fn strip_sandbox_prefix(path: &str) -> String {
    let parts: Vec<&str> = path.trim_start_matches('/').splitn(3, '/').collect();
    if parts.len() >= 3 && parts[0] == "sandbox" {
        format!("/{}", parts[2])
    } else if parts.len() == 2 && parts[0] == "sandbox" {
        "/".to_string()
    } else {
        path.to_string()
    }
}

/// Check if a set of scopes allows access to a sandbox.
pub fn check_sandbox_access(scopes: &[TokenScope], sandbox_id: &Uuid) -> bool {
    scopes.iter().any(|s| s.allows_sandbox(sandbox_id))
}

/// Check if a set of scopes allows write access.
pub fn check_write_access(scopes: &[TokenScope]) -> bool {
    scopes.iter().any(|s| s.allows_write())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_sandbox_id() {
        let id = Uuid::new_v4();
        let path = format!("/sandbox/{}/api/v1/sessions", id);
        assert_eq!(extract_sandbox_id_from_path(&path), Some(id));
    }

    #[test]
    fn test_strip_prefix() {
        let id = Uuid::new_v4();
        let path = format!("/sandbox/{}/api/v1/sessions", id);
        assert_eq!(strip_sandbox_prefix(&path), "/api/v1/sessions");
    }

    #[test]
    fn test_strip_prefix_no_trailing() {
        let id = Uuid::new_v4();
        let path = format!("/sandbox/{}", id);
        assert_eq!(strip_sandbox_prefix(&path), "/");
    }

    #[test]
    fn test_no_sandbox_prefix() {
        assert_eq!(extract_sandbox_id_from_path("/api/v1/health"), None);
    }
}
