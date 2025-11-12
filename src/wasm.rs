// WebAssembly module - enables browser-based payment verification
// This allows the facilitator to run entirely client-side with zero server dependency

#![cfg(target_arch = "wasm32")]

use wasm_bindgen::prelude::*;
use crate::types::{requests::PaymentPayload, requests::PaymentRequirements, responses::VerifyResponse};
use crate::solana::decoder::decode_transaction_from_base64;

/// Initialize panic hook for better debugging in the browser
#[wasm_bindgen(start)]
pub fn init_wasm() {
    console_error_panic_hook::set_once();
    web_sys::console::log_1(&"🦀 x402 WASM module initialized!".into());
}

/// WASM-friendly verifier
/// 
/// This struct provides payment verification that runs entirely in the browser.
/// No server required - perfect for:
/// - Decentralized applications
/// - Offline-first apps
/// - Browser extensions
/// - Progressive Web Apps
/// 
/// # Example (JavaScript)
/// ```javascript
/// import init, { WasmVerifier } from './x402_facilitator.js';
/// 
/// await init();
/// const verifier = WasmVerifier.new();
/// 
/// const result = verifier.verify(payment, requirements);
/// console.log(result.is_valid);
/// ```
#[wasm_bindgen]
pub struct WasmVerifier {
    // Lightweight state - no RPC client, no async runtime
}

#[wasm_bindgen]
impl WasmVerifier {
    /// Create a new WASM verifier
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        WasmVerifier {}
    }

    /// Verify a payment from JavaScript objects
    /// 
    /// This performs client-side verification without any network calls.
    /// Suitable for basic validation before submitting to a facilitator.
    /// 
    /// # Parameters
    /// - `payment_js`: JavaScript object matching PaymentPayload schema
    /// - `requirements_js`: JavaScript object matching PaymentRequirements schema
    /// 
    /// # Returns
    /// JavaScript object with `is_valid`, `invalid_reason`, and `payer` fields
    /// 
    /// # Example
    /// ```javascript
    /// const payment = {
    ///   x402_version: 1,
    ///   scheme: "exact",
    ///   network: "solana-devnet",
    ///   payload: { transaction: "base64..." }
    /// };
    /// 
    /// const requirements = {
    ///   scheme: "exact",
    ///   network: "solana-devnet",
    ///   max_amount_required: "1000000",
    ///   // ... other fields
    /// };
    /// 
    /// const result = verifier.verify(payment, requirements);
    /// if (result.is_valid) {
    ///   console.log(`Payer: ${result.payer}`);
    /// }
    /// ```
    #[wasm_bindgen]
    pub fn verify(&self, payment_js: JsValue, requirements_js: JsValue) -> JsValue {
        // Convert JS values to Rust types
        let payment: PaymentPayload = match serde_wasm_bindgen::from_value(payment_js) {
            Ok(p) => p,
            Err(e) => {
                return serde_wasm_bindgen::to_value(&VerifyResponse {
                    is_valid: false,
                    invalid_reason: Some(format!("Invalid payment format: {}", e)),
                    payer: None,
                })
                .unwrap();
            }
        };

        let requirements: PaymentRequirements = match serde_wasm_bindgen::from_value(requirements_js) {
            Ok(r) => r,
            Err(e) => {
                return serde_wasm_bindgen::to_value(&VerifyResponse {
                    is_valid: false,
                    invalid_reason: Some(format!("Invalid requirements format: {}", e)),
                    payer: None,
                })
                .unwrap();
            }
        };

        // Perform WASM-safe verification
        let result = verify_wasm_safe(&payment, &requirements);

        // Convert back to JS
        serde_wasm_bindgen::to_value(&result).unwrap()
    }

    /// Get the library version
    #[wasm_bindgen]
    pub fn version(&self) -> String {
        "2.0.0-wasm".to_string()
    }

    /// Check if a scheme is supported
    #[wasm_bindgen]
    pub fn supports_scheme(&self, scheme: String) -> bool {
        scheme == "exact"
    }

    /// Check if a network is supported
    #[wasm_bindgen]
    pub fn supports_network(&self, network: String) -> bool {
        network == "solana" || network == "solana-devnet"
    }
}

