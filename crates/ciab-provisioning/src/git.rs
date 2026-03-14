use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use ciab_core::error::{CiabError, CiabResult};
use ciab_core::traits::runtime::SandboxRuntime;
use ciab_core::types::sandbox::{ExecRequest, GitRepoSpec};
use uuid::Uuid;

/// Dispatch to clone_repo or create_worktree based on the repo strategy.
pub async fn provision_repo(
    runtime: &dyn SandboxRuntime,
    sandbox_id: &Uuid,
    repo: &GitRepoSpec,
) -> CiabResult<()> {
    let strategy = repo.strategy.as_deref().unwrap_or("clone");
    match strategy {
        "worktree" => create_worktree(runtime, sandbox_id, repo).await,
        _ => clone_repo(runtime, sandbox_id, repo).await,
    }
}

/// Create a git worktree from a shared bare clone.
///
/// 1. Compute stable base path from URL hash: `/tmp/ciab-worktree-bases/<hash>`
/// 2. If base doesn't exist: `git clone --bare <url> <base-path>`
/// 3. If base exists: `git fetch --all --prune` in base
/// 4. `git worktree add <dest_path> <branch>` from base
/// 5. If specific commit: `git checkout <commit>` in worktree
/// 6. Handle sparse checkout within worktree
pub async fn create_worktree(
    runtime: &dyn SandboxRuntime,
    sandbox_id: &Uuid,
    repo: &GitRepoSpec,
) -> CiabResult<()> {
    let repo_name = repo
        .url
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or("repo")
        .trim_end_matches(".git");

    let dest_path = repo
        .dest_path
        .clone()
        .unwrap_or_else(|| format!("/workspace/{}", repo_name));

    // Compute stable base path from URL hash
    let base_path = if let Some(ref base) = repo.worktree_base_path {
        base.clone()
    } else {
        let mut hasher = DefaultHasher::new();
        repo.url.hash(&mut hasher);
        let hash = hasher.finish();
        format!("/tmp/ciab-worktree-bases/{:x}", hash)
    };

    let mut env = HashMap::new();
    if repo.credential_id.is_some() {
        env.insert(
            "GIT_ASKPASS".to_string(),
            "/bin/sh -c 'echo $GIT_TOKEN'".to_string(),
        );
        env.insert("GIT_TERMINAL_PROMPT".to_string(), "0".to_string());
    }

    // Check if base bare clone exists
    let check_req = ExecRequest {
        command: vec!["test".to_string(), "-d".to_string(), base_path.clone()],
        workdir: None,
        env: env.clone(),
        stdin: None,
        timeout_secs: Some(10),
        tty: false,
    };
    let check_result = runtime.exec(sandbox_id, &check_req).await?;

    if check_result.exit_code != 0 {
        // Base doesn't exist — create bare clone
        let mkdir_req = ExecRequest {
            command: vec![
                "mkdir".to_string(),
                "-p".to_string(),
                "/tmp/ciab-worktree-bases".to_string(),
            ],
            workdir: None,
            env: env.clone(),
            stdin: None,
            timeout_secs: Some(10),
            tty: false,
        };
        runtime.exec(sandbox_id, &mkdir_req).await?;

        let clone_req = ExecRequest {
            command: vec![
                "git".to_string(),
                "clone".to_string(),
                "--bare".to_string(),
                repo.url.clone(),
                base_path.clone(),
            ],
            workdir: None,
            env: env.clone(),
            stdin: None,
            timeout_secs: Some(300),
            tty: false,
        };
        let clone_result = runtime.exec(sandbox_id, &clone_req).await?;
        if clone_result.exit_code != 0 {
            return Err(CiabError::GitWorktreeFailed(format!(
                "bare clone failed for {} (exit code {}): {}",
                repo.url, clone_result.exit_code, clone_result.stderr
            )));
        }
    } else {
        // Base exists — fetch latest
        let fetch_req = ExecRequest {
            command: vec![
                "git".to_string(),
                "fetch".to_string(),
                "--all".to_string(),
                "--prune".to_string(),
            ],
            workdir: Some(base_path.clone()),
            env: env.clone(),
            stdin: None,
            timeout_secs: Some(120),
            tty: false,
        };
        let fetch_result = runtime.exec(sandbox_id, &fetch_req).await?;
        if fetch_result.exit_code != 0 {
            tracing::warn!(
                repo_url = %repo.url,
                exit_code = fetch_result.exit_code,
                "git fetch in bare clone failed (non-fatal): {}",
                fetch_result.stderr
            );
        }
    }

    // Determine branch for worktree
    let branch = repo
        .tag
        .clone()
        .or_else(|| repo.branch.clone())
        .unwrap_or_else(|| "HEAD".to_string());

    // Create worktree
    let worktree_req = ExecRequest {
        command: vec![
            "git".to_string(),
            "worktree".to_string(),
            "add".to_string(),
            dest_path.clone(),
            branch.clone(),
        ],
        workdir: Some(base_path.clone()),
        env: env.clone(),
        stdin: None,
        timeout_secs: Some(120),
        tty: false,
    };
    let worktree_result = runtime.exec(sandbox_id, &worktree_req).await?;
    if worktree_result.exit_code != 0 {
        return Err(CiabError::GitWorktreeFailed(format!(
            "git worktree add failed for {} branch {} (exit code {}): {}",
            repo.url, branch, worktree_result.exit_code, worktree_result.stderr
        )));
    }

    // If a specific commit is requested, checkout in the worktree
    if let Some(ref commit) = repo.commit {
        let checkout_req = ExecRequest {
            command: vec!["git".to_string(), "checkout".to_string(), commit.clone()],
            workdir: Some(dest_path.clone()),
            env: env.clone(),
            stdin: None,
            timeout_secs: Some(120),
            tty: false,
        };
        let checkout_result = runtime.exec(sandbox_id, &checkout_req).await?;
        if checkout_result.exit_code != 0 {
            return Err(CiabError::GitWorktreeFailed(format!(
                "git checkout commit {} failed in worktree for {} (exit code {}): {}",
                commit, repo.url, checkout_result.exit_code, checkout_result.stderr
            )));
        }
    }

    // Handle sparse checkout within worktree
    if !repo.sparse_paths.is_empty() {
        let mut sparse_cmd = vec![
            "git".to_string(),
            "sparse-checkout".to_string(),
            "set".to_string(),
        ];
        sparse_cmd.extend(repo.sparse_paths.clone());

        let sparse_req = ExecRequest {
            command: sparse_cmd,
            workdir: Some(dest_path.clone()),
            env: env.clone(),
            stdin: None,
            timeout_secs: Some(120),
            tty: false,
        };
        let sparse_result = runtime.exec(sandbox_id, &sparse_req).await?;
        if sparse_result.exit_code != 0 {
            return Err(CiabError::GitWorktreeFailed(format!(
                "git sparse-checkout in worktree failed for {} (exit code {}): {}",
                repo.url, sparse_result.exit_code, sparse_result.stderr
            )));
        }
    }

    // Initialize submodules if requested
    if repo.submodules {
        let submodule_req = ExecRequest {
            command: vec![
                "git".to_string(),
                "submodule".to_string(),
                "update".to_string(),
                "--init".to_string(),
                "--recursive".to_string(),
            ],
            workdir: Some(dest_path.clone()),
            env: env.clone(),
            stdin: None,
            timeout_secs: Some(600),
            tty: false,
        };
        let submodule_result = runtime.exec(sandbox_id, &submodule_req).await?;
        if submodule_result.exit_code != 0 {
            return Err(CiabError::GitWorktreeFailed(format!(
                "git submodule update in worktree failed for {} (exit code {}): {}",
                repo.url, submodule_result.exit_code, submodule_result.stderr
            )));
        }
    }

    tracing::info!(
        repo_url = %repo.url,
        dest = %dest_path,
        base = %base_path,
        "created worktree successfully"
    );

    Ok(())
}

