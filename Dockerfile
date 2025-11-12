# Multi-stage build for minimal image size
# x402 Rust Facilitator - Production Dockerfile

# Stage 1: Builder
FROM rust:1.82-slim AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build for release with optimizations
RUN cargo build --release && \
    strip target/release/x402-facilitator

# Stage 2: Runtime
FROM debian:bookworm-slim

# Install runtime dependencies only
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user for security
RUN useradd -m -u 1000 facilitator && \
    mkdir -p /app && \
    chown facilitator:facilitator /app

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/x402-facilitator /usr/local/bin/x402-facilitator

# Switch to non-root user
USER facilitator

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Run the binary
CMD ["x402-facilitator"]

