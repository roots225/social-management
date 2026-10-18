pub mod dashboard;
pub mod publications;
pub mod scraped_items;
pub mod social_accounts;
pub mod sources;

use crate::AppState;
use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde_json::json;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

async fn health_check() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({
            "status": "healthy",
            "service": "social-media-manager-backend",
            "timestamp": chrono::Utc::now()
        })),
    )
}

pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let api_v1 = Router::new()
        // Health
        .route("/health", get(health_check))
        // Sources
        .route("/sources", get(sources::list_sources).post(sources::create_source))
        .route(
            "/sources/:id",
            put(sources::update_source).delete(sources::delete_source),
        )
        .route("/sources/:id/scrape-now", post(sources::scrape_source_now))
        // Scraped items (Curation)
        .route("/scraped-items", get(scraped_items::list_scraped_items))
        .route(
            "/scraped-items/:id/status",
            put(scraped_items::update_item_status),
        )
        .route(
            "/scraped-items/:id/queue",
            post(scraped_items::queue_scraped_item),
        )
        // Publications (Queue)
        .route(
            "/publications",
            get(publications::list_publications),
        )
        .route(
            "/publications/:id",
            put(publications::update_publication).delete(publications::delete_publication),
        )
        .route("/publications/:id/retry", post(publications::retry_publication))
        .route(
            "/publications/:id/publish-now",
            post(publications::publish_now),
        )
        // Social Accounts
        .route(
            "/social-accounts",
            get(social_accounts::list_social_accounts).post(social_accounts::create_social_account),
        )
        .route(
            "/social-accounts/:id",
            delete(social_accounts::delete_social_account),
        )
        .route(
            "/social-accounts/:id/verify",
            post(social_accounts::verify_social_account),
        )
        // Dashboard Stats
        .route("/dashboard/stats", get(dashboard::get_dashboard_stats));

    Router::new()
        .nest("/api/v1", api_v1)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