/// Clone a git repository inside a sandbox.
pub async fn clone_repo(
    runtime: &dyn SandboxRuntime,
    sandbox_id: &Uuid,
    repo: &GitRepoSpec,
) -> CiabResult<()> {
    // Extract repo name from URL for default dest_path
    let repo_name = repo
        .url
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or("repo")
        .trim_end_matches(".git");

    let dest_path = repo
        .dest_path
        .clone()
        .unwrap_or_else(|| format!("/workspace/{}", repo_name));

    let use_sparse = !repo.sparse_paths.is_empty();
    // If a specific commit is requested (without branch/tag), we need a different flow
    let use_commit_checkout = repo.commit.is_some() && repo.branch.is_none() && repo.tag.is_none();

    let mut env = HashMap::new();

    // If credential_id is set, configure git credential helper via GIT_ASKPASS
    if repo.credential_id.is_some() {
        env.insert(
            "GIT_ASKPASS".to_string(),
            "/bin/sh -c 'echo $GIT_TOKEN'".to_string(),
        );
        env.insert("GIT_TERMINAL_PROMPT".to_string(), "0".to_string());
    }

    // Build clone command
    let mut args = vec!["clone".to_string()];

    if let Some(depth) = repo.depth {
        args.push("--depth".to_string());
        args.push(depth.to_string());
    }

    if use_sparse {
        // Sparse checkout: clone without checking out files
        args.push("--no-checkout".to_string());
        args.push("--filter=blob:none".to_string());
    }

    // Tag takes precedence over branch (git clone --branch works with tags)
    if let Some(ref tag) = repo.tag {
        args.push("--branch".to_string());
        args.push(tag.clone());
    } else if let Some(ref branch) = repo.branch {
        if !use_commit_checkout {
            args.push("--branch".to_string());
            args.push(branch.clone());
        }
    }

    args.push(repo.url.clone());
    args.push(dest_path.clone());

    let mut command = vec!["git".to_string()];
    command.extend(args);

    let request = ExecRequest {
        command,
        workdir: None,
        env: env.clone(),
        stdin: None,
        timeout_secs: Some(300),
        tty: false,
    };

    let result = runtime.exec(sandbox_id, &request).await?;

    if result.exit_code != 0 {
        return Err(CiabError::GitCloneFailed(format!(
            "git clone failed for {} (exit code {}): {}",
            repo.url, result.exit_code, result.stderr
        )));
    }

    // Sparse checkout: set paths and checkout
    if use_sparse {
        let mut sparse_cmd = vec![
            "git".to_string(),
            "sparse-checkout".to_string(),
            "set".to_string(),
        ];
        sparse_cmd.extend(repo.sparse_paths.clone());

        let sparse_req = ExecRequest {
            command: sparse_cmd,
            workdir: Some(dest_path.clone()),
            env: env.clone(),
            stdin: None,
            timeout_secs: Some(120),
            tty: false,
        };
        let sparse_result = runtime.exec(sandbox_id, &sparse_req).await?;
        if sparse_result.exit_code != 0 {
            return Err(CiabError::GitCloneFailed(format!(
                "git sparse-checkout set failed for {} (exit code {}): {}",
                repo.url, sparse_result.exit_code, sparse_result.stderr
            )));
        }

        let checkout_req = ExecRequest {
            command: vec!["git".to_string(), "checkout".to_string()],
            workdir: Some(dest_path.clone()),
            env: env.clone(),
            stdin: None,
            timeout_secs: Some(120),
            tty: false,
        };
        let checkout_result = runtime.exec(sandbox_id, &checkout_req).await?;
        if checkout_result.exit_code != 0 {
            return Err(CiabError::GitCloneFailed(format!(
                "git checkout after sparse-checkout failed for {} (exit code {}): {}",
                repo.url, checkout_result.exit_code, checkout_result.stderr
            )));
        }
    }

    // If a specific commit is requested, checkout that commit
    if let Some(ref commit) = repo.commit {
        let checkout_req = ExecRequest {
            command: vec!["git".to_string(), "checkout".to_string(), commit.clone()],
            workdir: Some(dest_path.clone()),
            env: env.clone(),
            stdin: None,
            timeout_secs: Some(120),
            tty: false,
        };
        let checkout_result = runtime.exec(sandbox_id, &checkout_req).await?;
        if checkout_result.exit_code != 0 {
            return Err(CiabError::GitCloneFailed(format!(
                "git checkout commit {} failed for {} (exit code {}): {}",
                commit, repo.url, checkout_result.exit_code, checkout_result.stderr
            )));
        }
    }

    // Initialize and update submodules if requested
    if repo.submodules {
        let submodule_req = ExecRequest {
            command: vec![
                "git".to_string(),
                "submodule".to_string(),
                "update".to_string(),
                "--init".to_string(),
                "--recursive".to_string(),
            ],
            workdir: Some(dest_path.clone()),
            env: env.clone(),
            stdin: None,
            timeout_secs: Some(600),
            tty: false,
        };
        let submodule_result = runtime.exec(sandbox_id, &submodule_req).await?;
        if submodule_result.exit_code != 0 {
            return Err(CiabError::GitCloneFailed(format!(
                "git submodule update failed for {} (exit code {}): {}",
                repo.url, submodule_result.exit_code, submodule_result.stderr
            )));
        }
    }

    tracing::info!(
        repo_url = %repo.url,
        dest = %dest_path,
        "cloned repository successfully"
    );

    Ok(())
}
