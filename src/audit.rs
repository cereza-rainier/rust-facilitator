use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;
use chrono::{DateTime, Utc};

/// Audit event types for compliance and debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventType {
    /// Payment verification requested
    VerificationRequested,
    /// Payment verification succeeded
    VerificationSuccess,
    /// Payment verification failed
    VerificationFailed,
    /// Settlement requested
    SettlementRequested,
    /// Settlement succeeded
    SettlementSuccess,
    /// Settlement failed
    SettlementFailed,
    /// Duplicate transaction detected (replay attack)
    DuplicateDetected,
    /// Payment expired
    PaymentExpired,
    /// Rate limit exceeded
    RateLimitExceeded,
    /// Server started
    ServerStarted,
    /// Server stopped
    ServerStopped,
    /// Configuration changed
    ConfigChanged,
}

/// Structured audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Unique event ID
    pub id: String,
    /// Event type
    pub event_type: AuditEventType,
    /// Timestamp (UTC)
    pub timestamp: DateTime<Utc>,
    /// Transaction signature (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_signature: Option<String>,
    /// Payer address (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payer: Option<String>,
    /// Network (solana, solana-devnet, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<String>,
    /// Amount in lamports (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<u64>,
    /// Recipient address (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipient: Option<String>,
    /// Error message (if failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Additional context (free-form JSON)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl AuditEvent {
    /// Create a new audit event
    pub fn new(event_type: AuditEventType) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            event_type,
            timestamp: Utc::now(),
            transaction_signature: None,
            payer: None,
            network: None,
            amount: None,
            recipient: None,
            error: None,
            metadata: None,
        }
    }

    /// Builder method to add transaction signature
    pub fn with_transaction(mut self, signature: String) -> Self {
        self.transaction_signature = Some(signature);
        self
    }

    /// Builder method to add payer
    pub fn with_payer(mut self, payer: String) -> Self {
        self.payer = Some(payer);
        self
    }

    /// Builder method to add network
    pub fn with_network(mut self, network: String) -> Self {
        self.network = Some(network);
        self
    }

    /// Builder method to add amount
    pub fn with_amount(mut self, amount: u64) -> Self {
        self.amount = Some(amount);
        self
    }

    /// Builder method to add recipient
    pub fn with_recipient(mut self, recipient: String) -> Self {
        self.recipient = Some(recipient);
        self
    }

    /// Builder method to add error
    pub fn with_error(mut self, error: String) -> Self {
        self.error = Some(error);
        self
    }

    /// Builder method to add metadata
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// Audit logger - async writer to file/database
#[derive(Clone)]
pub struct AuditLogger {
    sender: Arc<mpsc::UnboundedSender<AuditEvent>>,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new() -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel::<AuditEvent>();

        // Spawn background task to write audit logs
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                // Log as structured JSON
                let json = serde_json::to_string(&event).unwrap_or_else(|e| {
                    format!(r#"{{"error":"Failed to serialize audit event: {}"}}"#, e)
                });
                
                // Write to stdout (can be captured by logging infrastructure)
                // In production, could write to file, database, or external service
                tracing::info!(target: "audit", "{}", json);
            }
        });

        tracing::info!("📋 Audit logging initialized");

        Self {
            sender: Arc::new(tx),
        }
    }

    /// Log an audit event (non-blocking)
    pub fn log(&self, event: AuditEvent) {
        if let Err(e) = self.sender.send(event) {
            tracing::error!("Failed to send audit event: {}", e);
        }
    }

    /// Log a verification request
    pub fn log_verification_request(&self, network: &str, payer: Option<&str>) {
        let mut event = AuditEvent::new(AuditEventType::VerificationRequested)
            .with_network(network.to_string());
        
        if let Some(p) = payer {
            event = event.with_payer(p.to_string());
        }
        
        self.log(event);
    }

    /// Log a verification success
    pub fn log_verification_success(&self, network: &str, payer: &str, transaction: Option<&str>) {
        let mut event = AuditEvent::new(AuditEventType::VerificationSuccess)
            .with_network(network.to_string())
            .with_payer(payer.to_string());
        
        if let Some(tx) = transaction {
            event = event.with_transaction(tx.to_string());
        }
        
        self.log(event);
    }

    /// Log a verification failure
    pub fn log_verification_failure(&self, network: &str, error: &str, payer: Option<&str>) {
        let mut event = AuditEvent::new(AuditEventType::VerificationFailed)
            .with_network(network.to_string())
            .with_error(error.to_string());
        
        if let Some(p) = payer {
            event = event.with_payer(p.to_string());
        }
        
        self.log(event);
    }

    /// Log a settlement success
    pub fn log_settlement_success(&self, network: &str, signature: &str, payer: &str, amount: u64) {
        let event = AuditEvent::new(AuditEventType::SettlementSuccess)
            .with_network(network.to_string())
            .with_transaction(signature.to_string())
            .with_payer(payer.to_string())
            .with_amount(amount);
        
        self.log(event);
    }

    /// Log a settlement failure
    pub fn log_settlement_failure(&self, network: &str, error: &str, payer: Option<&str>) {
        let mut event = AuditEvent::new(AuditEventType::SettlementFailed)
            .with_network(network.to_string())
            .with_error(error.to_string());
        
        if let Some(p) = payer {
            event = event.with_payer(p.to_string());
        }
        
        self.log(event);
    }

    /// Log a duplicate transaction detection (replay attack)
    pub fn log_duplicate_detected(&self, network: &str, transaction: &str) {
        let event = AuditEvent::new(AuditEventType::DuplicateDetected)
            .with_network(network.to_string())
            .with_transaction(transaction.to_string());
        
        self.log(event);
    }

    /// Log a payment expiry
    pub fn log_payment_expired(&self, network: &str, age_seconds: u64) {
        let event = AuditEvent::new(AuditEventType::PaymentExpired)
            .with_network(network.to_string())
            .with_metadata(serde_json::json!({
                "age_seconds": age_seconds
            }));
        
        self.log(event);
    }

    /// Log server startup
    pub fn log_server_started(&self, port: u16, network: &str) {
        let event = AuditEvent::new(AuditEventType::ServerStarted)
            .with_network(network.to_string())
            .with_metadata(serde_json::json!({
                "port": port
            }));
        
        self.log(event);
    }

    /// Log server shutdown
    pub fn log_server_stopped(&self) {
        let event = AuditEvent::new(AuditEventType::ServerStopped);
        self.log(event);
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_event_creation() {
        let event = AuditEvent::new(AuditEventType::VerificationSuccess)
            .with_payer("test_payer".to_string())
            .with_network("solana-devnet".to_string())
            .with_amount(1000000);

        assert_eq!(event.payer, Some("test_payer".to_string()));
        assert_eq!(event.network, Some("solana-devnet".to_string()));
        assert_eq!(event.amount, Some(1000000));
    }

    #[test]
    fn test_audit_event_serialization() {
        let event = AuditEvent::new(AuditEventType::VerificationFailed)
            .with_error("Invalid signature".to_string())
            .with_network("solana-devnet".to_string());

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("verification_failed"));
        assert!(json.contains("Invalid signature"));
    }

    #[tokio::test]
    async fn test_audit_logger() {
        let logger = AuditLogger::new();
        
        // Log a few events
        logger.log_verification_request("solana-devnet", Some("test_payer"));
        logger.log_verification_success("solana-devnet", "test_payer", None);
        logger.log_verification_failure("solana-devnet", "Test error", Some("test_payer"));

        // Give the background task time to process
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
}


