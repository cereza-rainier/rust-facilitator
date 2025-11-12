use anyhow::{anyhow, Result};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    signature::Signature,
    transaction::Transaction,
};
use std::str::FromStr;
use std::time::{Duration, Instant};

/// Submit a signed transaction and wait for confirmation
pub async fn submit_and_confirm_transaction(
    rpc_client: &RpcClient,
    transaction: &Transaction,
    timeout_seconds: u64,
) -> Result<Signature> {
    // Send the transaction
    let signature = rpc_client
        .send_transaction(transaction)
        .map_err(|e| anyhow!("Failed to send transaction: {}", e))?;

    tracing::info!("Transaction sent: {}", signature);

    // Wait for confirmation with timeout
    let start = Instant::now();
    let timeout = Duration::from_secs(timeout_seconds);

    loop {
        if start.elapsed() > timeout {
            return Err(anyhow!("Transaction confirmation timed out after {} seconds", timeout_seconds));
        }

        // Check transaction status
        match rpc_client.get_signature_status(&signature) {
            Ok(Some(status)) => {
                if let Err(e) = status {
                    return Err(anyhow!("Transaction failed: {:?}", e));
                }
                // Transaction confirmed!
                tracing::info!("Transaction confirmed: {}", signature);
                return Ok(signature);
            }
            Ok(None) => {
                // Transaction not yet processed, wait and retry
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
            Err(e) => {
                tracing::warn!("Error checking transaction status: {}", e);
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }
    }
}

/// Submit transaction with retries
pub async fn submit_transaction_with_retries(
    rpc_client: &RpcClient,
    transaction: &Transaction,
    max_retries: u32,
    timeout_seconds: u64,
) -> Result<Signature> {
    let mut last_error = None;

    for attempt in 1..=max_retries {
        tracing::info!("Submission attempt {}/{}", attempt, max_retries);

        match submit_and_confirm_transaction(rpc_client, transaction, timeout_seconds).await {
            Ok(signature) => return Ok(signature),
            Err(e) => {
                tracing::warn!("Attempt {} failed: {}", attempt, e);
                last_error = Some(e);
                
                if attempt < max_retries {
                    // Wait before retry (exponential backoff)
                    let backoff = Duration::from_secs(2u64.pow(attempt - 1));
                    tokio::time::sleep(backoff).await;
                }
            }
        }
    }

    Err(last_error.unwrap_or_else(|| anyhow!("All retry attempts failed")))
}

/// Get transaction signature as string
pub fn signature_to_string(signature: &Signature) -> String {
    signature.to_string()
}

/// Parse signature from string
pub fn string_to_signature(s: &str) -> Result<Signature> {
    Signature::from_str(s).map_err(|e| anyhow!("Invalid signature: {}", e))
}

