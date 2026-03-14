use std::time::Duration;

use axum::routing::{get, post};
use tower_http::cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
use tower_http::timeout::TimeoutLayer;

use crate::middleware::auth::auth_middleware;
use crate::routes;
use crate::state::AppState;

/// Build the application router.
///
/// Architecture:
/// - Public routes (`/health`, `/ready`) — no auth, no timeout.
/// - All `/api/v1/*` routes — auth enforced via middleware layer, request timeout applied.
/// - CORS is configured from `config.server.cors_origins`; falls back to
///   permissive when the list is empty (dev mode).
/// - Auth supports `Authorization: Bearer`, `X-API-Key` header, and `?token=`
///   query param (for EventSource/SSE which cannot set custom headers).
pub fn build_router(state: AppState) -> axum::Router {
    let cors = build_cors_layer(&state);

    let timeout = TimeoutLayer::with_status_code(
        axum::http::StatusCode::GATEWAY_TIMEOUT,
        Duration::from_secs(state.config.server.request_timeout_secs),
    );

    // -- Public (no auth) --
    // Hook endpoints are public because Claude Code's HTTP hooks cannot easily
    // pass custom auth headers. Security is provided by the session UUID being
    // unguessable and the hook only being called by the local agent process.
    let public_routes = axum::Router::new()
        .route("/health", get(routes::health::health))
        .route("/ready", get(routes::health::ready))
        .route(
            "/api/v1/channels/webhook/{id}/inbound",
            post(routes::channels::webhook_inbound),
        )
        .route(
            "/api/v1/hooks/claude/{session_id}",
            post(routes::hooks::claude_hook),
        );

    // -- Authenticated API routes --
    let api_routes = axum::Router::new()
        // Sandboxes
        .route(
            "/sandboxes",
            post(routes::sandboxes::create_sandbox).get(routes::sandboxes::list_sandboxes),
        )
        .route(
            "/sandboxes/{id}",
            get(routes::sandboxes::get_sandbox).delete(routes::sandboxes::delete_sandbox),
        )
        .route(
            "/sandboxes/{id}/start",
            post(routes::sandboxes::start_sandbox),
        )
        .route(
            "/sandboxes/{id}/stop",
            post(routes::sandboxes::stop_sandbox),
        )
        .route(
            "/sandboxes/{id}/pause",
            post(routes::sandboxes::pause_sandbox),
        )
        .route(
            "/sandboxes/{id}/resume",
            post(routes::sandboxes::resume_sandbox),
        )
        .route(
            "/sandboxes/{id}/stats",
            get(routes::sandboxes::sandbox_stats),
        )
        .route("/sandboxes/{id}/logs", get(routes::sandboxes::sandbox_logs))
        .route(
            "/sandboxes/{id}/stream",
            get(routes::sandboxes::sandbox_stream),
        )
        // Sessions
        .route(
            "/sandboxes/{id}/sessions",
            post(routes::sessions::create_session).get(routes::sessions::list_sessions),
        )
        .route("/sessions/{sid}", get(routes::sessions::get_session))
        .route(
            "/sessions/{sid}/messages",
            post(routes::sessions::send_message),
        )
        .route(
            "/sessions/{sid}/interrupt",
            post(routes::sessions::interrupt_session),
        )
        .route(
            "/sessions/{sid}/stream",
            get(routes::sessions::session_stream),
        )
        .route(
            "/sessions/{sid}/skills",
            post(routes::sessions::update_session_skills),
        )
        .route("/sessions/{sid}/queue", get(routes::sessions::get_queue))
        .route(
            "/sessions/{sid}/queue/{msg_id}",
            axum::routing::delete(routes::sessions::cancel_queued_message),
        )
        .route(
            "/sessions/{sid}/permissions",
            post(routes::permissions::set_permission_mode),
        )
        .route(
            "/sessions/{sid}/permissions/{rid}/respond",
            post(routes::permissions::respond_to_permission),
        )
        .route(
            "/sessions/{sid}/user-input/{rid}/respond",
            post(routes::permissions::respond_to_user_input),
        )
        // Exec & Files
        .route("/sandboxes/{id}/exec", post(routes::exec::exec_command))
        .route(
            "/sandboxes/{id}/exec/stream",
            post(routes::exec::exec_stream),
        )
        .route("/sandboxes/{id}/files", get(routes::files::list_files))
        .route(
            "/sandboxes/{id}/files/{*path}",
            get(routes::files::download_file)
                .put(routes::files::upload_file)
                .delete(routes::files::delete_file),
        )
        // Credentials
        .route(
            "/credentials",
            post(routes::credentials::create_credential).get(routes::credentials::list_credentials),
        )
        .route(
            "/credentials/{id}",
            get(routes::credentials::get_credential).delete(routes::credentials::delete_credential),
        )
        // OAuth
        .route("/oauth/{provider}/authorize", get(routes::oauth::authorize))
        .route("/oauth/{provider}/callback", get(routes::oauth::callback))
        .route(
            "/oauth/{provider}/device-code",
            get(routes::oauth::device_code),
        )
        .route(
            "/oauth/{provider}/device-poll",
            post(routes::oauth::device_poll),
        )
        .route(
            "/oauth/{provider}/refresh",
            post(routes::oauth::refresh_token),
        )
        // Skills (registry proxy)
        .route("/skills/search", get(routes::skills::search_skills))
        .route("/skills/trending", get(routes::skills::trending_skills))
        .route("/skills/metadata", get(routes::skills::skill_metadata))
        // Workspaces
        .route(
            "/workspaces",
            post(routes::workspaces::create_workspace).get(routes::workspaces::list_workspaces),
        )
        .route(
            "/workspaces/{id}",
            get(routes::workspaces::get_workspace)
                .put(routes::workspaces::update_workspace)
                .delete(routes::workspaces::delete_workspace),
        )
        .route(
            "/workspaces/{id}/launch",
            post(routes::workspaces::launch_workspace),
        )
        .route(
            "/workspaces/{id}/sandboxes",
            get(routes::workspaces::list_workspace_sandboxes),
        )
        .route(
            "/workspaces/{id}/export",
            get(routes::workspaces::export_workspace_toml),
        )
        .route(
            "/workspaces/import",
            post(routes::workspaces::import_workspace_toml),
        )
        // Templates (sources routes first to avoid {id} capture)
        .route(
            "/templates/sources",
            get(routes::templates::list_template_sources)
                .post(routes::templates::add_template_source),
        )
        .route(
            "/templates/sources/{id}",
            axum::routing::delete(routes::templates::delete_template_source),
        )
        .route(
            "/templates/sources/{id}/sync",
            post(routes::templates::sync_template_source),
        )
        .route(
            "/templates",
            get(routes::templates::list_templates).post(routes::templates::create_template),
        )
        .route(
            "/templates/{id}/create",
            post(routes::templates::create_from_template),
        )
        // Agents
        .route("/agents", get(routes::agents::list_providers))
        .route(
            "/agents/{provider}/commands",
            get(routes::agents::get_slash_commands),
        )
        // Gateway
        .route(
            "/gateway/config",
            get(routes::gateway::get_gateway_config).put(routes::gateway::update_gateway_config),
        )
        .route("/gateway/status", get(routes::gateway::gateway_status))
        .route(
            "/gateway/tokens",
            post(routes::gateway::create_token).get(routes::gateway::list_tokens),
        )
        .route(
            "/gateway/tokens/{id}",
            get(routes::gateway::get_token).delete(routes::gateway::revoke_token),
        )
        .route(
            "/gateway/tunnels",
            post(routes::gateway::create_tunnel).get(routes::gateway::list_tunnels),
        )
        .route(
            "/gateway/tunnels/{id}",
            get(routes::gateway::get_tunnel).delete(routes::gateway::delete_tunnel),
        )
        .route(
            "/gateway/tunnels/sandbox/{sandbox_id}",
            post(routes::gateway::create_sandbox_tunnel),
        )
        .route("/gateway/expose", post(routes::gateway::expose))
        .route("/gateway/discover", get(routes::gateway::discover))
        .route(
            "/gateway/providers/prepare",
            post(routes::gateway::prepare_provider),
        )
        // LLM Providers
        .route(
            "/llm-providers",
            post(routes::llm_providers::create_llm_provider)
                .get(routes::llm_providers::list_llm_providers),
        )
        .route(
            "/llm-providers/detect",
            get(routes::llm_providers::detect_providers),
        )
        .route(
            "/llm-providers/compatibility",
            get(routes::llm_providers::compatibility),
        )
        .route(
            "/llm-providers/ollama/pull",
            post(routes::llm_providers::ollama_pull),
        )
        .route(
            "/llm-providers/{id}",
            get(routes::llm_providers::get_llm_provider)
                .put(routes::llm_providers::update_llm_provider)
                .delete(routes::llm_providers::delete_llm_provider),
        )
        .route(
            "/llm-providers/{id}/models",
            get(routes::llm_providers::list_models),
        )
        .route(
            "/llm-providers/{id}/models/refresh",
            post(routes::llm_providers::refresh_models),
        )
        .route(
            "/llm-providers/{id}/test",
            post(routes::llm_providers::test_provider),
        )
        // Channels
        .route(
            "/channels",
            post(routes::channels::create_channel).get(routes::channels::list_channels),
        )
        .route(
            "/channels/{id}",
            get(routes::channels::get_channel)
                .put(routes::channels::update_channel)
                .delete(routes::channels::delete_channel),
        )
        .route(
            "/channels/{id}/start",
            post(routes::channels::start_channel),
        )
        .route("/channels/{id}/stop", post(routes::channels::stop_channel))
        .route(
            "/channels/{id}/restart",
            post(routes::channels::restart_channel),
        )
        .route(
            "/channels/{id}/status",
            get(routes::channels::channel_status),
        )
        .route("/channels/{id}/qr", get(routes::channels::channel_qr))
        .route(
            "/channels/{id}/messages",
            get(routes::channels::channel_messages),
        )
        // Auth middleware on all API routes
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    // Compose: public routes + nested API routes under /api/v1
    let mut app = axum::Router::new()
        .merge(public_routes)
        .nest("/api/v1", api_routes);

    // Serve the built web UI as a SPA when configured.
    // All non-API requests fall through to static files, with index.html as
    // the fallback so client-side routing (e.g. /gateway, /sandboxes) works.
    if let Some(ref web_ui_dir) = state.config.server.web_ui_dir {
        let dir = std::path::PathBuf::from(web_ui_dir);
        if dir.join("index.html").exists() {
            tracing::info!(path = %dir.display(), "serving web UI");
            let serve =
                ServeDir::new(&dir).not_found_service(ServeFile::new(dir.join("index.html")));
            app = app.fallback_service(serve);
        } else {
            tracing::warn!(
                path = %dir.display(),
                "web_ui_dir configured but index.html not found — skipping SPA serving"
            );
        }
    }

    app.layer(timeout)
        .layer(cors)
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .with_state(state)
}

/// Build CORS layer from configuration.
///
/// - If `cors_origins` is empty → permissive (dev mode).
/// - If `cors_origins` contains `"*"` → allow any origin.
/// - Otherwise → restrict to the listed origins.
fn build_cors_layer(state: &AppState) -> CorsLayer {
    let origins = &state.config.server.cors_origins;

    if origins.is_empty() {
        return CorsLayer::permissive();
    }

    let allow_origin = if origins.iter().any(|o| o == "*") {
        AllowOrigin::any()
    } else {
        let parsed: Vec<_> = origins.iter().filter_map(|o| o.parse().ok()).collect();
        AllowOrigin::list(parsed)
    };

    CorsLayer::new()
        .allow_origin(allow_origin)
        .allow_methods(AllowMethods::any())
        .allow_headers(AllowHeaders::any())
        .max_age(Duration::from_secs(86400))
}
