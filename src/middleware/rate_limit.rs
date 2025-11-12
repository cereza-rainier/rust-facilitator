use governor::{Quota, RateLimiter, clock::DefaultClock, state::{direct::NotKeyed, InMemoryState}};
use std::num::NonZeroU32;
use std::sync::Arc;

pub type DefaultDirectRateLimiter = RateLimiter<NotKeyed, InMemoryState, DefaultClock>;

/// Rate limiter state
#[derive(Clone)]
pub struct RateLimitState {
    pub limiter: Arc<DefaultDirectRateLimiter>,
}

impl RateLimitState {
    /// Create a new rate limiter
    /// 
    /// # Arguments
    /// * `per_second` - Number of requests allowed per second
    /// * `burst_size` - Maximum burst size
    pub fn new(per_second: u32, burst_size: u32) -> Self {
        let per_second_nz = NonZeroU32::new(per_second).unwrap_or(NonZeroU32::new(10).unwrap());
        let burst_size_nz = NonZeroU32::new(burst_size).unwrap_or(NonZeroU32::new(20).unwrap());
        
        let quota = Quota::per_second(per_second_nz)
            .allow_burst(burst_size_nz);
        
        let limiter = Arc::new(RateLimiter::direct(quota));
        
        tracing::info!(
            "✅ Rate limiter initialized: {} req/s, burst {}",
            per_second,
            burst_size
        );

        Self { limiter }
    }
    
    /// Check if a request is allowed
    pub fn check(&self) -> bool {
        self.limiter.check().is_ok()
    }
}

/// Check rate limit for a specific key
pub fn check_rate_limit(limiter: &DefaultDirectRateLimiter) -> bool {
    limiter.check().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_creation() {
        let state = RateLimitState::new(10, 20);
        assert!(state.limiter.check().is_ok());
    }

    #[test]
    fn test_rate_limiter_allows_within_limit() {
        let state = RateLimitState::new(10, 10);
        
        // Should allow up to burst size
        for _ in 0..10 {
            assert!(state.limiter.check().is_ok());
        }
    }

    #[test]
    fn test_rate_limiter_blocks_over_limit() {
        let state = RateLimitState::new(1, 2);
        
        // Use up the burst
        assert!(state.limiter.check().is_ok());
        assert!(state.limiter.check().is_ok());
        
        // Should be rate limited
        assert!(state.limiter.check().is_err());
    }
}

