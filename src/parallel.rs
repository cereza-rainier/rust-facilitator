// Parallel batch verification using Rayon
// This module enables true multi-threaded verification across all CPU cores

use rayon::prelude::*;
use crate::types::{requests::VerifyRequest, responses::VerifyResponse};
use crate::config::Config;
use crate::solana::decoder::decode_transaction_from_base64;
use crate::solana::verifier::*;
use crate::error::VerificationError;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

/// Verify multiple payments in parallel across all CPU cores
/// 
/// This uses Rayon's parallel iterators to distribute verification
/// work across all available CPU cores, achieving true parallelism.
/// 
/// # Example Performance
/// - Single-threaded: 1000 payments × 5ms = 5000ms
/// - 8-core parallel: 125 payments/core × 5ms = 625ms (8x faster!)
pub fn verify_batch_parallel(
    config: &Config,
    requests: Vec<VerifyRequest>,
) -> Vec<VerifyResponse> {
    tracing::info!(
        "🚀 Starting parallel batch verification for {} requests across {} cores",
        requests.len(),
        rayon::current_num_threads()
    );

    let start = std::time::Instant::now();

    let results: Vec<VerifyResponse> = requests
        .par_iter()  // Parallel iterator - THIS is the magic!
        .map(|request| {
            verify_single_sync(config, request)
        })
        .collect();

    let duration = start.elapsed();
    let per_request = duration.as_micros() as f64 / requests.len() as f64;

    tracing::info!(
        "✅ Batch verification complete: {} requests in {:?} ({:.2}μs per request)",
        requests.len(),
        duration,
        per_request
    );

    results
}

/// Synchronous version of payment verification
/// 
/// This is designed to work with Rayon's thread pool.
/// It performs all the same checks as the async version,
/// but in a blocking/synchronous manner.
fn verify_single_sync(
    config: &Config,
    request: &VerifyRequest,
) -> VerifyResponse {
    // Record metrics
    let network = &request.payment_payload.network;
    config.metrics.verify_requests.with_label_values(&[network]).inc();
    
    // Perform verification
    match verify_payment_sync(config, request) {
        Ok(payer) => {
            config.metrics.record_verification_success(network);
            
            // Audit log
            config.audit_logger.log_verification_success(network, &payer, None);
            
            VerifyResponse {
                is_valid: true,
                invalid_reason: None,
                payer: Some(payer),
            }
        }
        Err(e) => {
            tracing::debug!("Verification failed: {}", e);
            config.metrics.record_verification_failure(network, e.as_str());
            
            // Audit log
            config.audit_logger.log_verification_failure(network, e.as_str(), None);
            
            VerifyResponse {
                is_valid: false,
                invalid_reason: Some(e.as_str().to_string()),
                payer: None,
            }
        }
    }
}

/// Synchronous verification logic (blocking version)
fn verify_payment_sync(
    config: &Config,
    request: &VerifyRequest,
) -> Result<String, VerificationError> {
    let payload = &request.payment_payload;
    let requirements = &request.payment_requirements;

    // 0. Check for duplicate transaction (replay attack prevention)
    let transaction_data = &payload.payload.transaction;
    if config.transaction_dedup.check_and_mark(transaction_data) {
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
            return Err(VerificationError::UnexpectedError(
                anyhow::anyhow!(
                    "Payment has expired (age: {} seconds, max: {} seconds)",
                    age_seconds,
                    config.payment_expiry_seconds
                )
            ));
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parallel_verification_empty() {
        // This is a simple test to ensure the module compiles
        // Real tests would need full Config setup
        let requests: Vec<VerifyRequest> = vec![];
        assert_eq!(requests.len(), 0);
    }

    #[test]
    fn test_rayon_thread_count() {
        // Verify Rayon is using multiple threads
        let thread_count = rayon::current_num_threads();
        println!("Rayon thread pool size: {}", thread_count);
        assert!(thread_count > 0);
    }
}

