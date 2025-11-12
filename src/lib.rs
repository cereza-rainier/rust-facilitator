// Library exports for x402-facilitator
// This allows integration tests and external crates to use our modules

use utoipa::OpenApi;

pub mod audit;
pub mod cache;
pub mod config;
pub mod dedup;
pub mod error;
pub mod ffi;
pub mod metrics;
pub mod parallel;
pub mod types;
pub mod webhooks;

// WebAssembly module (only when targeting wasm32)
#[cfg(target_arch = "wasm32")]
pub mod wasm;

// Internal modules needed by server
pub mod handlers;
mod solana;
pub mod middleware;

// Server module needs handlers
pub mod server;

// Re-export commonly used items
pub use config::Config;
pub use error::{AppError, VerificationError};

/// OpenAPI documentation
#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::health::health_check,
        handlers::supported::supported,
        handlers::verify::verify,
        handlers::batch::verify_batch,
        handlers::settle::settle,
    ),
    components(
        schemas(
            types::requests::PaymentPayload,
            types::requests::SvmPayload,
            types::requests::PaymentRequirements,
            types::requests::ExtraFields,
            types::requests::VerifyRequest,
            types::requests::SettleRequest,
            types::responses::VerifyResponse,
            types::responses::SettleResponse,
            types::responses::SupportedResponse,
            types::responses::SchemeSupport,
        )
    ),
    tags(
        (name = "Health", description = "Health check endpoints"),
        (name = "Information", description = "Information endpoints"),
        (name = "Payment", description = "Payment verification and settlement endpoints")
    ),
    info(
        title = "x402 Rust Facilitator API",
        version = "2.0.0",
        description = "High-performance x402 payment facilitator for Solana blockchain",
        contact(
            name = "API Support",
            url = "https://github.com/yourname/x402-facilitator-rust"
        ),
        license(
            name = "Apache 2.0",
            url = "https://www.apache.org/licenses/LICENSE-2.0"
        )
    )
)]
pub struct ApiDoc;

