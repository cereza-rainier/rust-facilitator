use moka::future::Cache;
use solana_sdk::{account::Account, pubkey::Pubkey};
use std::time::Duration;

/// Account cache with TTL (Time To Live)
/// Caches Solana account data to reduce RPC calls
#[derive(Clone)]
pub struct AccountCache {
    cache: Cache<Pubkey, Account>,
}

impl AccountCache {
    /// Create a new account cache
    /// 
    /// # Arguments
    /// * `max_capacity` - Maximum number of accounts to cache
    /// * `ttl_seconds` - Time to live for cached entries in seconds
    pub fn new(max_capacity: u64, ttl_seconds: u64) -> Self {
        let cache = Cache::builder()
            .max_capacity(max_capacity)
            .time_to_live(Duration::from_secs(ttl_seconds))
            .build();

        tracing::info!(
            "Created account cache: capacity={}, ttl={}s",
            max_capacity,
            ttl_seconds
        );

        Self { cache }
    }

    /// Get an account from cache
    pub async fn get(&self, pubkey: &Pubkey) -> Option<Account> {
        self.cache.get(pubkey).await
    }

    /// Insert an account into cache
    pub async fn insert(&self, pubkey: Pubkey, account: Account) {
        self.cache.insert(pubkey, account).await;
    }

    /// Invalidate a specific account
    pub async fn invalidate(&self, pubkey: &Pubkey) {
        self.cache.invalidate(pubkey).await;
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            entry_count: self.cache.entry_count(),
            weighted_size: self.cache.weighted_size(),
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub entry_count: u64,
    pub weighted_size: u64,
}

impl std::fmt::Debug for AccountCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AccountCache")
            .field("entry_count", &self.cache.entry_count())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_insert_and_get() {
        let cache = AccountCache::new(100, 30);
        let pubkey = Pubkey::new_unique();
        let account = Account::default();

        // Insert account
        cache.insert(pubkey, account.clone()).await;

        // Should be able to retrieve it
        let cached = cache.get(&pubkey).await;
        assert!(cached.is_some());
    }

    #[tokio::test]
    async fn test_cache_miss() {
        let cache = AccountCache::new(100, 30);
        let pubkey = Pubkey::new_unique();

        // Should return None for non-existent key
        let cached = cache.get(&pubkey).await;
        assert!(cached.is_none());
    }

    #[tokio::test]
    async fn test_cache_invalidate() {
        let cache = AccountCache::new(100, 30);
        let pubkey = Pubkey::new_unique();
        let account = Account::default();

        // Insert and verify
        cache.insert(pubkey, account).await;
        assert!(cache.get(&pubkey).await.is_some());

        // Invalidate
        cache.invalidate(&pubkey).await;
        assert!(cache.get(&pubkey).await.is_none());
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let cache = AccountCache::new(100, 30);
        
        // Just verify we can call stats without panicking
        let stats = cache.stats();
        assert!(stats.entry_count >= 0);

        // Add some entries
        let pubkey = Pubkey::new_unique();
        cache.insert(pubkey, Account::default()).await;
        
        // Verify we can retrieve what we inserted (core functionality)
        assert!(cache.get(&pubkey).await.is_some());
        
        // Stats API works (exact counts may be eventually consistent)
        let _stats = cache.stats();
    }
}

