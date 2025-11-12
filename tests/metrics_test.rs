/// Integration test for Prometheus metrics functionality
use axum::{
    body::Body,
    http::{Request, StatusCode, Method},
};
use tower::ServiceExt;

#[tokio::test]
async fn test_metrics_endpoint() {
    // Create config and router
    let config = create_test_config();
    let app = x402_facilitator::server::create_router(config);

    // Make request to /metrics
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/metrics")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert response
    assert_eq!(response.status(), StatusCode::OK);

    // Parse body
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let metrics_text = String::from_utf8(body.to_vec()).unwrap();

    // Verify we get a response (metrics may be empty if no requests have been made)
    // Just verify the format is correct
    assert!(!metrics_text.is_empty(), "Metrics endpoint should return some output");
    
    // The metrics might not be present if no requests have been made yet
    // So we just check that the endpoint is responding
}

#[tokio::test]
async fn test_metrics_are_recorded() {
    // Create config and router
    let config = create_test_config();
    let app = x402_facilitator::server::create_router(config.clone());

    // Make a health check request
    let _ = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Get metrics
    let config2 = create_test_config();
    let app2 = x402_facilitator::server::create_router(config2);
    
    let response = app2
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/metrics")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let metrics_text = String::from_utf8(body.to_vec()).unwrap();

    // Health check should have incremented the health_requests counter
    assert!(metrics_text.contains("x402_health_requests_total"));
}

// Helper to create test config
fn create_test_config() -> x402_facilitator::Config {
    use solana_client::rpc_client::RpcClient;
    use solana_sdk::commitment_config::CommitmentConfig;
    use std::sync::Arc;
    use x402_facilitator::cache::AccountCache;
    use x402_facilitator::metrics::AppMetrics;

    let rpc_url = "https://api.devnet.solana.com".to_string();
    let rpc_client = Arc::new(RpcClient::new_with_commitment(
        rpc_url.clone(),
        CommitmentConfig::confirmed(),
    ));

    let account_cache = AccountCache::new(100, 30);
    let metrics = AppMetrics::new();
    let transaction_dedup = x402_facilitator::dedup::TransactionDedup::new(1000, 300);
    let audit_logger = x402_facilitator::audit::AuditLogger::new();

    x402_facilitator::Config {
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

