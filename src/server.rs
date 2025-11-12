use axum::{routing::{get, post}, Router, middleware, response::IntoResponse, Json};
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use crate::{config::Config, handlers, middleware::request_id::request_id_middleware, ApiDoc};

pub fn create_router(config: Config) -> Router {
    Router::new()
        // Core endpoints
        .route("/health", get(handlers::health::health_check))
        .route("/supported", get(handlers::supported::supported))
        .route("/verify", post(handlers::verify::verify))
        .route("/verify/batch", post(handlers::batch::verify_batch))
        .route("/settle", post(handlers::settle::settle))
        
        // Observability endpoints
        .route("/metrics", get(metrics_handler))
        
        // API Documentation
        .route("/api-docs/openapi.json", get(openapi_json))
        
        // Admin endpoints
        .route("/admin/health", get(handlers::admin::detailed_health))
        .route("/admin/stats", get(handlers::admin::get_stats))
        .route("/admin/config", get(handlers::admin::get_config))
        
        .layer(middleware::from_fn(request_id_middleware))
        .layer(TraceLayer::new_for_http())
        .with_state(config)
}

/// GET /api-docs/openapi.json - OpenAPI specification
async fn openapi_json() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

/// GET /metrics - Prometheus metrics endpoint
async fn metrics_handler() -> impl IntoResponse {
    match crate::metrics::create_prometheus_handle() {
        Ok(metrics) => metrics,
        Err(e) => format!("Error gathering metrics: {}", e),
    }
}