/// WASM-safe verification logic
/// 
/// This performs verification without any I/O operations:
/// - No file system access
/// - No network calls
/// - No async operations
/// - Pure computational verification
/// 
/// Perfect for running in the browser sandbox.
fn verify_wasm_safe(
    payment: &PaymentPayload,
    requirements: &PaymentRequirements,
) -> VerifyResponse {
    // 1. Verify scheme match
    if payment.scheme != requirements.scheme {
        return VerifyResponse {
            is_valid: false,
            invalid_reason: Some(format!(
                "Scheme mismatch: payment uses '{}', requirements specify '{}'",
                payment.scheme, requirements.scheme
            )),
            payer: None,
        };
    }

    // 2. Verify scheme is supported
    if payment.scheme != "exact" {
        return VerifyResponse {
            is_valid: false,
            invalid_reason: Some(format!(
                "Unsupported scheme: '{}'. Only 'exact' is supported.",
                payment.scheme
            )),
            payer: None,
        };
    }

    // 3. Verify network match
    if payment.network != requirements.network {
        return VerifyResponse {
            is_valid: false,
            invalid_reason: Some(format!(
                "Network mismatch: payment uses '{}', requirements specify '{}'",
                payment.network, requirements.network
            )),
            payer: None,
        };
    }

    // 4. Verify network is supported
    if !payment.network.starts_with("solana") {
        return VerifyResponse {
            is_valid: false,
            invalid_reason: Some(format!(
                "Unsupported network: '{}'. Only Solana networks are supported.",
                payment.network
            )),
            payer: None,
        };
    }

    // 5. Verify timestamp (if present)
    if let Some(timestamp) = payment.timestamp {
        // Get current time (WASM-compatible)
        let current_time = js_sys::Date::now() / 1000.0;
        let age_seconds = current_time as u64 - timestamp;
        
        // Default expiry: 10 minutes
        let max_age = 600;
        
        if age_seconds > max_age {
            return VerifyResponse {
                is_valid: false,
                invalid_reason: Some(format!(
                    "Payment expired: age {} seconds exceeds maximum {} seconds",
                    age_seconds, max_age
                )),
                payer: None,
            };
        }
    }

    // 6. Decode transaction to extract payer
    let transaction_base64 = &payment.payload.transaction;
    
    match decode_transaction_from_base64(transaction_base64) {
        Ok(tx) => {
            // Extract payer (second account key, index 1)
            let payer = if let Some(payer_key) = tx.message.account_keys.get(1) {
                payer_key.to_string()
            } else {
                "unknown".to_string()
            };

            // Basic instruction count check
            let instruction_count = tx.message.instructions.len();
            if instruction_count < 3 || instruction_count > 4 {
                return VerifyResponse {
                    is_valid: false,
                    invalid_reason: Some(format!(
                        "Invalid instruction count: expected 3 or 4, got {}",
                        instruction_count
                    )),
                    payer: None,
                };
            }

            // Success!
            VerifyResponse {
                is_valid: true,
                invalid_reason: None,
                payer: Some(payer),
            }
        }
        Err(e) => VerifyResponse {
            is_valid: false,
            invalid_reason: Some(format!("Failed to decode transaction: {}", e)),
            payer: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_verifier_creation() {
        let verifier = WasmVerifier::new();
        assert_eq!(verifier.version(), "2.0.0-wasm");
    }

    #[test]
    fn test_scheme_support() {
        let verifier = WasmVerifier::new();
        assert!(verifier.supports_scheme("exact".to_string()));
        assert!(!verifier.supports_scheme("random".to_string()));
    }

    #[test]
    fn test_network_support() {
        let verifier = WasmVerifier::new();
        assert!(verifier.supports_network("solana".to_string()));
        assert!(verifier.supports_network("solana-devnet".to_string()));
        assert!(!verifier.supports_network("ethereum".to_string()));
    }
}

