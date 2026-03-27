use std::path::{Path, PathBuf};

use tracing::info;

use crate::error::{CiabError, CiabResult};

#[derive(Debug, Clone)]
pub enum ResourceSource {
    FilePath(PathBuf),
    Url(String),
    Git {
        url: String,
        path: String,
        ref_: Option<String>,
    },
    Builtin(String),
}

pub fn parse_source_string(s: &str) -> ResourceSource {
    if let Some(rest) = s.strip_prefix("git::") {
        parse_git_source(rest)
    } else if s.starts_with("http://") || s.starts_with("https://") {
        ResourceSource::Url(s.to_string())
    } else if let Some(name) = s.strip_prefix("builtin://") {
        ResourceSource::Builtin(name.to_string())
    } else {
        ResourceSource::FilePath(PathBuf::from(s))
    }
}

fn parse_git_source(s: &str) -> ResourceSource {
    let (url_and_path, ref_) = if let Some(idx) = s.find("?ref=") {
        (&s[..idx], Some(s[idx + 5..].to_string()))
    } else {
        (s, None)
    };

    // Skip past the protocol's "://" when searching for the "//" path separator.
    let search_start = if let Some(proto_end) = url_and_path.find("://") {
        proto_end + 3
    } else {
        0
    };

    let (url, path) = if let Some(rel_idx) = url_and_path[search_start..].find("//") {
        let idx = search_start + rel_idx;
        (
            url_and_path[..idx].to_string(),
            url_and_path[idx + 2..].to_string(),
        )
    } else {
        (url_and_path.to_string(), String::new())
    };

    ResourceSource::Git { url, path, ref_ }
}

pub async fn resolve_resource(source: &ResourceSource) -> CiabResult<String> {
    match source {
        ResourceSource::FilePath(path) => resolve_file(path).await,
        ResourceSource::Url(url) => resolve_url(url).await,
        ResourceSource::Git { url, path, ref_ } => {
            resolve_git(url, path, ref_.as_deref()).await
        }
        ResourceSource::Builtin(name) => resolve_builtin(name),
    }
}

async fn resolve_file(path: &Path) -> CiabResult<String> {
    tokio::fs::read_to_string(path).await.map_err(|e| {
        CiabError::ResourceResolutionError(format!(
            "Failed to read file {}: {}",
            path.display(),
            e
        ))
    })
}

async fn resolve_url(url: &str) -> CiabResult<String> {
    info!(url = url, "Fetching resource from URL");
    let response = reqwest::get(url).await.map_err(|e| {
        CiabError::ResourceResolutionError(format!("Failed to fetch {}: {}", url, e))
    })?;

    if !response.status().is_success() {
        return Err(CiabError::ResourceResolutionError(format!(
            "HTTP {} fetching {}",
            response.status(),
            url
        )));
    }

    response.text().await.map_err(|e| {
        CiabError::ResourceResolutionError(format!(
            "Failed to read response from {}: {}",
            url, e
        ))
    })
}

async fn resolve_git(url: &str, subpath: &str, ref_: Option<&str>) -> CiabResult<String> {
    let url = url.to_string();
    let subpath = subpath.to_string();
    let ref_ = ref_.map(|s| s.to_string());

    tokio::task::spawn_blocking(move || {
        let tmp = tempfile::tempdir().map_err(|e| {
            CiabError::ResourceResolutionError(format!("Failed to create temp dir: {}", e))
        })?;

        info!(url = %url, subpath = %subpath, ref_ = ?ref_, "Cloning git resource");

        let mut builder = git2::build::RepoBuilder::new();
        let mut fetch_opts = git2::FetchOptions::new();
        fetch_opts.depth(1);
        builder.fetch_options(fetch_opts);

        if let Some(ref branch) = ref_ {
            builder.branch(branch);
        }

        let repo = builder.clone(&url, tmp.path()).map_err(|e| {
            CiabError::ResourceResolutionError(format!("Git clone failed for {}: {}", url, e))
        })?;

        let file_path = if subpath.is_empty() {
            repo.workdir()
                .ok_or_else(|| {
                    CiabError::ResourceResolutionError("Bare repository".to_string())
                })?
                .to_path_buf()
        } else {
            repo.workdir()
                .ok_or_else(|| {
                    CiabError::ResourceResolutionError("Bare repository".to_string())
                })?
                .join(&subpath)
        };

        std::fs::read_to_string(&file_path).map_err(|e| {
            CiabError::ResourceResolutionError(format!(
                "Failed to read {} from cloned repo: {}",
                file_path.display(),
                e
            ))
        })
    })
    .await
    .map_err(|e| CiabError::ResourceResolutionError(format!("Git task panicked: {}", e)))?
}

fn resolve_builtin(name: &str) -> CiabResult<String> {
    match name {
        "default-ec2" => Ok(
            include_str!("../templates/default-ec2.pkr.hcl").to_string(),
        ),
        "default-config" => Ok(include_str!("../config.default.toml").to_string()),
        _ => Err(CiabError::ResourceResolutionError(format!(
            "Unknown builtin resource: {}",
            name
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_file_path() {
        let source = parse_source_string("/path/to/file.toml");
        assert!(
            matches!(source, ResourceSource::FilePath(p) if p == PathBuf::from("/path/to/file.toml"))
        );
    }

    #[test]
    fn test_parse_url() {
        let source = parse_source_string("https://example.com/config.toml");
        assert!(
            matches!(source, ResourceSource::Url(u) if u == "https://example.com/config.toml")
        );
    }

    #[test]
    fn test_parse_builtin() {
        let source = parse_source_string("builtin://default-ec2");
        assert!(matches!(source, ResourceSource::Builtin(n) if n == "default-ec2"));
    }

    #[test]
    fn test_parse_git_full() {
        let source = parse_source_string(
            "git::https://github.com/org/repo.git//path/to/file.hcl?ref=main",
        );
        match source {
            ResourceSource::Git { url, path, ref_ } => {
                assert_eq!(url, "https://github.com/org/repo.git");
                assert_eq!(path, "path/to/file.hcl");
                assert_eq!(ref_, Some("main".to_string()));
            }
            _ => panic!("Expected Git source"),
        }
    }

    #[test]
    fn test_parse_git_no_ref() {
        let source =
            parse_source_string("git::https://github.com/org/repo.git//template.hcl");
        match source {
            ResourceSource::Git { url, path, ref_ } => {
                assert_eq!(url, "https://github.com/org/repo.git");
                assert_eq!(path, "template.hcl");
                assert_eq!(ref_, None);
            }
            _ => panic!("Expected Git source"),
        }
    }

    #[test]
    fn test_parse_git_no_subpath() {
        let source =
            parse_source_string("git::https://github.com/org/repo.git?ref=v1.0");
        match source {
            ResourceSource::Git { url, path, ref_ } => {
                assert_eq!(url, "https://github.com/org/repo.git");
                assert_eq!(path, "");
                assert_eq!(ref_, Some("v1.0".to_string()));
            }
            _ => panic!("Expected Git source"),
        }
    }
}
