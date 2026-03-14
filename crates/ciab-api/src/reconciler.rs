//! Background sandbox state reconciler.
//!
//! Periodically checks all "active" sandboxes in the DB against the runtime
//! and marks any that no longer exist as terminated.

use std::sync::Arc;
use std::time::Duration;

use ciab_core::traits::runtime::SandboxRuntime;
use ciab_core::types::sandbox::SandboxState;
use ciab_db::Database;

/// Spawn the background reconciliation loop.
///
/// Runs every `interval_secs` seconds, checking all "active" sandboxes
/// against the runtime. Any sandbox that the runtime doesn't recognize
/// gets marked as `terminated` in the database.
pub fn spawn_reconciler(db: Arc<Database>, runtime: Arc<dyn SandboxRuntime>, interval_secs: u64) {
    let interval = Duration::from_secs(interval_secs);

    tokio::spawn(async move {
        // Wait a bit before the first reconciliation to let things settle.
        tokio::time::sleep(Duration::from_secs(10)).await;

        loop {
            if let Err(e) = reconcile_once(&db, &*runtime).await {
                tracing::warn!(error = %e, "sandbox reconciliation failed");
            }
            tokio::time::sleep(interval).await;
        }
    });
}

async fn reconcile_once(
    db: &Database,
    runtime: &dyn SandboxRuntime,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Fetch all "active" sandboxes from the DB.
    let filters = ciab_core::types::sandbox::SandboxFilters {
        state: None,
        provider: None,
        labels: Default::default(),
    };

    let sandboxes = db.list_sandboxes(&filters).await?;

    let mut reconciled = 0u32;

    for sandbox in &sandboxes {
        // Only reconcile sandboxes that are supposed to be "active".
        if !matches!(
            sandbox.state,
            SandboxState::Running
                | SandboxState::Creating
                | SandboxState::Pending
                | SandboxState::Paused
                | SandboxState::Pausing
        ) {
            continue;
        }

        // Try to fetch the sandbox from the runtime.
        match runtime.get_sandbox(&sandbox.id).await {
            Ok(runtime_info) => {
                // Runtime knows about it — check if state differs.
                if runtime_info.state != sandbox.state {
                    tracing::info!(
                        sandbox_id = %sandbox.id,
                        db_state = ?sandbox.state,
                        runtime_state = ?runtime_info.state,
                        "reconciling sandbox state"
                    );
                    let _ = db
                        .update_sandbox_state(&sandbox.id, &runtime_info.state)
                        .await;
                    reconciled += 1;
                }
            }
            Err(_) => {
                // Runtime doesn't know about this sandbox — mark as terminated.
                tracing::info!(
                    sandbox_id = %sandbox.id,
                    prev_state = ?sandbox.state,
                    "sandbox not found in runtime, marking as terminated"
                );
                let _ = db
                    .update_sandbox_state(&sandbox.id, &SandboxState::Terminated)
                    .await;
                reconciled += 1;
            }
        }
    }

    if reconciled > 0 {
        tracing::info!(count = reconciled, "reconciled sandbox states");
    }

    Ok(())
}
