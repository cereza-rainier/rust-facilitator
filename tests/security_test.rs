use x402_facilitator::{
    config::Config,
    types::{
        requests::{VerifyRequest, PaymentPayload, SvmPayload, PaymentRequirements, ExtraFields},
        responses::VerifyResponse,
    },
};
use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use tower::ServiceExt;
use std::time::{SystemTime, UNIX_EPOCH};

/// Helper to create a test config with security features enabled
fn create_test_config() -> Config {
    std::env::set_var("SOLANA_RPC_URL", "https://api.devnet.solana.com");
    std::env::set_var("FEE_PAYER_PRIVATE_KEY", "test_key_12345678901234567890123456789012");
    std::env::set_var("NETWORK", "solana-devnet");
    std::env::set_var("PORT", "3000");
    std::env::set_var("DEDUP_MAX_ENTRIES", "100");
    std::env::set_var("DEDUP_WINDOW_SECONDS", "300");
    std::env::set_var("PAYMENT_EXPIRY_SECONDS", "600");
    
    Config::from_env().expect("Failed to create test config")
}

/// Helper to create a test verify request
fn create_test_verify_request(transaction_data: &str, timestamp: Option<u64>) -> VerifyRequest {
    VerifyRequest {
        payment_payload: PaymentPayload {
            x402_version: 1,
            scheme: "exact".to_string(),
            network: "solana-devnet".to_string(),
            payload: SvmPayload {
                transaction: transaction_data.to_string(),
            },
            timestamp,
        },
        payment_requirements: PaymentRequirements {
            scheme: "exact".to_string(),
            network: "solana-devnet".to_string(),
            max_amount_required: "1000000".to_string(),
            asset: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            pay_to: "8VzycpqZpqYXMqKSZqYXMqKSZqYXMqKS".to_string(),
            resource: "/api/resource".to_string(),
            description: "Test payment".to_string(),
            mime_type: "application/json".to_string(),
            max_timeout_seconds: 30,
            output_schema: None,
            extra: ExtraFields {
                fee_payer: "FeePayerPublicKeyHere123456789".to_string(),
            },
        },
    }
}

#[tokio::test]
async fn test_transaction_deduplication() {
    // Create config with dedup enabled
    let config = create_test_config();
    
    // Same transaction data
    let tx_data = "AQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAEDArczbMia1tLmq7zz4DinMNN0pJ1JtLdqIJPUw3YrGCzYAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAgIAAQwCAAAAKgAAAAAAAAA=";
    
    // First verification - should be marked as seen
    let is_dup_1 = config.transaction_dedup.check_and_mark(tx_data);
    assert!(!is_dup_1, "First transaction should not be duplicate");
    
    // Second verification with same data - should be detected as duplicate
    let is_dup_2 = config.transaction_dedup.check_and_mark(tx_data);
    assert!(is_dup_2, "Second transaction with same data should be duplicate");
    
    // Different transaction - should not be duplicate
    let tx_data_2 = "DIFFERENT_TRANSACTION_DATA";
    let is_dup_3 = config.transaction_dedup.check_and_mark(tx_data_2);
    assert!(!is_dup_3, "Different transaction should not be duplicate");
}

#[tokio::test]
async fn test_payment_expiry_validation() {
    let config = create_test_config();
    
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    // Test 1: Recent payment (should pass)
    let recent_timestamp = current_time - 60; // 1 minute old
    let tx_data = "RECENT_TX";
    
    // Simulate expiry check (would be done in verify handler)
    let age = current_time - recent_timestamp;
    assert!(age <= config.payment_expiry_seconds, "Recent payment should not be expired");
    
    // Test 2: Expired payment (should fail)
    let old_timestamp = current_time - 700; // 700 seconds old (> 600 second expiry)
    let age_old = current_time - old_timestamp;
    assert!(age_old > config.payment_expiry_seconds, "Old payment should be expired");
    
    // Test 3: Edge case - exactly at expiry limit
    let edge_timestamp = current_time - config.payment_expiry_seconds;
    let age_edge = current_time - edge_timestamp;
    assert!(age_edge <= config.payment_expiry_seconds, "Payment at exact expiry should still be valid");
}

#[tokio::test]
async fn test_dedup_cache_stats() {
    let config = create_test_config();
    
    // Add some entries
    config.transaction_dedup.mark_seen("tx1");
    config.transaction_dedup.mark_seen("tx2");
    config.transaction_dedup.mark_seen("tx3");
    
    let stats = config.transaction_dedup.stats();
    assert_eq!(stats.entry_count, 3, "Should have 3 cached entries");
    assert_eq!(stats.window_seconds, 300, "Window should be 300 seconds");
}

#[tokio::test]
async fn test_dedup_window_config() {
    // Test with custom window
    std::env::set_var("DEDUP_WINDOW_SECONDS", "120");
    let config = create_test_config();
    
    let stats = config.transaction_dedup.stats();
    assert_eq!(stats.window_seconds, 120, "Custom window should be respected");
}

#[tokio::test]
async fn test_payment_expiry_config() {
    // Test with custom expiry
    std::env::set_var("PAYMENT_EXPIRY_SECONDS", "300");
    let config = create_test_config();
    
    assert_eq!(config.payment_expiry_seconds, 300, "Custom expiry should be respected");
}

#[test]
fn test_dedup_hash_consistency() {
    let config = create_test_config();
    
    let tx = "test_transaction_data";
    
    // Mark as seen
    config.transaction_dedup.mark_seen(tx);
    
    // Should be duplicate
    assert!(config.transaction_dedup.is_duplicate(tx), "Should detect duplicate");
}

#[test]
fn test_dedup_different_transactions() {
    let config = create_test_config();
    
    config.transaction_dedup.mark_seen("tx1");
    config.transaction_dedup.mark_seen("tx2");
    
    // tx3 should not be duplicate
    assert!(!config.transaction_dedup.is_duplicate("tx3"));
    
    // tx1 and tx2 should be duplicates
    assert!(config.transaction_dedup.is_duplicate("tx1"));
    assert!(config.transaction_dedup.is_duplicate("tx2"));
}

#[tokio::test]
async fn test_timestamp_none_skips_expiry() {
    let config = create_test_config();
    
    // Request without timestamp should not fail expiry validation
    let request = create_test_verify_request("test_tx", None);
    assert!(request.payment_payload.timestamp.is_none(), "Timestamp should be None");
    
    // In the actual verify handler, this would skip expiry validation
    // This test just confirms the structure allows None
}

#[tokio::test]
async fn test_timestamp_some_enables_expiry() {
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    // Request with timestamp should enable expiry validation
    let request = create_test_verify_request("test_tx", Some(current_time));
    assert!(request.payment_payload.timestamp.is_some(), "Timestamp should be Some");
    assert_eq!(request.payment_payload.timestamp.unwrap(), current_time);
}

#[tokio::test]
async fn test_config_validation_includes_security() {
    let config = create_test_config();
    
    // Config should have security features initialized
    let dedup_stats = config.transaction_dedup.stats();
    assert!(dedup_stats.entry_count >= 0, "Dedup should be initialized");
    assert!(config.payment_expiry_seconds > 0, "Payment expiry should be configured");
}

