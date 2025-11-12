use axum::{
    extract::Request,
    http::HeaderValue,
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

/// Request ID extension that can be extracted in handlers
#[derive(Clone, Debug)]
pub struct RequestId(pub String);

/// Middleware that adds a unique request ID to each request
/// The request ID is:
/// 1. Extracted from X-Request-ID header if present
/// 2. Generated as a new UUID if not present
/// 3. Added to response headers
/// 4. Available in request extensions for logging
pub async fn request_id_middleware(
    mut req: Request,
    next: Next,
) -> Response {
    // Try to get request ID from header, or generate new one
    let request_id = req
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    // Store in request extensions for handler access
    req.extensions_mut().insert(RequestId(request_id.clone()));

    // Create span for structured logging
    let span = tracing::info_span!(
        "request",
        request_id = %request_id,
        method = %req.method(),
        uri = %req.uri().path(),
    );
    let _enter = span.enter();

    tracing::info!("Request started");

    // Process request
    let mut response = next.run(req).await;

    // Add request ID to response headers
    if let Ok(header_value) = HeaderValue::from_str(&request_id) {
        response.headers_mut().insert("x-request-id", header_value);
    }

    tracing::info!("Request completed");

    response
}

