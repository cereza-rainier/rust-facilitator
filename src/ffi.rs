// Foreign Function Interface (FFI) module
// Exposes core verification functions to other programming languages
// Compatible with: Python, Go, Java, Ruby, Node.js (N-API), C, C++, etc.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;

/// C-compatible verification result structure
/// 
/// This struct is guaranteed to have a stable memory layout (repr(C))
/// so it can be safely passed across FFI boundaries.
#[repr(C)]
pub struct CVerifyResult {
    /// Whether the payment is valid
    pub is_valid: bool,
    /// Error message (NULL if valid)
    /// Caller must free with x402_free_string()
    pub error_message: *mut c_char,
    /// Payer address (NULL if invalid)
    /// Caller must free with x402_free_string()
    pub payer: *mut c_char,
}

/// Initialize the FFI library
/// 
/// Call this once before using any other functions.
/// Safe to call multiple times.
/// 
/// # Returns
/// 0 on success, non-zero on error
#[no_mangle]
pub extern "C" fn x402_init() -> i32 {
    // Initialize tracing/logging (optional, for debugging)
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into())
        )
        .try_init();
    
    0 // Success
}

/// Free a C string allocated by Rust
/// 
/// Must be called for every string returned by x402_* functions.
/// 
/// # Safety
/// - Caller must ensure the pointer was allocated by Rust
/// - Must not use the pointer after calling this function
/// - Safe to call with NULL pointer (no-op)
#[no_mangle]
pub extern "C" fn x402_free_string(s: *mut c_char) {
    if s.is_null() {
        return;
    }
    unsafe {
        let _ = CString::from_raw(s);
        // Drops and frees the CString
    }
}

/// Free a CVerifyResult structure
/// 
/// Frees all strings within the result.
/// 
/// # Safety
/// - Caller must ensure the result was returned by x402_verify_payment
/// - Must not use the result after calling this function
#[no_mangle]
pub extern "C" fn x402_free_result(result: CVerifyResult) {
    x402_free_string(result.error_message);
    x402_free_string(result.payer);
}

/// Verify a payment from C-compatible JSON strings
/// 
/// This is a simplified verification that checks basic structure.
/// For full verification with RPC calls, use the async Rust API.
/// 
/// # Parameters
/// - `payment_json`: JSON string of PaymentPayload
/// - `requirements_json`: JSON string of PaymentRequirements
/// 
/// # Returns
/// CVerifyResult with is_valid and either error_message or payer
/// 
/// # Memory Management
/// Caller must call x402_free_result() to free the returned result.
/// 
/// # Safety
/// - Caller must ensure strings are valid UTF-8 and NULL-terminated
/// - Caller must not modify strings during function execution
/// - Returned strings must be freed with x402_free_string()
/// 
/// # Example (Python with ctypes)
/// ```python
/// lib = ctypes.CDLL("libx402_facilitator.so")
/// result = lib.x402_verify_payment(payment.encode(), requirements.encode())
/// if result.is_valid:
///     print(f"Payer: {result.payer.decode()}")
/// lib.x402_free_result(result)
/// ```
#[no_mangle]
pub extern "C" fn x402_verify_payment(
    payment_json: *const c_char,
    requirements_json: *const c_char,
) -> CVerifyResult {
    // 1. Validate pointers
    if payment_json.is_null() {
        return error_result("Null payment pointer");
    }
    if requirements_json.is_null() {
        return error_result("Null requirements pointer");
    }

    // 2. Convert C strings to Rust strings
    let payment_str = unsafe {
        match CStr::from_ptr(payment_json).to_str() {
            Ok(s) => s,
            Err(_) => return error_result("Invalid UTF-8 in payment"),
        }
    };

    let requirements_str = unsafe {
        match CStr::from_ptr(requirements_json).to_str() {
            Ok(s) => s,
            Err(_) => return error_result("Invalid UTF-8 in requirements"),
        }
    };

    // 3. Parse JSON
    let payment: crate::types::requests::PaymentPayload = 
        match serde_json::from_str(payment_str) {
            Ok(p) => p,
            Err(e) => return error_result(&format!("Payment JSON parse error: {}", e)),
        };

    let requirements: crate::types::requests::PaymentRequirements = 
        match serde_json::from_str(requirements_str) {
            Ok(r) => r,
            Err(e) => return error_result(&format!("Requirements JSON parse error: {}", e)),
        };

    // 4. Perform basic validation (without RPC calls)
    if payment.scheme != requirements.scheme {
        return error_result("Scheme mismatch");
    }

    if payment.scheme != "exact" {
        return error_result("Only 'exact' scheme is supported");
    }

    if payment.network != requirements.network {
        return error_result("Network mismatch");
    }

    if !payment.network.starts_with("solana") {
        return error_result("Only Solana networks are supported");
    }

    // 5. Decode transaction to extract payer
    let transaction_base64 = &payment.payload.transaction;
    
    match crate::solana::decoder::decode_transaction_from_base64(transaction_base64) {
        Ok(tx) => {
            // Extract payer (second account key, index 1)
            let payer = if let Some(payer_key) = tx.message.account_keys.get(1) {
                payer_key.to_string()
            } else {
                "unknown".to_string()
            };

            // Success!
            CVerifyResult {
                is_valid: true,
                error_message: ptr::null_mut(),
                payer: CString::new(payer)
                    .expect("Failed to create payer CString")
                    .into_raw(),
            }
        }
        Err(e) => {
            error_result(&format!("Failed to decode transaction: {}", e))
        }
    }
}

