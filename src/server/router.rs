use axum::{
    routing::{get, patch, post},
    Router,
};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

use super::handlers::{self, AppState};
use super::static_files::serve_static;

/// Create the main router with all routes
pub fn create_router(state: Arc<AppState>) -> Router {
    // API routes
    let api_routes = Router::new()
        // Date/Archive routes
        .route("/dates", get(handlers::list_dates))
        .route("/dates/:date", get(handlers::get_daily_summary))
        .route("/dates/:date/digest", post(handlers::trigger_digest))
        .route("/dates/:date/insights", get(handlers::get_date_insights))
        .route("/dates/:date/sessions", get(handlers::list_sessions))
        .route("/dates/:date/sessions/:name", get(handlers::get_session))
        .route(
            "/dates/:date/sessions/:name/conversation",
            get(handlers::get_session_conversation),
        )
        // Job routes
        .route("/jobs", get(handlers::list_jobs))
        .route("/jobs/:id", get(handlers::get_job))
        .route("/jobs/:id/log", get(handlers::get_job_log))
        .route("/jobs/:id/kill", post(handlers::kill_job))
        // Config routes
        .route("/config", get(handlers::get_config))
        .route("/config", patch(handlers::update_config))
        .route(
            "/config/templates/defaults",
            get(handlers::get_default_templates),
        )
        // Health check
        .route("/health", get(handlers::health_check))
        // Insights routes
        .route("/insights", get(handlers::get_insights));

    // CORS layer for development
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Combine routes
    Router::new()
        .nest("/api", api_routes)
        .fallback_service(serve_static())
        .layer(cors)
        .with_state(state)
}
