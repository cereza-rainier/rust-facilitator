use hmac::{Hmac, Mac};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::time::Duration;

type HmacSha256 = Hmac<Sha256>;

/// Webhook configuration
#[derive(Clone, Debug)]
pub struct WebhookConfig {
    pub url: String,
    pub secret: String,
    pub enabled: bool,
    pub timeout_seconds: u64,
    pub retry_attempts: u32,
}

impl WebhookConfig {
    /// Load webhook configuration from environment
    pub fn from_env() -> Option<Self> {
        let enabled = std::env::var("WEBHOOK_ENABLED")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(false);

        if !enabled {
            return None;
        }

        Some(Self {
            url: std::env::var("WEBHOOK_URL").ok()?,
            secret: std::env::var("WEBHOOK_SECRET").ok()?,
            enabled: true,
            timeout_seconds: std::env::var("WEBHOOK_TIMEOUT_SECONDS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
            retry_attempts: std::env::var("WEBHOOK_RETRY_ATTEMPTS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3),
        })
    }
}

/// Webhook event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WebhookEvent {
    VerificationSuccess,
    VerificationFailure,
    SettlementSuccess,
    SettlementFailure,
}

impl WebhookEvent {
    pub fn as_str(&self) -> &str {
        match self {
            WebhookEvent::VerificationSuccess => "verification.success",
            WebhookEvent::VerificationFailure => "verification.failure",
            WebhookEvent::SettlementSuccess => "settlement.success",
            WebhookEvent::SettlementFailure => "settlement.failure",
        }
    }
}

/// Webhook payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    pub event: String,
    pub timestamp: i64,
    pub data: serde_json::Value,
}

impl WebhookPayload {
    pub fn new(event: WebhookEvent, data: serde_json::Value) -> Self {
        Self {
            event: event.as_str().to_string(),
            timestamp: chrono::Utc::now().timestamp(),
            data,
        }
    }
}

/// Send a webhook notification with retries
pub async fn send_webhook(
    config: &WebhookConfig,
    payload: &WebhookPayload,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if !config.enabled {
        return Ok(());
    }

    let client = Client::builder()
        .timeout(Duration::from_secs(config.timeout_seconds))
        .build()?;

    let payload_json = serde_json::to_string(payload)?;

    // Generate HMAC signature
    let signature = generate_signature(&config.secret, &payload_json)?;

    // Attempt delivery with retries
    let mut last_error = None;
    for attempt in 1..=config.retry_attempts {
        tracing::debug!(
            "Sending webhook to {} (attempt {}/{})",
            config.url,
            attempt,
            config.retry_attempts
        );

        match send_webhook_request(&client, &config.url, &payload_json, &signature).await {
            Ok(_) => {
                tracing::info!(
                    "✅ Webhook delivered successfully: {} to {}",
                    payload.event,
                    config.url
                );
                return Ok(());
            }
            Err(e) => {
                tracing::warn!(
                    "⚠️  Webhook delivery failed (attempt {}/{}): {}",
                    attempt,
                    config.retry_attempts,
                    e
                );
                last_error = Some(e);

                // Exponential backoff
                if attempt < config.retry_attempts {
                    let backoff_ms = 100 * 2u64.pow(attempt - 1);
                    tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
                }
            }
        }
    }

    // All retries failed
    if let Some(err) = last_error {
        tracing::error!(
            "❌ Webhook delivery failed after {} attempts: {}",
            config.retry_attempts,
            err
        );
        return Err(err);
    }

    Ok(())
}

/// Generate HMAC-SHA256 signature
fn generate_signature(secret: &str, payload: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())?;
    mac.update(payload.as_bytes());
    let result = mac.finalize();
    Ok(hex::encode(result.into_bytes()))
}

/// Send the actual HTTP request
async fn send_webhook_request(
    client: &Client,
    url: &str,
    payload: &str,
    signature: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("X-Webhook-Signature", signature)
        .header("User-Agent", "x402-facilitator/2.0")
        .body(payload.to_string())
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("HTTP {}: {}", response.status(), response.text().await?).into());
    }

    Ok(())
}

/// Verify webhook signature (for webhook receivers)
pub fn verify_signature(secret: &str, payload: &str, signature: &str) -> bool {
    match generate_signature(secret, payload) {
        Ok(expected) => expected == signature,
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_signature() {
        let secret = "test_secret";
        let payload = r#"{"event":"test","timestamp":1234567890,"data":{}}"#;

        let sig1 = generate_signature(secret, payload).unwrap();
        let sig2 = generate_signature(secret, payload).unwrap();

        // Same input should produce same signature
        assert_eq!(sig1, sig2);

        // Signature should be hex-encoded 64 chars (SHA256)
        assert_eq!(sig1.len(), 64);
    }

    #[test]
    fn test_verify_signature() {
        let secret = "test_secret";
        let payload = r#"{"event":"test","timestamp":1234567890,"data":{}}"#;

        let signature = generate_signature(secret, payload).unwrap();

        // Valid signature
        assert!(verify_signature(secret, payload, &signature));

        // Invalid signature
        assert!(!verify_signature(secret, payload, "invalid_signature"));

        // Wrong secret
        assert!(!verify_signature("wrong_secret", payload, &signature));
    }

    #[test]
    fn test_webhook_event_serialization() {
        let event = WebhookEvent::VerificationSuccess;
        assert_eq!(event.as_str(), "verification.success");

        let event = WebhookEvent::SettlementFailure;
        assert_eq!(event.as_str(), "settlement.failure");
    }

    #[test]
    fn test_webhook_payload_creation() {
        let data = serde_json::json!({"transaction": "abc123"});
        let payload = WebhookPayload::new(WebhookEvent::SettlementSuccess, data.clone());

        assert_eq!(payload.event, "settlement.success");
        assert_eq!(payload.data, data);
        assert!(payload.timestamp > 0);
    }
}

