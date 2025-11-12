use axum::{extract::State, Json};
use solana_sdk::signer::Signer;

use crate::{
    config::Config,
    handlers::verify::verify,
    solana::{
        decoder::decode_transaction_from_base64,
        signer::{load_keypair_from_base58, sign_transaction_as_fee_payer},
        submitter::{submit_transaction_with_retries, signature_to_string},
    },
    types::{
        requests::SettleRequest,
        responses::SettleResponse,
    },
};

/// POST /settle - Verify and settle a payment transaction
#[utoipa::path(
    post,
    path = "/settle",
    request_body = SettleRequest,
    responses(
        (status = 200, description = "Settlement result", body = SettleResponse)
    ),
    tag = "Payment"
)]
pub async fn settle(
    State(config): State<Config>,
    Json(request): Json<SettleRequest>,
) -> Json<SettleResponse> {
    let network = request.payment_requirements.network.clone();
    
    // Record settle request metric
    config.metrics.settle_requests.with_label_values(&[&network, &"attempt".to_string()]).inc();
    
    // First, verify the transaction
    let verify_request = crate::types::requests::VerifyRequest {
        payment_payload: request.payment_payload.clone(),
        payment_requirements: request.payment_requirements.clone(),
    };
    
    let verify_response = verify(State(config.clone()), Json(verify_request)).await.0;
    
    if !verify_response.is_valid {
        return Json(SettleResponse {
            success: false,
            network,
            transaction: String::new(),
            payer: verify_response.payer,
            error_reason: verify_response.invalid_reason,
        });
    }
    
    let payer = verify_response.payer;
    
    // Settle the transaction
    match settle_transaction(&config, &request).await {
        Ok(signature) => {
            tracing::info!("Transaction settled successfully: {}", signature);
            config.metrics.settle_requests.with_label_values(&[&network, &"success".to_string()]).inc();
            
            // Send webhook notification (async, non-blocking)
            if let Some(webhook_config) = &config.webhook {
                let webhook_config = webhook_config.clone();
                let sig_clone = signature.clone();
                let payer_clone = payer.clone();
                let network_clone = network.clone();
                tokio::spawn(async move {
                    let payload = crate::webhooks::WebhookPayload::new(
                        crate::webhooks::WebhookEvent::SettlementSuccess,
                        serde_json::json!({
                            "signature": sig_clone,
                            "payer": payer_clone,
                            "network": network_clone,
                        }),
                    );
                    let _ = crate::webhooks::send_webhook(&webhook_config, &payload).await;
                });
            }
            
            Json(SettleResponse {
                success: true,
                network,
                transaction: signature,
                payer,
                error_reason: None,
            })
        }
        Err(e) => {
            tracing::error!("Settlement failed: {}", e);
            config.metrics.settle_requests.with_label_values(&[&network, &"failure".to_string()]).inc();
            
            // Send webhook notification (async, non-blocking)
            if let Some(webhook_config) = &config.webhook {
                let webhook_config = webhook_config.clone();
                let error_msg = format!("{}", e);
                let payer_clone = payer.clone();
                let network_clone = network.clone();
                tokio::spawn(async move {
                    let payload = crate::webhooks::WebhookPayload::new(
                        crate::webhooks::WebhookEvent::SettlementFailure,
                        serde_json::json!({
                            "error": error_msg,
                            "payer": payer_clone,
                            "network": network_clone,
                        }),
                    );
                    let _ = crate::webhooks::send_webhook(&webhook_config, &payload).await;
                });
            }
            
            Json(SettleResponse {
                success: false,
                network,
                transaction: String::new(),
                payer,
                error_reason: Some(format!("settle_error: {}", e)),
            })
        }
    }
}

/// Internal settlement logic
async fn settle_transaction(
    config: &Config,
    request: &SettleRequest,
) -> Result<String, anyhow::Error> {
    // 1. Decode the transaction
    let mut transaction = decode_transaction_from_base64(
        &request.payment_payload.payload.transaction
    )?;
    
    tracing::info!("Decoded transaction for settlement");
    
    // 2. Load fee payer keypair
    let fee_payer = load_keypair_from_base58(&config.fee_payer_private_key)?;
    
    tracing::info!("Loaded fee payer keypair: {}", fee_payer.pubkey());
    
    // 3. Sign the transaction as fee payer
    sign_transaction_as_fee_payer(&mut transaction, &fee_payer)?;
    
    tracing::info!("Transaction signed by fee payer");
    
    // 4. Use shared RPC client (connection pooling)
    let rpc_client = &config.rpc_client;
    
    // 5. Submit transaction with retries (3 attempts, 30 second timeout each)
    let signature = submit_transaction_with_retries(
        rpc_client.as_ref(),
        &transaction,
        3,  // max retries
        30, // timeout seconds
    ).await?;
    
    Ok(signature_to_string(&signature))
}
