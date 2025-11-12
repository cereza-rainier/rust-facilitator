use axum::Json;
use crate::types::responses::{SchemeSupport, SupportedResponse};

/// GET /supported - Returns supported payment schemes and networks
#[utoipa::path(
    get,
    path = "/supported",
    responses(
        (status = 200, description = "List of supported schemes", body = SupportedResponse)
    ),
    tag = "Information"
)]
pub async fn supported() -> Json<SupportedResponse> {
    Json(SupportedResponse {
        schemes: vec![SchemeSupport {
            scheme: "exact".to_string(),
            networks: vec![
                "solana-devnet".to_string(),
                "solana".to_string(),
            ],
        }],
    })
}
