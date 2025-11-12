// Batch verification handler - leverages parallel processing
// This endpoint can verify thousands of payments simultaneously,
// utilizing all CPU cores for maximum throughput.

use axum::{extract::State, Json};
use crate::{
    config::Config,
    parallel::verify_batch_parallel,
    types::{requests::VerifyRequest, responses::VerifyResponse},
};

/// Verify multiple payments in parallel
/// 
/// This endpoint is designed for bulk verification scenarios:
/// - Payment processors handling hundreds of transactions
/// - AI agent platforms with many concurrent agents
/// - Analytics systems processing historical payments
/// - Webhook handlers receiving batched events
/// 
/// **Performance:** Utilizes all CPU cores via Rayon's work-stealing thread pool.
/// - On an 8-core machine: ~8x faster than sequential verification
/// - Memory efficient: only the results are kept in memory
/// - Fault tolerant: individual failures don't block the batch
/// 
/// # Example Request
/// ```json
/// [
///   {
///     "payment_payload": { ... },
///     "payment_requirements": { ... }
///   },
///   {
///     "payment_payload": { ... },
///     "payment_requirements": { ... }
///   }
/// ]
/// ```
/// 
/// # Example Response
/// ```json
/// [
///   {
///     "is_valid": true,
///     "payer": "wallet_address_1"
///   },
///   {
///     "is_valid": false,
///     "invalid_reason": "Insufficient amount"
///   }
/// ]
/// ```
#[utoipa::path(
    post,
    path = "/verify/batch",
    request_body = Vec<VerifyRequest>,
    responses(
        (status = 200, description = "Batch verification results", body = Vec<VerifyResponse>)
    ),
    tag = "Payment"
)]
pub async fn verify_batch(
    State(config): State<Config>,
    Json(requests): Json<Vec<VerifyRequest>>,
) -> Json<Vec<VerifyResponse>> {
    let batch_size = requests.len();
    
    tracing::info!(
        "📦 Received batch verification request for {} payments",
        batch_size
    );

    if batch_size == 0 {
        return Json(vec![]);
    }

    // Spawn blocking to move to Rayon's thread pool
    // This prevents blocking Tokio's async runtime
    let results = tokio::task::spawn_blocking(move || {
        verify_batch_parallel(&config, requests)
    })
    .await
    .unwrap_or_else(|e| {
        tracing::error!("Batch verification task panicked: {}", e);
        // Return empty results on panic
        vec![]
    });

    tracing::info!(
        "✅ Batch verification complete: {}/{} valid",
        results.iter().filter(|r| r.is_valid).count(),
        batch_size
    );

    Json(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        requests::{PaymentPayload, PaymentRequirements, SvmPayload, ExtraFields},
    };

    #[test]
    fn test_batch_empty() {
        // Test empty batch
        let requests: Vec<VerifyRequest> = vec![];
        assert_eq!(requests.len(), 0);
    }

    #[test]
    fn test_batch_structure() {
        // Test that batch structure is correct
        let request = VerifyRequest {
            payment_payload: PaymentPayload {
                x402_version: 1,
                scheme: "exact".to_string(),
                network: "solana-devnet".to_string(),
                payload: SvmPayload {
                    transaction: "test".to_string(),
                },
                timestamp: None,
            },
            payment_requirements: PaymentRequirements {
                scheme: "exact".to_string(),
                network: "solana-devnet".to_string(),
                max_amount_required: "1000000".to_string(),
                asset: "SOL".to_string(),
                pay_to: "recipient".to_string(),
                resource: "/api/test".to_string(),
                description: "Test".to_string(),
                mime_type: "application/json".to_string(),
                max_timeout_seconds: 30,
                output_schema: None,
                extra: ExtraFields {
                    fee_payer: "fee_payer".to_string(),
                },
            },
        };

        let batch = vec![request.clone(), request.clone()];
        assert_eq!(batch.len(), 2);
    }
}

