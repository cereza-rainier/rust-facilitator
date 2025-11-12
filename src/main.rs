use std::net::SocketAddr;
use std::time::Duration;
use tokio::signal;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Use library from lib.rs
use x402_facilitator::{config, server};

/// Graceful shutdown handler
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("🛑 Received Ctrl+C signal");
        }
        _ = terminate => {
            tracing::info!("🛑 Received terminate signal");
        }
    }
    
    tracing::info!("⏳ Starting graceful shutdown...");
    tracing::info!("   Waiting for in-flight requests to complete (max 10s)");
    
    // Give in-flight requests time to complete
    tokio::time::sleep(Duration::from_secs(10)).await;
    
    tracing::info!("✅ Shutdown complete");
}

#[tokio::main]
async fn main() {
    // Initialize structured logging with environment filter
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "x402_facilitator=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load config
    let config = config::Config::from_env().expect("Failed to load config");

    tracing::info!("🚀 Starting x402 Rust Facilitator v{}", env!("CARGO_PKG_VERSION"));
    tracing::info!("📡 Network: {}", config.network);
    tracing::info!("🔗 RPC: {}", config.solana_rpc_url);
    if config.rate_limiter.is_some() {
        tracing::info!("🛡️  Rate limiting: enabled");
    } else {
        tracing::info!("⚠️  Rate limiting: disabled");
    }

    // Create router
    let app = server::create_router(config.clone());

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("🎧 Listening on {}", addr);
    tracing::info!("📚 OpenAPI Spec: http://{}:{}/api-docs/openapi.json", addr.ip(), addr.port());
    tracing::info!("✅ Server ready! Press Ctrl+C to stop");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    // Serve with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("Server failed");
}

