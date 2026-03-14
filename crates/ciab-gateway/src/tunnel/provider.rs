use std::path::{Path, PathBuf};
use std::process::Stdio;

use ciab_core::error::{CiabError, CiabResult};

use crate::types::ProviderPrepareResult;

/// Check if a binary is available on PATH or at the given path.
pub fn find_binary(name: &str) -> Option<PathBuf> {
    // Check if it's an absolute path
    let path = PathBuf::from(name);
    if path.is_absolute() && path.exists() {
        return Some(path);
    }

    // Check PATH
    which::which(name).ok()
}

/// Get the version string from a binary.
pub async fn get_binary_version(binary: &str, version_flag: &str) -> Option<String> {
    let output = tokio::process::Command::new(binary)
        .arg(version_flag)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = if stdout.trim().is_empty() {
        stderr.to_string()
    } else {
        stdout.to_string()
    };

    // Extract version-like string from first line
    let first_line = combined.lines().next().unwrap_or("").trim();
    if first_line.is_empty() {
        None
    } else {
        Some(first_line.to_string())
    }
}

/// Install a binary using cargo install.
pub async fn cargo_install(crate_name: &str) -> CiabResult<PathBuf> {
    tracing::info!(crate_name, "Installing via cargo install");

    let output = tokio::process::Command::new("cargo")
        .args(["install", crate_name])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| {
            CiabError::TunnelProviderError(format!(
                "cargo install {} failed to start: {}. Is cargo installed?",
                crate_name, e
            ))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CiabError::TunnelProviderError(format!(
            "cargo install {} failed: {}",
            crate_name, stderr
        )));
    }

    find_binary(crate_name).ok_or_else(|| {
        CiabError::TunnelProviderError(format!("{} installed but not found on PATH", crate_name))
    })
}

/// Download a binary from a URL using curl or platform-appropriate method.
async fn download_file(url: &str, dest: &Path) -> CiabResult<()> {
    let output = tokio::process::Command::new("curl")
        .args(["-fsSL", "-o"])
        .arg(dest.as_os_str())
        .arg(url)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| CiabError::TunnelProviderError(format!("curl download failed: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CiabError::TunnelProviderError(format!(
            "Download from {} failed: {}",
            url, stderr
        )));
    }

    Ok(())
}

/// Install cloudflared via its official install script/package.
pub async fn install_cloudflared() -> CiabResult<PathBuf> {
    tracing::info!("Installing cloudflared");

    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    let (url, filename) = match (os, arch) {
        ("macos", "aarch64") => (
            "https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-darwin-arm64.tgz",
            "cloudflared-darwin-arm64.tgz",
        ),
        ("macos", "x86_64") => (
            "https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-darwin-amd64.tgz",
            "cloudflared-darwin-amd64.tgz",
        ),
        ("linux", "x86_64") => (
            "https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-amd64",
            "cloudflared-linux-amd64",
        ),
        ("linux", "aarch64") => (
            "https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-arm64",
            "cloudflared-linux-arm64",
        ),
        _ => {
            return Err(CiabError::TunnelProviderError(format!(
                "Unsupported platform for cloudflared auto-install: {}/{}",
                os, arch
            )));
        }
    };

    let install_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("ciab")
        .join("bin");
    tokio::fs::create_dir_all(&install_dir).await.map_err(|e| {
        CiabError::TunnelProviderError(format!("Failed to create install dir: {}", e))
    })?;

    let download_path = install_dir.join(filename);
    download_file(url, &download_path).await?;

    let binary_path = install_dir.join("cloudflared");

    if filename.ends_with(".tgz") {
        // Extract tarball
        let output = tokio::process::Command::new("tar")
            .args(["xzf"])
            .arg(&download_path)
            .arg("-C")
            .arg(&install_dir)
            .output()
            .await
            .map_err(|e| CiabError::TunnelProviderError(format!("tar extract failed: {}", e)))?;
        if !output.status.success() {
            return Err(CiabError::TunnelProviderError(
                "Failed to extract cloudflared tarball".to_string(),
            ));
        }
        let _ = tokio::fs::remove_file(&download_path).await;
    } else {
        // Direct binary download
        tokio::fs::rename(&download_path, &binary_path)
            .await
            .map_err(|e| CiabError::TunnelProviderError(format!("Failed to move binary: {}", e)))?;
    }

    // Make executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o755);
        tokio::fs::set_permissions(&binary_path, perms)
            .await
            .map_err(|e| CiabError::TunnelProviderError(format!("Failed to chmod: {}", e)))?;
    }

    Ok(binary_path)
}

