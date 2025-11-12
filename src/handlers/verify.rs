use axum::{extract::State, Json};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

use crate::{
    config::Config,
    error::VerificationError,
    solana::{
        decoder::decode_transaction_from_base64,
        verifier::*,
    },
    types::{
        requests::VerifyRequest,
        responses::VerifyResponse,
    },
};

/// POST /verify - Verify a payment transaction
#[utoipa::path(
    post,
    path = "/verify",
    request_body = VerifyRequest,
    responses(
        (status = 200, description = "Verification result", body = VerifyResponse)
    ),
    tag = "Payment"
)]
pub async fn verify(
    State(config): State<Config>,
    Json(request): Json<VerifyRequest>,
) -> Json<VerifyResponse> {
    // Record metrics
    let network = &request.payment_payload.network;
    config.metrics.verify_requests.with_label_values(&[network]).inc();
    
    // Update cache size metric
    let stats = config.account_cache.stats();
    config.metrics.update_cache_size(stats.entry_count);
    tracing::debug!("Cache stats: {} entries", stats.entry_count);
    
    // Log verification request
    config.audit_logger.log_verification_request(network, None);
    
    // Perform verification
    match verify_payment(&config, &request).await {
        Ok(payer) => {
            config.metrics.record_verification_success(network);
            
            // Audit log success
            config.audit_logger.log_verification_success(network, &payer, None);
            
            // Send webhook notification (async, non-blocking)
            if let Some(webhook_config) = &config.webhook {
                let webhook_config = webhook_config.clone();
                let payer_clone = payer.clone();
                let network_clone = network.clone();
                tokio::spawn(async move {
                    let payload = crate::webhooks::WebhookPayload::new(
                        crate::webhooks::WebhookEvent::VerificationSuccess,
                        serde_json::json!({
                            "payer": payer_clone,
                            "network": network_clone,
                        }),
                    );
                    let _ = crate::webhooks::send_webhook(&webhook_config, &payload).await;
                });
            }
            
            Json(VerifyResponse {
                is_valid: true,
                invalid_reason: None,
                payer: Some(payer),
            })
        }
        Err(e) => {
            tracing::warn!("Verification failed: {}", e);
            config.metrics.record_verification_failure(network, e.as_str());
            
            // Audit log failure
            config.audit_logger.log_verification_failure(network, e.as_str(), None);
            
            // Send webhook notification (async, non-blocking)
            if let Some(webhook_config) = &config.webhook {
                let webhook_config = webhook_config.clone();
                let reason = e.as_str().to_string();
                let network_clone = network.clone();
                tokio::spawn(async move {
                    let payload = crate::webhooks::WebhookPayload::new(
                        crate::webhooks::WebhookEvent::VerificationFailure,
                        serde_json::json!({
                            "reason": reason,
                            "network": network_clone,
                        }),
                    );
                    let _ = crate::webhooks::send_webhook(&webhook_config, &payload).await;
                });
            }
            
            Json(VerifyResponse {
                is_valid: false,
                invalid_reason: Some(e.as_str().to_string()),
                payer: None,
            })
        }
    }
}

/// Internal verification logic
async fn verify_payment(
    config: &Config,
    request: &VerifyRequest,
) -> Result<String, VerificationError> {
    let payload = &request.payment_payload;
    let requirements = &request.payment_requirements;

    // 0. Check for duplicate transaction (replay attack prevention)
    let transaction_data = &payload.payload.transaction;
    if config.transaction_dedup.check_and_mark(transaction_data) {
        tracing::warn!("🚨 Duplicate transaction detected - rejecting");
        return Err(VerificationError::UnexpectedError(
            anyhow::anyhow!("Transaction has already been processed (replay attack prevented)")
        ));
    }

    // 0.5. Validate payment expiry (if timestamp is provided)
    if let Some(timestamp) = payload.timestamp {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| VerificationError::UnexpectedError(anyhow::anyhow!("System time error: {}", e)))?
            .as_secs();
        
        let age_seconds = current_time.saturating_sub(timestamp);
        
        if age_seconds > config.payment_expiry_seconds {
            tracing::warn!(
                "⏰ Payment expired: age={} seconds, max={} seconds",
                age_seconds,
                config.payment_expiry_seconds
            );
            return Err(VerificationError::UnexpectedError(
                anyhow::anyhow!(
                    "Payment has expired (age: {} seconds, max: {} seconds)",
                    age_seconds,
                    config.payment_expiry_seconds
                )
            ));
        }
        
        tracing::debug!("✅ Payment age validation passed: {} seconds old", age_seconds);
    } else {
        tracing::debug!("⚠️  No timestamp in payload, skipping expiry validation");
    }

    // 1. Verify scheme and network match
    if payload.scheme != requirements.scheme || payload.scheme != "exact" {
        return Err(VerificationError::UnsupportedScheme);
    }

    if payload.network != requirements.network {
        return Err(VerificationError::InvalidNetwork);
    }

    // Verify network is supported
    if requirements.network != "solana" && requirements.network != "solana-devnet" {
        return Err(VerificationError::InvalidNetwork);
    }

    // 2. Decode transaction
    let transaction = decode_transaction_from_base64(&payload.payload.transaction)
        .map_err(|_| VerificationError::UnexpectedError(
            anyhow::anyhow!("Failed to decode transaction")
        ))?;

    // Get fee payer from requirements
    let fee_payer = Pubkey::from_str(&requirements.extra.fee_payer)
        .map_err(|_| VerificationError::UnexpectedError(
            anyhow::anyhow!("Invalid fee payer pubkey")
        ))?;

    // Get payer (client) for response
    let payer = if let Some(first_key) = transaction.message.account_keys.get(1) {
        first_key.to_string()
    } else {
        "unknown".to_string()
    };

    // 3. Verify instruction count (3 or 4)
    let has_create_ata = verify_instruction_count(&transaction)?;

    // 4. Verify compute budget instructions
    verify_compute_limit_instruction(
        &transaction.message.instructions[0],
        &transaction.message,
    )?;

    verify_compute_price_instruction(
        &transaction.message.instructions[1],
        &transaction.message,
    )?;

    // 5. Verify fee payer safety (not in any instruction accounts)
    verify_fee_payer_safety(&transaction, &fee_payer)?;

    // 6. Use shared RPC client (connection pooling)
    let rpc_client = &config.rpc_client;

    // 7. Verify CreateATA instruction (if present)
    if has_create_ata {
        verify_create_ata_instruction(
            &transaction.message.instructions[2],
            &transaction.message,
            requirements,
        )?;
    }

    // 8. Verify transfer instruction (last instruction)
    let transfer_idx = if has_create_ata { 3 } else { 2 };
    verify_transfer_instruction(
        &transaction.message.instructions[transfer_idx],
        &transaction.message,
        requirements,
        &fee_payer,
        has_create_ata,
        rpc_client.as_ref(),
    )?;

    Ok(payer)
}
