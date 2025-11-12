/// Integration test for account caching functionality
use x402_facilitator::cache::AccountCache;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::account::Account;

#[tokio::test]
async fn test_cache_hit_and_miss() {
    let cache = AccountCache::new(100, 30);
    let pubkey = Pubkey::new_unique();
    
    // Initial cache miss
    assert!(cache.get(&pubkey).await.is_none());
    
    // Insert account
    let account = Account {
        lamports: 1_000_000,
        data: vec![],
        owner: Pubkey::new_unique(),
        executable: false,
        rent_epoch: 0,
    };
    cache.insert(pubkey, account.clone()).await;
    
    // Cache hit
    let cached = cache.get(&pubkey).await;
    assert!(cached.is_some());
    assert_eq!(cached.unwrap().lamports, 1_000_000);
}

#[tokio::test]
async fn test_cache_multiple_accounts() {
    let cache = AccountCache::new(100, 30);
    
    // Insert multiple accounts
    let mut accounts = vec![];
    for i in 0..10 {
        let pubkey = Pubkey::new_unique();
        let account = Account {
            lamports: i * 1_000_000,
            data: vec![],
            owner: Pubkey::new_unique(),
            executable: false,
            rent_epoch: 0,
        };
        cache.insert(pubkey, account.clone()).await;
        accounts.push((pubkey, account));
    }
    
    // Verify all accounts are cached
    for (pubkey, original) in accounts {
        let cached = cache.get(&pubkey).await;
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().lamports, original.lamports);
    }
}

#[tokio::test]
async fn test_cache_invalidation() {
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

