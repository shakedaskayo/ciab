use ciab_core::error::{CiabError, CiabResult};
use ciab_core::traits::runtime::SandboxRuntime;
use ciab_core::types::sandbox::ExecRequest;
use uuid::Uuid;

/// Check if the agentfs binary is available in the sandbox.
pub async fn check_agentfs_available(
    runtime: &dyn SandboxRuntime,
    sandbox_id: &Uuid,
    binary: &str,
) -> CiabResult<bool> {
    let req = ExecRequest {
        command: vec!["which".to_string(), binary.to_string()],
        workdir: None,
        env: Default::default(),
        stdin: None,
        timeout_secs: Some(10),
        tty: false,
    };
    match runtime.exec(sandbox_id, &req).await {
        Ok(result) => Ok(result.exit_code == 0),
        Err(_) => Ok(false),
    }
}

/// Initialize an AgentFS SQLite database for a session.
pub async fn init_agentfs_db(
    runtime: &dyn SandboxRuntime,
    sandbox_id: &Uuid,
    binary: &str,
    db_path: &str,
) -> CiabResult<()> {
    // Ensure parent directory exists
    if let Some(parent) = std::path::Path::new(db_path).parent() {
        let mkdir_req = ExecRequest {
            command: vec![
                "mkdir".to_string(),
                "-p".to_string(),
                parent.to_string_lossy().to_string(),
            ],
            workdir: None,
            env: Default::default(),
            stdin: None,
            timeout_secs: Some(10),
            tty: false,
        };
        runtime.exec(sandbox_id, &mkdir_req).await?;
    }

    let req = ExecRequest {
        command: vec![
            binary.to_string(),
            "init".to_string(),
            "--db".to_string(),
            db_path.to_string(),
        ],
        workdir: None,
        env: Default::default(),
        stdin: None,
        timeout_secs: Some(30),
        tty: false,
    };
    let result = runtime.exec(sandbox_id, &req).await?;
    if result.exit_code != 0 {
        return Err(CiabError::AgentFsError(format!(
            "agentfs init failed (exit code {}): {}",
            result.exit_code, result.stderr
        )));
    }

    tracing::info!(
        sandbox_id = %sandbox_id,
        db_path = %db_path,
        "agentfs database initialized"
    );

    Ok(())
}

/// Wrap an agent command with agentfs for CoW isolation.
///
/// Transforms `["claude", "--arg"]` into `["agentfs", "run", "--db", "<db_path>", "--", "claude", "--arg"]`.
pub fn wrap_command_with_agentfs(
    command: &str,
    args: &[String],
    binary: &str,
    db_path: &str,
    operation_logging: bool,
) -> (String, Vec<String>) {
    let mut agentfs_args = vec!["run".to_string(), "--db".to_string(), db_path.to_string()];

    if operation_logging {
        agentfs_args.push("--log-ops".to_string());
    }

    agentfs_args.push("--".to_string());
    agentfs_args.push(command.to_string());
    agentfs_args.extend(args.iter().cloned());

    (binary.to_string(), agentfs_args)
}
