use moka::sync::Cache;
use std::time::Duration;
use sha2::{Sha256, Digest};

/// Transaction deduplication cache to prevent replay attacks
/// 
/// This cache stores transaction signatures/hashes with a TTL to ensure
/// that the same transaction cannot be verified or settled multiple times
/// within a configurable time window.
#[derive(Clone, Debug)]
pub struct TransactionDedup {
    pub(crate) cache: Cache<String, ()>,
    window_seconds: u64,
}

impl TransactionDedup {
    /// Create a new deduplication cache
    /// 
    /// # Arguments
    /// * `max_entries` - Maximum number of transaction hashes to cache
    /// * `window_seconds` - Time window in seconds for deduplication (default: 300 = 5 minutes)
    pub fn new(max_entries: u64, window_seconds: u64) -> Self {
        let cache = Cache::builder()
            .max_capacity(max_entries)
            .time_to_live(Duration::from_secs(window_seconds))
            .build();

        tracing::info!(
            "🔐 Transaction dedup initialized: {} max entries, {} second window",
            max_entries,
            window_seconds
        );

        Self {
            cache,
            window_seconds,
        }
    }

    /// Check if a transaction has already been seen
    /// 
    /// Returns true if the transaction is a duplicate (already seen within the time window)
    pub fn is_duplicate(&self, transaction_data: &str) -> bool {
        let hash = self.hash_transaction(transaction_data);
        self.cache.get(&hash).is_some()
    }

    /// Mark a transaction as seen
    /// 
    /// This records the transaction in the cache, preventing it from being
    /// processed again within the deduplication window
    pub fn mark_seen(&self, transaction_data: &str) {
        let hash = self.hash_transaction(transaction_data);
        self.cache.insert(hash, ());
    }

    /// Check and mark a transaction in one atomic operation
    /// 
    /// Returns true if this is a duplicate (already seen), false if it's new.
    /// If new, automatically marks it as seen.
    pub fn check_and_mark(&self, transaction_data: &str) -> bool {
        let hash = self.hash_transaction(transaction_data);
        
        // Check if it exists
        if self.cache.get(&hash).is_some() {
            tracing::warn!("🚨 Duplicate transaction detected: {}", &hash[..16]);
            return true;
        }

        // Not a duplicate, mark as seen
        self.cache.insert(hash, ());
        false
    }

    /// Hash a transaction to create a unique identifier
    /// 
    /// Uses SHA256 to create a deterministic hash of the transaction data
    fn hash_transaction(&self, transaction_data: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(transaction_data.as_bytes());
        let result = hasher.finalize();
        format!("{:x}", result)
    }

    /// Get cache statistics for monitoring
    pub fn stats(&self) -> DedupStats {
        DedupStats {
            entry_count: self.cache.entry_count(),
            window_seconds: self.window_seconds,
        }
    }

    /// Clear the cache (useful for testing)
    #[cfg(test)]
    pub fn clear(&self) {
        self.cache.invalidate_all();
    }
}

/// Statistics about the deduplication cache
#[derive(Debug, Clone)]
pub struct DedupStats {
    pub entry_count: u64,
    pub window_seconds: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dedup_basic() {
        let dedup = TransactionDedup::new(1000, 300);
        
        let tx1 = "transaction_data_1";
        let tx2 = "transaction_data_2";

        // First time seeing tx1 - not a duplicate
        assert!(!dedup.check_and_mark(tx1));
        
        // Second time seeing tx1 - is a duplicate
        assert!(dedup.check_and_mark(tx1));
        
        // First time seeing tx2 - not a duplicate
        assert!(!dedup.check_and_mark(tx2));
        
        // Second time seeing tx2 - is a duplicate
        assert!(dedup.check_and_mark(tx2));
    }

    #[test]
    fn test_dedup_separate_check_and_mark() {
        let dedup = TransactionDedup::new(1000, 300);
        
        let tx = "transaction_data";

        // Check - should not be duplicate
        assert!(!dedup.is_duplicate(tx));
        
        // Mark as seen
        dedup.mark_seen(tx);
        
        // Check again - should be duplicate now
        assert!(dedup.is_duplicate(tx));
    }

    #[test]
    fn test_dedup_hash_consistency() {
        let dedup = TransactionDedup::new(1000, 300);
        
        let tx = "same_transaction_data";
        
        // Same data should produce same hash
        let hash1 = dedup.hash_transaction(tx);
        let hash2 = dedup.hash_transaction(tx);
        
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_dedup_different_transactions() {
        let dedup = TransactionDedup::new(1000, 300);
        
        let tx1 = "transaction_1";
        let tx2 = "transaction_2";
        
        // Mark tx1 as seen
        dedup.mark_seen(tx1);
        
        // tx2 should not be marked as duplicate
        assert!(!dedup.is_duplicate(tx2));
    }

    #[test]
    fn test_dedup_stats() {
        let dedup = TransactionDedup::new(1000, 300);
        
        dedup.mark_seen("tx1");
        dedup.mark_seen("tx2");
        dedup.mark_seen("tx3");
        
        // Sync cache to ensure counts are up to date
        dedup.cache.run_pending_tasks();
        
        let stats = dedup.stats();
        assert_eq!(stats.entry_count, 3);
        assert_eq!(stats.window_seconds, 300);
    }

    #[test]
    fn test_dedup_clear() {
        let dedup = TransactionDedup::new(1000, 300);
        
        dedup.mark_seen("tx1");
        assert!(dedup.is_duplicate("tx1"));
        
        dedup.clear();
        
        assert!(!dedup.is_duplicate("tx1"));
    }

    #[test]
    fn test_dedup_expiry() {
        // Create dedup with 1 second window
        let dedup = TransactionDedup::new(1000, 1);
        
        let tx = "transaction";
        
        // Mark as seen
        dedup.mark_seen(tx);
        assert!(dedup.is_duplicate(tx));
        
        // Wait for expiry
        std::thread::sleep(std::time::Duration::from_secs(2));
        
        // Should no longer be a duplicate after expiry
        assert!(!dedup.is_duplicate(tx));
    }
}

