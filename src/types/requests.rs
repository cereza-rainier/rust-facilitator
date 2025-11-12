use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Payment payload sent by client in X-PAYMENT header
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PaymentPayload {
    /// x402 protocol version (currently 1)
    #[schema(example = 1)]
    pub x402_version: u32,
    
    /// Payment scheme (e.g., "exact")
    #[schema(example = "exact")]
    pub scheme: String,
    
    /// Network identifier (e.g., "solana-devnet")
    #[schema(example = "solana-devnet")]
    pub network: String,
    
    /// SVM-specific payload
    pub payload: SvmPayload,
    
    /// Unix timestamp when payment was created (optional, for expiry validation)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = 1699000000)]
    pub timestamp: Option<u64>,
}

/// Solana-specific payload containing the partially-signed transaction
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct SvmPayload {
    /// Base64-encoded partially-signed Solana transaction
    #[schema(example = "AQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAEDArczbMia1tLmq7zz4DinMNN0pJ1JtLdqIJPUw3YrGCzYAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAgIAAQwCAAAAKgAAAAAAAAA=")]
    pub transaction: String,
}

/// Payment requirements sent by resource server
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PaymentRequirements {
    /// Payment scheme (must match payload scheme)
    #[schema(example = "exact")]
    pub scheme: String,
    
    /// Network identifier
    #[schema(example = "solana-devnet")]
    pub network: String,
    
    /// Maximum amount required in lamports
    #[schema(example = "1000000")]
    pub max_amount_required: String,
    
    /// Asset (token mint) address
    #[schema(example = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")]
    pub asset: String,
    
    /// Recipient address
    #[schema(example = "8VzycpqZpqYXMqKSZqYXMqKSZqYXMqKS")]
    pub pay_to: String,
    
    /// Resource path
    #[schema(example = "/api/resource")]
    pub resource: String,
    
    /// Human-readable description
    #[schema(example = "Premium API Access")]
    pub description: String,
    
    /// Response MIME type
    #[schema(example = "application/json")]
    pub mime_type: String,
    
    /// Maximum timeout in seconds
    #[schema(example = 30)]
    pub max_timeout_seconds: u64,
    
    /// Optional output schema
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<serde_json::Value>,
    
    /// Extra fields (contains fee payer)
    pub extra: ExtraFields,
}

/// Extra fields in payment requirements (contains fee payer)
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExtraFields {
    /// Fee payer public key
    #[schema(example = "FeePayerPublicKeyHere123456789")]
    pub fee_payer: String,
}

/// Request to /verify endpoint
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct VerifyRequest {
    /// Payment payload from client
    pub payment_payload: PaymentPayload,
    
    /// Payment requirements from server
    pub payment_requirements: PaymentRequirements,
}

/// Request to /settle endpoint
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct SettleRequest {
    /// Payment payload from client
    pub payment_payload: PaymentPayload,
    
    /// Payment requirements from server
    pub payment_requirements: PaymentRequirements,
}
