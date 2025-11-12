use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use std::sync::Arc;
use crate::audit::AuditLogger;
use crate::cache::AccountCache;
use crate::dedup::TransactionDedup;
use crate::metrics::AppMetrics;
use crate::middleware::rate_limit::RateLimitState;
use crate::webhooks::WebhookConfig;

#[derive(Clone)]
pub struct Config {
    pub solana_rpc_url: String,
    pub fee_payer_private_key: String,
    pub network: String,
    pub port: u16,
    pub rpc_client: Arc<RpcClient>,
    pub account_cache: AccountCache,
    pub metrics: AppMetrics,
    pub rate_limiter: Option<RateLimitState>,
    pub webhook: Option<WebhookConfig>,
    pub transaction_dedup: TransactionDedup,
    pub payment_expiry_seconds: u64,
    pub audit_logger: AuditLogger,
}

// Manual Debug implementation since RpcClient doesn't implement Debug
impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("solana_rpc_url", &self.solana_rpc_url)
            .field("fee_payer_private_key", &"[REDACTED]")
            .field("network", &self.network)
            .field("port", &self.port)
            .field("rpc_client", &"Arc<RpcClient>")
            .field("account_cache", &self.account_cache)
            .field("metrics", &"AppMetrics")
            .field("rate_limiter", &self.rate_limiter.is_some())
            .field("webhook", &self.webhook.is_some())
            .field("transaction_dedup", &"TransactionDedup")
            .field("payment_expiry_seconds", &self.payment_expiry_seconds)
            .field("audit_logger", &"AuditLogger")
            .finish()
    }
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        let solana_rpc_url = std::env::var("SOLANA_RPC_URL")
            .unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());

        // Create shared RPC client for connection pooling
        let rpc_client = Arc::new(RpcClient::new_with_commitment(
            solana_rpc_url.clone(),
            CommitmentConfig::confirmed(),
        ));

        tracing::info!("✅ Created shared RPC client for: {}", solana_rpc_url);

        // Create account cache with configurable parameters
        let cache_size = std::env::var("CACHE_SIZE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1000);
        
        let cache_ttl = std::env::var("CACHE_TTL_SECONDS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(30);

        let account_cache = AccountCache::new(cache_size, cache_ttl);

        // Initialize metrics
        let metrics = AppMetrics::new();

        // Initialize rate limiter if configured
        let rate_limiter = if std::env::var("ENABLE_RATE_LIMIT").unwrap_or_else(|_| "true".to_string()) == "true" {
            let per_second = std::env::var("RATE_LIMIT_PER_SECOND")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10);
            
            let burst_size = std::env::var("RATE_LIMIT_BURST_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(20);

            Some(RateLimitState::new(per_second, burst_size))
        } else {
            tracing::info!("⚠️  Rate limiting disabled");
            None
        };

        // Load webhook configuration
        let webhook = WebhookConfig::from_env();
        if webhook.is_some() {
            tracing::info!("🔔 Webhooks enabled");
        }

        // Initialize transaction deduplication
        let dedup_max_entries = std::env::var("DEDUP_MAX_ENTRIES")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(10000);
        
        let dedup_window_seconds = std::env::var("DEDUP_WINDOW_SECONDS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(300); // 5 minutes default

        let transaction_dedup = TransactionDedup::new(dedup_max_entries, dedup_window_seconds);

        // Payment expiry time
        let payment_expiry_seconds = std::env::var("PAYMENT_EXPIRY_SECONDS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(600); // 10 minutes default
        
        tracing::info!("⏰ Payment expiry set to {} seconds", payment_expiry_seconds);

        // Initialize audit logger
        let audit_logger = AuditLogger::new();

        let config = Config {
            solana_rpc_url,
            fee_payer_private_key: std::env::var("FEE_PAYER_PRIVATE_KEY")
                .expect("FEE_PAYER_PRIVATE_KEY must be set"),
            network: std::env::var("NETWORK")
                .unwrap_or_else(|_| "solana-devnet".to_string()),
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .expect("PORT must be a valid number"),
            rpc_client,
            account_cache,
            metrics,
            rate_limiter,
            webhook,
            transaction_dedup,
            payment_expiry_seconds,
            audit_logger,
        };

        // Validate configuration
        config.validate()?;

        Ok(config)
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        tracing::info!("🔍 Validating configuration...");

        // Validate RPC URL format
        if !self.solana_rpc_url.starts_with("http://") && 
           !self.solana_rpc_url.starts_with("https://") {
            anyhow::bail!("Invalid RPC URL format: must start with http:// or https://");
        }

        // Validate private key is not empty
        if self.fee_payer_private_key.is_empty() || 
           self.fee_payer_private_key == "your_base58_private_key_here" {
            anyhow::bail!("Fee payer private key is not configured");
        }

        // Validate network value
        let valid_networks = ["solana", "solana-devnet", "solana-testnet"];
        if !valid_networks.contains(&self.network.as_str()) {
            anyhow::bail!("Invalid network: {} (must be one of: {:?})", self.network, valid_networks);
        }

        // Validate port range
        if self.port < 1024 || self.port > 65535 {
            tracing::warn!("⚠️  Port {} is outside recommended range (1024-65535)", self.port);
        }

        // Test RPC connection
        tracing::info!("🔍 Testing RPC connection...");
        match self.rpc_client.get_health() {
            Ok(_) => tracing::info!("✅ RPC connection validated"),
            Err(e) => {
                tracing::warn!("⚠️  RPC connection test failed: {}", e);
                tracing::warn!("⚠️  Proceeding anyway, but RPC might be unavailable");
                // Don't fail startup, but log warning
            }
        }

        tracing::info!("✅ Configuration validated");
        Ok(())
    }
}

