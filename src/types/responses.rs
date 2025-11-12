use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Response from /verify endpoint
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct VerifyResponse {
    /// Whether the payment is valid
    #[schema(example = true)]
    pub is_valid: bool,
    
    /// Reason if invalid
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "invalid_amount")]
    pub invalid_reason: Option<String>,
    
    /// Payer public key if valid
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "PayerPublicKey123456789")]
    pub payer: Option<String>,
}

/// Response from /settle endpoint
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SettleResponse {
    /// Whether settlement was successful
    #[schema(example = true)]
    pub success: bool,
    
    /// Network where transaction was settled
    #[schema(example = "solana-devnet")]
    pub network: String,
    
    /// Transaction signature
    #[schema(example = "5j7s6NiJS3JAkvgkoc18WVAsiSaci2pxB2A6ueCJP4tprA2TFg9wSyTLeYouxPBJEMzJinENTkpA52YStRW5Dia7")]
    pub transaction: String,
    
    /// Payer public key if successful
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "PayerPublicKey123456789")]
    pub payer: Option<String>,
    
    /// Error reason if failed
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "transaction_failed")]
    pub error_reason: Option<String>,
}

/// Response from /supported endpoint
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SupportedResponse {
    /// List of supported schemes
    pub schemes: Vec<SchemeSupport>,
}

/// Information about a supported scheme
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SchemeSupport {
    /// Scheme name
    #[schema(example = "exact")]
    pub scheme: String,
    
    /// Supported networks
    #[schema(example = json!(["solana-devnet", "solana", "solana-testnet"]))]
    pub networks: Vec<String>,
}
