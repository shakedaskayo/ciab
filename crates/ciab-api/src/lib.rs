pub mod middleware;
pub mod ollama;
pub mod permission_gate;
pub mod reconciler;
pub mod router;
pub mod routes;
pub mod state;

pub use router::build_router;
pub use state::AppState;

use std::net::SocketAddr;

use ciab_core::error::CiabResult;

/// Start the HTTP server with the given `AppState`.
pub async fn start_server(state: AppState) -> CiabResult<()> {
    let host = state.config.server.host.clone();
    let port = state.config.server.port;
    let addr: SocketAddr = format!("{}:{}", host, port)
        .parse()
        .map_err(|e| ciab_core::error::CiabError::ConfigError(format!("invalid address: {e}")))?;

    // Spawn background sandbox state reconciler (every 30s).
    reconciler::spawn_reconciler(state.db.clone(), state.runtime.clone(), 30);

    let router = build_router(state);

    tracing::info!(%addr, "starting ciab-api server");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| ciab_core::error::CiabError::Internal(format!("bind failed: {e}")))?;

    axum::serve(listener, router)
        .await
        .map_err(|e| ciab_core::error::CiabError::Internal(format!("server error: {e}")))?;

    Ok(())
}
