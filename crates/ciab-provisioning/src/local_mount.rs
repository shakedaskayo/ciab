use std::path::Path;

use ciab_core::error::{CiabError, CiabResult};
use ciab_core::traits::runtime::SandboxRuntime;
use ciab_core::types::sandbox::LocalMountSpec;
use uuid::Uuid;

/// Mount local directories into a sandbox based on the sync mode.
pub async fn mount_local_dir(
    runtime: &dyn SandboxRuntime,
    sandbox_id: &Uuid,
    mount: &LocalMountSpec,
) -> CiabResult<()> {
    let source = Path::new(&mount.source);

    if !source.exists() {
        return Err(CiabError::LocalMountFailed(format!(
            "source path does not exist: {}",
            mount.source
        )));
    }

    match mount.sync_mode.as_str() {
        "copy" => copy_directory(runtime, sandbox_id, mount).await,
        "link" => create_symlink(runtime, sandbox_id, mount).await,
        "bind" => setup_bind_mount(runtime, sandbox_id, mount).await,
        other => Err(CiabError::LocalMountFailed(format!(
            "unknown sync mode: {}",
            other
        ))),
    }
}

/// Copy directory contents into the sandbox, respecting exclude patterns.
async fn copy_directory(
    runtime: &dyn SandboxRuntime,
    sandbox_id: &Uuid,
    mount: &LocalMountSpec,
) -> CiabResult<()> {
    // Create destination directory
    let mkdir_req = ciab_core::types::sandbox::ExecRequest {
        command: vec![
            "mkdir".to_string(),
            "-p".to_string(),
            mount.dest_path.clone(),
        ],
        workdir: None,
        env: std::collections::HashMap::new(),
        stdin: None,
        timeout_secs: Some(30),
        tty: false,
    };
    runtime.exec(sandbox_id, &mkdir_req).await?;

    // Build rsync/cp command with exclusions
    let mut command = vec![
        "rsync".to_string(),
        "-a".to_string(),
        "--delete".to_string(),
    ];

    for pattern in &mount.exclude_patterns {
        command.push("--exclude".to_string());
        command.push(pattern.clone());
    }

    // Ensure source ends with / for rsync to copy contents
    let mut source = mount.source.clone();
    if !source.ends_with('/') {
        source.push('/');
    }
    command.push(source);
    command.push(mount.dest_path.clone());

    let request = ciab_core::types::sandbox::ExecRequest {
        command,
        workdir: None,
        env: std::collections::HashMap::new(),
        stdin: None,
        timeout_secs: Some(600),
        tty: false,
    };

    let result = runtime.exec(sandbox_id, &request).await?;
    if result.exit_code != 0 {
        return Err(CiabError::LocalMountFailed(format!(
            "rsync failed for {} -> {} (exit {}): {}",
            mount.source, mount.dest_path, result.exit_code, result.stderr
        )));
    }

    tracing::info!(
        source = %mount.source,
        dest = %mount.dest_path,
        "copied local directory into sandbox"
    );

    Ok(())
}

/// Create a symlink from dest to source (changes affect original).
async fn create_symlink(
    runtime: &dyn SandboxRuntime,
    sandbox_id: &Uuid,
    mount: &LocalMountSpec,
) -> CiabResult<()> {
    // Ensure parent directory exists
    let parent = Path::new(&mount.dest_path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "/workspace".to_string());

    let mkdir_req = ciab_core::types::sandbox::ExecRequest {
        command: vec!["mkdir".to_string(), "-p".to_string(), parent],
        workdir: None,
        env: std::collections::HashMap::new(),
        stdin: None,
        timeout_secs: Some(30),
        tty: false,
    };
    runtime.exec(sandbox_id, &mkdir_req).await?;

    let request = ciab_core::types::sandbox::ExecRequest {
        command: vec![
            "ln".to_string(),
            "-sfn".to_string(),
            mount.source.clone(),
            mount.dest_path.clone(),
        ],
        workdir: None,
        env: std::collections::HashMap::new(),
        stdin: None,
        timeout_secs: Some(30),
        tty: false,
    };

    let result = runtime.exec(sandbox_id, &request).await?;
    if result.exit_code != 0 {
        return Err(CiabError::LocalMountFailed(format!(
            "symlink failed for {} -> {} (exit {}): {}",
            mount.source, mount.dest_path, result.exit_code, result.stderr
        )));
    }

    tracing::info!(
        source = %mount.source,
        dest = %mount.dest_path,
        "created symlink for local directory"
    );

    Ok(())
}

/// Set up bind mount (for Docker/container runtimes).
/// This records the intent; the actual bind mount is handled by the runtime.
async fn setup_bind_mount(
    _runtime: &dyn SandboxRuntime,
    _sandbox_id: &Uuid,
    mount: &LocalMountSpec,
) -> CiabResult<()> {
    // Bind mounts are configured at sandbox creation time via volumes,
    // not at provisioning time. Log that this was requested.
    tracing::info!(
        source = %mount.source,
        dest = %mount.dest_path,
        "bind mount requested — should be configured as volume mount at sandbox creation"
    );
    Ok(())
}
