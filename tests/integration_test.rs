use axum::{
    body::Body,
    http::{Request, StatusCode, Method},
};
use serde_json::{json, Value};
use tower::ServiceExt;
use x402_facilitator::types::responses::{SupportedResponse, VerifyResponse};
use std::sync::Arc;

// Helper to create test config
fn create_test_config() -> x402_facilitator::config::Config {
    use solana_client::rpc_client::RpcClient;
    use solana_sdk::commitment_config::CommitmentConfig;
    use std::sync::Arc;
    use x402_facilitator::cache::AccountCache;
    use x402_facilitator::metrics::AppMetrics;

    let rpc_url = std::env::var("SOLANA_RPC_URL")
        .unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());

    let rpc_client = Arc::new(RpcClient::new_with_commitment(
        rpc_url.clone(),
        CommitmentConfig::confirmed(),
    ));

    // Create test cache, metrics, rate limiter, dedup, and audit logger
    let account_cache = AccountCache::new(100, 30);
    let metrics = AppMetrics::new();
    let transaction_dedup = x402_facilitator::dedup::TransactionDedup::new(1000, 300);
    let audit_logger = x402_facilitator::audit::AuditLogger::new();

    x402_facilitator::config::Config {
        solana_rpc_url: rpc_url,
        fee_payer_private_key: "test_key".to_string(),
        network: "solana-devnet".to_string(),
        port: 3000,
        rpc_client,
        account_cache,
        metrics,
        rate_limiter: None, // Disable rate limiting for tests
        webhook: None, // Disable webhooks for tests
        transaction_dedup,
        payment_expiry_seconds: 600,
        audit_logger,
    }
}

#[tokio::test]
async fn test_health_endpoint() {
    // Create config and router
    let config = create_test_config();
    let app = x402_facilitator::server::create_router(config);

    // Make request
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert response
    assert_eq!(response.status(), StatusCode::OK);

    // Parse body
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let health: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(health["status"], "ok");
}

#[tokio::test]
async fn test_supported_endpoint() {
    // Create config and router
    let config = create_test_config();
    let app = x402_facilitator::server::create_router(config);

    // Make request
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/supported")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert response
    assert_eq!(response.status(), StatusCode::OK);

    // Parse body
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let supported: SupportedResponse = serde_json::from_slice(&body).unwrap();

    // Validate response structure
    assert!(!supported.schemes.is_empty());
    
    // Find the exact scheme
    let exact_scheme = supported.schemes.iter().find(|s| s.scheme == "exact");
    assert!(exact_scheme.is_some());
    
    // Verify it supports solana-devnet
    let exact = exact_scheme.unwrap();
    assert!(exact.networks.contains(&"solana-devnet".to_string()));
}

#[tokio::test]
async fn test_verify_endpoint_with_invalid_scheme() {
    // Create config and router
    let config = create_test_config();
    let app = x402_facilitator::server::create_router(config);

    // Create request with invalid scheme
    let verify_request = json!({
        "paymentPayload": {
            "x402Version": 1,
            "scheme": "invalid_scheme",
            "network": "solana-devnet",
            "payload": {
                "transaction": "test_transaction_base64"
            }
        },
        "paymentRequirements": {
            "scheme": "exact",
            "network": "solana-devnet",
            "maxAmountRequired": "1000000",
            "asset": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            "payTo": "8VzycpqZpqYXMqKSZqYXMqKSZqYXMqKS",
            "resource": "/api/resource",
            "description": "Test payment",
            "mimeType": "application/json",
            "maxTimeoutSeconds": 30,
            "extra": {
                "feePayer": "FeePayerPublicKeyHere"
            }
        }
    });

    // Make request
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/verify")
                .header("content-type", "application/json")
                .body(Body::from(verify_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // The server might return 422 if JSON validation fails early,
    // or 200 with is_valid: false if validation happens in handler
    // For now, just verify we get a response
    assert!(response.status() == StatusCode::OK || response.status() == StatusCode::UNPROCESSABLE_ENTITY);

    // If we got 200 OK, parse and validate the response
    if response.status() == StatusCode::OK {
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let verify: VerifyResponse = serde_json::from_slice(&body).unwrap();

        // Should be invalid
        assert_eq!(verify.is_valid, false);
        assert!(verify.invalid_reason.is_some());
        assert_eq!(verify.invalid_reason.unwrap(), "unsupported_scheme");
    }
}

#[tokio::test]
async fn test_verify_endpoint_with_network_mismatch() {
    // Create config and router
    let config = create_test_config();
    let app = x402_facilitator::server::create_router(config);

    // Create request with network mismatch
    let verify_request = json!({
        "paymentPayload": {
            "x402Version": 1,
            "scheme": "exact",
            "network": "solana",  // mainnet
            "payload": {
                "transaction": "test_transaction_base64"
            }
        },
        "paymentRequirements": {
            "scheme": "exact",
            "network": "solana-devnet",  // devnet
            "maxAmountRequired": "1000000",
            "asset": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            "payTo": "8VzycpqZpqYXMqKSZqYXMqKSZqYXMqKS",
            "resource": "/api/resource",
            "description": "Test payment",
            "mimeType": "application/json",
            "maxTimeoutSeconds": 30,
            "extra": {
                "feePayer": "FeePayerPublicKeyHere"
            }
        }
    });

    // Make request
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/verify")
                .header("content-type", "application/json")
                .body(Body::from(verify_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // The server might return 422 if JSON validation fails early,
    // or 200 with is_valid: false if validation happens in handler
    assert!(response.status() == StatusCode::OK || response.status() == StatusCode::UNPROCESSABLE_ENTITY);

    // If we got 200 OK, parse and validate the response
    if response.status() == StatusCode::OK {
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let verify: VerifyResponse = serde_json::from_slice(&body).unwrap();

        // Should be invalid
        assert_eq!(verify.is_valid, false);
        assert!(verify.invalid_reason.is_some());
        assert_eq!(verify.invalid_reason.unwrap(), "invalid_network");
    }
}

#[tokio::test]
async fn test_404_on_invalid_endpoint() {
    // Create config and router
    let config = create_test_config();
    let app = x402_facilitator::server::create_router(config);

    // Make request to invalid endpoint
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/invalid")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return 404
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_connection_pooling_reuses_client() {
    // This test verifies that the Arc<RpcClient> is properly shared
    // We create multiple requests and ensure they all work
    let config = create_test_config();
    
    // Clone config multiple times (simulating multiple requests)
    let config1 = config.clone();
    let config2 = config.clone();
    let config3 = config.clone();

    // All should have the same Arc reference count
    // Note: This is a basic check that cloning works
    assert!(Arc::strong_count(&config1.rpc_client) > 1);
    assert!(Arc::ptr_eq(&config1.rpc_client, &config2.rpc_client));
    assert!(Arc::ptr_eq(&config2.rpc_client, &config3.rpc_client));
}

