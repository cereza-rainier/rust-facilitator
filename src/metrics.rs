use prometheus::{
    register_histogram_vec, register_int_counter_vec, register_int_gauge, HistogramVec,
    IntCounterVec, IntGauge,
};
use lazy_static::lazy_static;

lazy_static! {
    static ref VERIFY_REQUESTS: IntCounterVec = register_int_counter_vec!(
        "x402_verify_requests_total",
        "Total number of verify requests",
        &["network"]
    ).expect("Failed to register verify_requests metric");

    static ref SETTLE_REQUESTS: IntCounterVec = register_int_counter_vec!(
        "x402_settle_requests_total",
        "Total number of settle requests",
        &["network", "status"]
    ).expect("Failed to register settle_requests metric");

    static ref HEALTH_REQUESTS: IntCounterVec = register_int_counter_vec!(
        "x402_health_requests_total",
        "Total number of health check requests",
        &[]
    ).expect("Failed to register health_requests metric");

    static ref CACHE_HITS: IntCounterVec = register_int_counter_vec!(
        "x402_cache_hits_total",
        "Total number of cache hits",
        &["account_type"]
    ).expect("Failed to register cache_hits metric");

    static ref CACHE_MISSES: IntCounterVec = register_int_counter_vec!(
        "x402_cache_misses_total",
        "Total number of cache misses",
        &["account_type"]
    ).expect("Failed to register cache_misses metric");

    static ref CACHE_SIZE: IntGauge = register_int_gauge!(
        "x402_cache_size",
        "Current number of entries in the account cache"
    ).expect("Failed to register cache_size metric");

    static ref VERIFICATION_SUCCESS: IntCounterVec = register_int_counter_vec!(
        "x402_verification_success_total",
        "Total number of successful verifications",
        &["network"]
    ).expect("Failed to register verification_success metric");

    static ref VERIFICATION_FAILURE: IntCounterVec = register_int_counter_vec!(
        "x402_verification_failure_total",
        "Total number of failed verifications",
        &["network", "reason"]
    ).expect("Failed to register verification_failure metric");

    static ref REQUEST_DURATION: HistogramVec = register_histogram_vec!(
        "x402_request_duration_seconds",
        "Request duration in seconds",
        &["endpoint", "method"],
        vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0]
    ).expect("Failed to register request_duration metric");

    static ref RPC_CALLS: IntCounterVec = register_int_counter_vec!(
        "x402_rpc_calls_total",
        "Total number of RPC calls",
        &["method"]
    ).expect("Failed to register rpc_calls metric");

    static ref RPC_ERRORS: IntCounterVec = register_int_counter_vec!(
        "x402_rpc_errors_total",
        "Total number of RPC errors",
        &["method", "error_type"]
    ).expect("Failed to register rpc_errors metric");
}

/// Application-specific metrics
#[derive(Clone)]
pub struct AppMetrics {
    // Request counters
    pub verify_requests: &'static IntCounterVec,
    pub settle_requests: &'static IntCounterVec,
    pub health_requests: &'static IntCounterVec,

    // Cache metrics
    pub cache_hits: &'static IntCounterVec,
    pub cache_misses: &'static IntCounterVec,
    pub cache_size: &'static IntGauge,

    // Verification metrics
    pub verification_success: &'static IntCounterVec,
    pub verification_failure: &'static IntCounterVec,

    // Latency metrics
    pub request_duration: &'static HistogramVec,

    // RPC metrics
    pub rpc_calls: &'static IntCounterVec,
    pub rpc_errors: &'static IntCounterVec,
}

impl AppMetrics {
    pub fn new() -> Self {
        tracing::info!("✅ Initialized Prometheus metrics");

        Self {
            verify_requests: &VERIFY_REQUESTS,
            settle_requests: &SETTLE_REQUESTS,
            health_requests: &HEALTH_REQUESTS,
            cache_hits: &CACHE_HITS,
            cache_misses: &CACHE_MISSES,
            cache_size: &CACHE_SIZE,
            verification_success: &VERIFICATION_SUCCESS,
            verification_failure: &VERIFICATION_FAILURE,
            request_duration: &REQUEST_DURATION,
            rpc_calls: &RPC_CALLS,
            rpc_errors: &RPC_ERRORS,
        }
    }

    /// Update cache size metric
    pub fn update_cache_size(&self, size: u64) {
        self.cache_size.set(size as i64);
    }

    /// Record a cache hit
    pub fn record_cache_hit(&self, account_type: &str) {
        self.cache_hits
            .with_label_values(&[account_type])
            .inc();
    }

    /// Record a cache miss
    pub fn record_cache_miss(&self, account_type: &str) {
        self.cache_misses
            .with_label_values(&[account_type])
            .inc();
    }

    /// Record verification success
    pub fn record_verification_success(&self, network: &str) {
        self.verification_success
            .with_label_values(&[network])
            .inc();
    }

    /// Record verification failure
    pub fn record_verification_failure(&self, network: &str, reason: &str) {
        self.verification_failure
            .with_label_values(&[network, reason])
            .inc();
    }
}

impl Default for AppMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Create Prometheus recorder
pub fn create_prometheus_handle() -> prometheus::Result<String> {
    use prometheus::Encoder;
    let encoder = prometheus::TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer)?;
    Ok(String::from_utf8(buffer).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_initialization() {
        let metrics = AppMetrics::new();
        
        // Verify we can update metrics
        metrics.record_cache_hit("test_account");
        metrics.record_cache_miss("test_account");
        metrics.update_cache_size(100);
        metrics.record_verification_success("solana-devnet");
        metrics.record_verification_failure("solana-devnet", "invalid_transaction");
    }
}

