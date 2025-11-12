use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;

/// Wrapper for Solana RPC client
pub struct SolanaClient {
    client: RpcClient,
}

impl SolanaClient {
    /// Create a new Solana RPC client
    pub fn new(rpc_url: &str) -> Self {
        let client = RpcClient::new_with_commitment(
            rpc_url.to_string(),
            CommitmentConfig::confirmed(),
        );

        Self { client }
    }

    /// Get the underlying RPC client
    pub fn client(&self) -> &RpcClient {
        &self.client
    }
}

impl Clone for SolanaClient {
    fn clone(&self) -> Self {
        Self::new(&self.client.url())
    }
}
