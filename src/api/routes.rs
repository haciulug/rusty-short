use axum::{
    routing::{delete, get, post},
    Router,
};

use super::handlers::{
    create_short_link, delete_link, generate_qr_code, get_link_stats, health_check, list_links,
    redirect_to_original, get_analytics_summary, get_detailed_analytics, AppState,
};

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/links", post(create_short_link))
        .route("/api/v1/links", get(list_links))
        .route("/api/v1/links/{key}/stats", get(get_link_stats))
        .route("/api/v1/links/{key}/analytics", get(get_analytics_summary))
        .route("/api/v1/links/{key}/analytics/detailed", get(get_detailed_analytics))
        .route("/api/v1/links/{key}", delete(delete_link))
        .route("/qr/{key}", get(generate_qr_code))
        .route("/{key}", get(redirect_to_original))
        .with_state(state)
}

