use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use crate::config::Config;

/// Detailed health check with system information
#[derive(Serialize, Deserialize)]
pub struct HealthDetail {
    pub status: String,
    pub version: String,
    pub network: String,
    pub rpc_url: String,
    pub rpc_status: String,
    pub features: HealthFeatures,
    pub cache: CacheInfo,
}

#[derive(Serialize, Deserialize)]
pub struct HealthFeatures {
    pub rate_limiting: bool,
    pub caching: bool,
    pub metrics: bool,
}

#[derive(Serialize, Deserialize)]
pub struct CacheInfo {
    pub entries: u64,
    pub size: u64,
}

/// GET /admin/health - Detailed health check
pub async fn detailed_health(State(config): State<Config>) -> Json<HealthDetail> {
    // Check RPC connection
    let rpc_status = match config.rpc_client.get_health() {
        Ok(_) => "healthy".to_string(),
        Err(e) => format!("unhealthy: {}", e),
    };

    // Get cache stats
    let cache_stats = config.account_cache.stats();

    let health = HealthDetail {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        network: config.network.clone(),
        rpc_url: config.solana_rpc_url.clone(),
        rpc_status,
        features: HealthFeatures {
            rate_limiting: config.rate_limiter.is_some(),
            caching: true,
            metrics: true,
        },
        cache: CacheInfo {
            entries: cache_stats.entry_count,
            size: cache_stats.weighted_size,
        },
    };

    Json(health)
}

/// Stats for monitoring
#[derive(Serialize, Deserialize)]
pub struct Stats {
    pub uptime_info: String,
    pub version: String,
    pub network: String,
    pub cache_stats: CacheStatsDetail,
}

#[derive(Serialize, Deserialize)]
pub struct CacheStatsDetail {
    pub entries: u64,
    pub size: u64,
}

/// GET /admin/stats - System statistics
pub async fn get_stats(State(config): State<Config>) -> Json<Stats> {
    let cache_stats = config.account_cache.stats();

    let stats = Stats {
        uptime_info: "See metrics endpoint for detailed stats".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        network: config.network.clone(),
        cache_stats: CacheStatsDetail {
            entries: cache_stats.entry_count,
            size: cache_stats.weighted_size,
        },
    };

    Json(stats)
}

/// GET /admin/config - Configuration info (redacted)
pub async fn get_config(State(config): State<Config>) -> Json<Value> {
    Json(json!({
        "network": config.network,
        "rpc_url": config.solana_rpc_url,
        "port": config.port,
        "features": {
            "rate_limiting": config.rate_limiter.is_some(),
            "caching": true,
            "metrics": true,
        }
    }))
}