/// Install ngrok via direct binary download.
pub async fn install_ngrok() -> CiabResult<PathBuf> {
    tracing::info!("Installing ngrok");

    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    let platform = match (os, arch) {
        ("macos", "aarch64") => "darwin-arm64",
        ("macos", "x86_64") => "darwin-amd64",
        ("linux", "x86_64") => "linux-amd64",
        ("linux", "aarch64") => "linux-arm64",
        _ => {
            return Err(CiabError::TunnelProviderError(format!(
                "Unsupported platform for ngrok auto-install: {}/{}",
                os, arch
            )));
        }
    };

    let install_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("ciab")
        .join("bin");
    tokio::fs::create_dir_all(&install_dir).await.map_err(|e| {
        CiabError::TunnelProviderError(format!("Failed to create install dir: {}", e))
    })?;

    let archive_name = format!("ngrok-v3-stable-{}.zip", platform);
    let url = format!(
        "https://bin.equinox.io/c/bNyj1mQVY4c/ngrok-v3-stable-{}.zip",
        platform
    );
    let archive_path = install_dir.join(&archive_name);

    download_file(&url, &archive_path).await?;

    // Extract zip
    let output = tokio::process::Command::new("unzip")
        .args(["-o"])
        .arg(&archive_path)
        .arg("-d")
        .arg(&install_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| CiabError::TunnelProviderError(format!("unzip failed: {}", e)))?;

    if !output.status.success() {
        return Err(CiabError::TunnelProviderError(
            "Failed to extract ngrok archive".to_string(),
        ));
    }

    let _ = tokio::fs::remove_file(&archive_path).await;

    let binary_path = install_dir.join("ngrok");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o755);
        let _ = tokio::fs::set_permissions(&binary_path, perms).await;
    }

    Ok(binary_path)
}

/// Install bore via cargo install.
pub async fn install_bore() -> CiabResult<PathBuf> {
    cargo_install("bore-cli").await
}

/// Prepare (find or install) a tunnel provider binary.
pub async fn prepare_provider(
    provider: &str,
    binary_hint: &str,
    auto_install: bool,
) -> CiabResult<ProviderPrepareResult> {
    // First check if binary is already available
    if let Some(path) = find_binary(binary_hint) {
        let version = get_binary_version(path.to_str().unwrap_or(binary_hint), "--version").await;
        return Ok(ProviderPrepareResult {
            provider: provider.to_string(),
            installed: true,
            binary_path: path.to_string_lossy().to_string(),
            version,
            message: format!("{} is already installed", provider),
        });
    }

    // Also check in our install directory
    let install_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("ciab")
        .join("bin");
    let local_binary = install_dir.join(binary_hint);
    if local_binary.exists() {
        let version =
            get_binary_version(local_binary.to_str().unwrap_or(binary_hint), "--version").await;
        return Ok(ProviderPrepareResult {
            provider: provider.to_string(),
            installed: true,
            binary_path: local_binary.to_string_lossy().to_string(),
            version,
            message: format!("{} found in local install directory", provider),
        });
    }

    if !auto_install {
        return Err(CiabError::TunnelProviderNotReady(format!(
            "{} binary '{}' not found and auto_install is disabled",
            provider, binary_hint
        )));
    }

    // Auto-install
    tracing::info!(provider, "Auto-installing tunnel provider");
    let installed_path = match provider {
        "bore" => install_bore().await?,
        "cloudflare" => install_cloudflared().await?,
        "ngrok" => install_ngrok().await?,
        other => {
            return Err(CiabError::TunnelProviderError(format!(
                "No auto-installer for provider: {}",
                other
            )));
        }
    };

    let version = get_binary_version(installed_path.to_str().unwrap_or(""), "--version").await;

    Ok(ProviderPrepareResult {
        provider: provider.to_string(),
        installed: true,
        binary_path: installed_path.to_string_lossy().to_string(),
        version,
        message: format!("{} installed successfully", provider),
    })
}
