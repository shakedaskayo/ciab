use ciab_core::error::{CiabError, CiabResult};
use ciab_core::resolve::{parse_source_string, resolve_resource};
use ciab_core::types::config::PackerConfig;
use ciab_core::types::image::TemplateSource;
use std::path::PathBuf;

pub async fn resolve_template(
    source: &Option<TemplateSource>,
    config: &PackerConfig,
) -> CiabResult<String> {
    match source {
        Some(TemplateSource::Inline { content }) => Ok(content.clone()),
        Some(TemplateSource::FilePath { path }) => {
            tokio::fs::read_to_string(path).await.map_err(|e| {
                CiabError::PackerError(format!("Failed to read template {}: {}", path.display(), e))
            })
        }
        Some(TemplateSource::Url { url }) => {
            let src = parse_source_string(url);
            resolve_resource(&src).await
        }
        Some(TemplateSource::Git { url, path, ref_ }) => {
            let git_uri = format!(
                "git::{}//{}{}",
                url,
                path,
                ref_.as_ref()
                    .map(|r| format!("?ref={}", r))
                    .unwrap_or_default()
            );
            let src = parse_source_string(&git_uri);
            resolve_resource(&src).await
        }
        Some(TemplateSource::Builtin { name }) => {
            let src = parse_source_string(&format!("builtin://{}", name));
            resolve_resource(&src).await
        }
        None => {
            let src = parse_source_string(&config.default_template);
            resolve_resource(&src).await
        }
    }
}

pub async fn write_temp_template(content: &str) -> CiabResult<PathBuf> {
    let dir = tempfile::tempdir()
        .map_err(|e| CiabError::PackerError(format!("Failed to create temp dir: {}", e)))?;
    let path = dir.keep().join("template.pkr.hcl");
    tokio::fs::write(&path, content).await.map_err(|e| {
        CiabError::PackerError(format!("Failed to write temp template: {}", e))
    })?;
    Ok(path)
}
