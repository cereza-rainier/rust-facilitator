use axum::{extract::State, Json};
use serde_json::{json, Value};
use crate::config::Config;

/// Health check endpoint
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service is healthy", body = Value,
         example = json!({"status": "ok", "version": "1.0.0"}))
    ),
    tag = "Health"
)]
pub async fn health_check(State(config): State<Config>) -> Json<Value> {
    // Record health check metric
    config.metrics.health_requests.with_label_values::<&str>(&[]).inc();
    
    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