/// Helper function to create error result
fn error_result(msg: &str) -> CVerifyResult {
    CVerifyResult {
        is_valid: false,
        error_message: CString::new(msg)
            .expect("Failed to create error CString")
            .into_raw(),
        payer: ptr::null_mut(),
    }
}

/// Get the library version
/// 
/// Returns a static string (does not need to be freed).
/// 
/// # Returns
/// Version string in format "major.minor.patch"
#[no_mangle]
pub extern "C" fn x402_version() -> *const c_char {
    // Static string - caller must NOT free this
    b"2.0.0\0".as_ptr() as *const c_char
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_init() {
        let result = x402_init();
        assert_eq!(result, 0);
    }

    #[test]
    fn test_version() {
        let version_ptr = x402_version();
        assert!(!version_ptr.is_null());
        
        let version = unsafe { CStr::from_ptr(version_ptr) };
        assert_eq!(version.to_str().unwrap(), "2.0.0");
    }

    #[test]
    fn test_free_null_string() {
        // Should not crash
        x402_free_string(ptr::null_mut());
    }

    #[test]
    fn test_verify_null_pointers() {
        let result = x402_verify_payment(ptr::null(), ptr::null());
        assert!(!result.is_valid);
        assert!(!result.error_message.is_null());
        
        // Cleanup
        x402_free_result(result);
    }

    #[test]
    fn test_verify_invalid_json() {
        let payment = CString::new("invalid json").unwrap();
        let requirements = CString::new("{}").unwrap();
        
        let result = x402_verify_payment(
            payment.as_ptr(),
            requirements.as_ptr()
        );
        
        assert!(!result.is_valid);
        assert!(!result.error_message.is_null());
        
        // Cleanup
        x402_free_result(result);
    }

    #[test]
    fn test_verify_scheme_mismatch() {
        let payment = CString::new(r#"{
            "x402_version": 1,
            "scheme": "exact",
            "network": "solana-devnet",
            "payload": {"transaction": "test"}
        }"#).unwrap();
        
        let requirements = CString::new(r#"{
            "scheme": "random",
            "network": "solana-devnet",
            "max_amount_required": "1000000",
            "asset": "SOL",
            "pay_to": "test",
            "resource": "/test",
            "description": "test",
            "mime_type": "application/json",
            "max_timeout_seconds": 30,
            "extra": {"fee_payer": "test"}
        }"#).unwrap();
        
        let result = x402_verify_payment(
            payment.as_ptr(),
            requirements.as_ptr()
        );
        
        assert!(!result.is_valid);
        
        // Cleanup
        x402_free_result(result);
    }
}

