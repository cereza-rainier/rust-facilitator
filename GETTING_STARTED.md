# 🚀 Getting Started with x402 Rust Facilitator

**Complete guide to go from zero to running facilitator in under 10 minutes.**

---

## 📋 Table of Contents

1. [Prerequisites](#prerequisites)
2. [Quick Start (5 minutes)](#quick-start-5-minutes)
3. [Detailed Setup Guide](#detailed-setup-guide)
4. [Testing Your Installation](#testing-your-installation)
5. [Deployment Options](#deployment-options)
6. [Troubleshooting](#troubleshooting)
7. [Next Steps](#next-steps)

---

## Prerequisites

### Required

- **Rust 1.75 or later**
  ```bash
  # Install Rust
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  source $HOME/.cargo/env
  
  # Verify installation
  rustc --version
  cargo --version
  ```

- **Git**
  ```bash
  # macOS
  xcode-select --install
  
  # Linux (Ubuntu/Debian)
  sudo apt-get install git
  
  # Verify
  git --version
  ```

### Optional (Recommended)

- **Solana CLI** (for keypair generation)
  ```bash
  # Install
  sh -c "$(curl -sSfL https://release.solana.com/stable/install)"
  
  # Verify
  solana --version
  ```

- **Docker** (for containerized deployment)
  - Install from [docker.com](https://docs.docker.com/get-docker/)

---

## Quick Start (5 minutes)

### Option A: Using Our Setup Script (Easiest)

```bash
# 1. Clone the repository
git clone https://github.com/yourname/x402-facilitator-rust.git
cd x402-facilitator-rust

# 2. Run the setup wizard
cargo run --bin facilitator-cli -- setup

# 3. Start the facilitator
cargo run --release

# ✅ Done! Facilitator running on http://localhost:3000
```

### Option B: Manual Setup (More Control)

```bash
# 1. Clone and navigate
git clone https://github.com/yourname/x402-facilitator-rust.git
cd x402-facilitator-rust

# 2. Copy environment template
cp env.example .env

# 3. Generate a Solana keypair
cargo run --bin facilitator-cli -- keygen

# This will:
# - Generate a new keypair
# - Display your public key (for funding)
# - Save private key to .env automatically

# 4. Fund your wallet (for devnet)
# Visit https://faucet.solana.com
# Paste your public key
# Request 2 SOL (for transaction fees)

# 5. Build and run
cargo build --release
./target/release/x402-facilitator

# ✅ Facilitator running on http://localhost:3000
```

---

## Detailed Setup Guide

### Step 1: Install Dependencies

#### macOS

```bash
# Install Homebrew if not already installed
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install Solana CLI (optional)
brew install solana
```

#### Linux (Ubuntu/Debian)

```bash
# Update package list
sudo apt-get update

# Install build essentials
sudo apt-get install -y build-essential pkg-config libssl-dev

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install Solana CLI (optional)
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
```

#### Windows

```powershell
# Install Rust
# Download and run: https://win.rustup.rs/

# Install Solana CLI
# Download installer: https://github.com/solana-labs/solana/releases
```

---

### Step 2: Clone the Repository

```bash
# Clone
git clone https://github.com/yourname/x402-facilitator-rust.git
cd x402-facilitator-rust

# Verify you're in the right directory
ls -la
# You should see: Cargo.toml, src/, README.md, etc.
```

---

### Step 3: Configuration

#### Option A: Using CLI Setup Wizard (Recommended)

```bash
cargo run --bin facilitator-cli -- setup
```

This interactive wizard will:
1. Generate a Solana keypair for you
2. Create a `.env` file with your configuration
3. Test your RPC connection
4. Provide instructions for funding your wallet

#### Option B: Manual Configuration

```bash
# 1. Copy the environment template
cp env.example .env

# 2. Edit .env with your favorite editor
nano .env  # or vim, code, etc.
```

**Minimum required configuration:**

```env
# Required: Your Solana private key
FEE_PAYER_PRIVATE_KEY=your_base58_key_here

# Required: Solana RPC endpoint
SOLANA_RPC_URL=https://api.devnet.solana.com

# Required: Network
NETWORK=devnet
```

**Everything else has sane defaults!**

---

### Step 4: Generate a Solana Keypair

#### Using Our CLI Tool (Easiest)

```bash
cargo run --bin facilitator-cli -- keygen
```

Output:
```
🔑 Generated new Solana keypair

Public Key:  7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU
Private Key: [automatically saved to .env]

✅ Configuration updated in .env

Next steps:
1. Fund your wallet with devnet SOL: https://faucet.solana.com
2. Paste your public key: 7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU
3. Request 2 SOL
4. Run: cargo run --release
```

#### Using Solana CLI (Alternative)

```bash
# Generate keypair
solana-keygen new --outfile facilitator-keypair.json

# Display public key
solana-keygen pubkey facilitator-keypair.json

# Convert to base58 (for .env file)
# Use our helper:
cargo run --bin facilitator-cli -- keygen --from-file facilitator-keypair.json
```

---

### Step 5: Fund Your Wallet

#### For Devnet (Testing)

1. Visit: https://faucet.solana.com
2. Select "Devnet"
3. Paste your public key
4. Click "Request 2 SOL"
5. Wait for confirmation (usually < 10 seconds)

#### Verify Balance

```bash
# Check your balance
cargo run --bin facilitator-cli -- balance

# Expected output:
# ✅ Balance: 2.0 SOL
```

#### For Mainnet (Production)

You'll need to:
1. Transfer real SOL to your wallet
2. Ensure you have enough SOL for transaction fees
3. **Recommended minimum: 0.5 SOL** for ongoing operations

---

### Step 6: Build and Run

#### Development Mode (Faster compilation, slower runtime)

```bash
cargo run

# Facilitator will start on http://localhost:3000
```

#### Production Mode (Recommended)

```bash
# Build with optimizations (takes 5-10 minutes first time)
cargo build --release

# Run the optimized binary
./target/release/x402-facilitator

# Expected output:
# 🚀 x402 Facilitator starting...
# ✅ Loaded configuration from .env
# ✅ Created shared RPC client for: https://api.devnet.solana.com
# 🔔 Webhooks disabled
# 📊 Metrics enabled at /metrics
# 🚀 Listening on http://0.0.0.0:3000
# 📖 OpenAPI docs at http://0.0.0.0:3000/api-docs/openapi.json
```

---

## Testing Your Installation

### 1. Health Check

```bash
curl http://localhost:3000/health

# Expected response:
# {"status":"ok","network":"devnet","version":"1.0.0"}
```

### 2. Check Supported Networks

```bash
curl http://localhost:3000/supported

# Expected response:
# {
#   "schemes": [
#     {
#       "scheme": "exact",
#       "networks": ["solana-devnet", "solana"]
#     }
#   ]
# }
```

### 3. Test Verification (with dummy data)

```bash
# This will fail (as expected) but confirms the endpoint works
curl -X POST http://localhost:3000/verify \
  -H "Content-Type: application/json" \
  -d '{"transaction": "test"}'

# Expected: 422 Unprocessable Entity (validation error)
# This is correct - it means the endpoint is working!
```

### 4. Check Metrics

```bash
curl http://localhost:3000/metrics

# Expected: Prometheus metrics output
# health_check_total 1
# verify_requests_total 0
# ...
```

### 5. Run Full Test Suite

```bash
# Run all tests
cargo test

# Expected:
# test result: ok. 41 passed; 0 failed; 0 ignored
```

---

## Deployment Options

### Option 1: Local Binary (Simplest)

```bash
# Build
cargo build --release

# Run in background
nohup ./target/release/x402-facilitator > facilitator.log 2>&1 &

# Check status
curl http://localhost:3000/health

# View logs
tail -f facilitator.log
```

### Option 2: Docker (Recommended for Production)

```bash
# Build image
docker build -t x402-facilitator .

# Run container
docker run -d \
  --name x402-facilitator \
  -p 3000:3000 \
  --env-file .env \
  x402-facilitator

# Check logs
docker logs -f x402-facilitator

# Stop
docker stop x402-facilitator
```

### Option 3: Docker Compose (Easy Multi-Service)

```bash
# Create docker-compose.yml (already included)
# Edit environment variables in docker-compose.yml

# Start
docker-compose up -d

# View logs
docker-compose logs -f

# Stop
docker-compose down
```

### Option 4: Kubernetes (Enterprise)

```bash
# See k8s/ directory for manifests

# Create namespace
kubectl create namespace x402

# Create secret from .env
kubectl create secret generic x402-secrets \
  --from-env-file=.env \
  -n x402

# Deploy
kubectl apply -f k8s/ -n x402

# Check status
kubectl get pods -n x402

# View logs
kubectl logs -f deployment/x402-facilitator -n x402
```

### Option 5: Cloud Platforms

#### Fly.io

```bash
# Install flyctl
curl -L https://fly.io/install.sh | sh

# Login
fly auth login

# Launch (interactive setup)
fly launch

# Set secrets
fly secrets set FEE_PAYER_PRIVATE_KEY=your_key_here

# Deploy
fly deploy
```

#### Railway

```bash
# Install Railway CLI
npm i -g @railway/cli

# Login
railway login

# Initialize
railway init

# Deploy
railway up

# Set variables
railway variables set FEE_PAYER_PRIVATE_KEY=your_key_here
```

---

## Troubleshooting

### Problem: "FEE_PAYER_PRIVATE_KEY must be set"

**Cause:** The .env file doesn't exist or the key isn't set.

**Solution:**
```bash
# Check if .env exists
ls -la .env

# If not, copy template
cp env.example .env

# Generate keypair
cargo run --bin facilitator-cli -- keygen
```

---

### Problem: "Connection refused" or RPC errors

**Cause:** Invalid or unreachable RPC URL.

**Solution:**
```bash
# Test your RPC connection
cargo run --bin facilitator-cli -- rpc test

# If it fails, try using the default devnet RPC
echo "SOLANA_RPC_URL=https://api.devnet.solana.com" >> .env
```

---

### Problem: "Insufficient funds"

**Cause:** Your wallet doesn't have enough SOL for transaction fees.

**Solution:**
```bash
# Check balance
cargo run --bin facilitator-cli -- balance

# For devnet, get more from faucet
# Visit: https://faucet.solana.com

# For mainnet, transfer SOL to your wallet
```

---

### Problem: Compilation errors

**Cause:** Rust version too old or missing dependencies.

**Solution:**
```bash
# Update Rust
rustup update

# Install required targets
rustup target add x86_64-unknown-linux-gnu

# Clean and rebuild
cargo clean
cargo build --release
```

---

### Problem: "Address already in use"

**Cause:** Port 3000 is already being used.

**Solution:**
```bash
# Option 1: Change port in .env
echo "PORT=3001" >> .env

# Option 2: Kill process using port 3000
lsof -ti:3000 | xargs kill -9
```

---

### Problem: High memory usage

**Cause:** Cache size is too large.

**Solution:**
```bash
# Reduce cache size in .env
echo "CACHE_SIZE=100" >> .env

# Restart facilitator
```

---

## Next Steps

### 🎯 After Getting It Running

1. **Read the API Documentation**
   ```bash
   # OpenAPI docs at:
   http://localhost:3000/api-docs/openapi.json
   ```

2. **Set Up Monitoring**
   ```bash
   # Prometheus metrics at:
   http://localhost:3000/metrics
   
   # See MONITORING.md for Grafana dashboard setup
   ```

3. **Configure Webhooks** (if needed)
   ```bash
   # Edit .env
   WEBHOOK_URL=https://your-app.com/webhooks
   WEBHOOK_SECRET=your_secret_here
   ```

4. **Run Load Tests**
   ```bash
   # Install k6
   brew install k6  # macOS
   # or download from https://k6.io
   
   # Run load test
   k6 run tests/load_test.js
   ```

5. **Deploy to Production**
   - See [Deployment Options](#deployment-options)
   - Consider using a paid RPC endpoint (Helius, QuickNode, Alchemy)
   - Set up monitoring and alerting
   - Configure backups for your private key

### 📚 Recommended Reading

- [Architecture Overview](./ARCHITECTURE_DIAGRAM.md)
- [API Reference](./README.md#api-endpoints)
- [Benchmarks](./ACTUAL_BENCHMARK_RESULTS.md)
- [Security Best Practices](./README.md#security)
- [Performance Tuning](./PERFORMANCE_OPTIMIZATIONS.md)

### 🤝 Get Help

- **Issues:** [GitHub Issues](https://github.com/yourname/x402-facilitator/issues)
- **Discussions:** [GitHub Discussions](https://github.com/yourname/x402-facilitator/discussions)
- **Discord:** Join the Coinbase Developer Platform Discord

---

## Common Configuration Scenarios

### Local Development

```env
FEE_PAYER_PRIVATE_KEY=your_key_here
SOLANA_RPC_URL=https://api.devnet.solana.com
NETWORK=devnet
LOG_LEVEL=debug
ENABLE_RATE_LIMIT=false  # Easier for testing
```

### Production (Low Traffic)

```env
FEE_PAYER_PRIVATE_KEY=your_mainnet_key
SOLANA_RPC_URL=https://your-project.helius-rpc.com
NETWORK=mainnet
PORT=3000
CACHE_SIZE=1000
RATE_LIMIT_PER_SECOND=50
LOG_LEVEL=info
SENTRY_DSN=your_sentry_dsn
```

### Production (High Traffic)

```env
FEE_PAYER_PRIVATE_KEY=your_mainnet_key
SOLANA_RPC_URL=https://your-premium-rpc.com
NETWORK=mainnet
CACHE_SIZE=10000
CACHE_TTL_SECONDS=15
RATE_LIMIT_PER_SECOND=1000
RATE_LIMIT_BURST_SIZE=2000
LOG_LEVEL=warn
ENABLE_METRICS=true
```

---

## Quick Reference

### Useful Commands

```bash
# Build
cargo build --release

# Run
./target/release/x402-facilitator

# Run with custom port
PORT=3001 ./target/release/x402-facilitator

# Test
cargo test

# Check code
cargo clippy

# Format code
cargo fmt

# Generate keypair
cargo run --bin facilitator-cli -- keygen

# Check balance
cargo run --bin facilitator-cli -- balance

# Test RPC connection
cargo run --bin facilitator-cli -- rpc test

# Run setup wizard
cargo run --bin facilitator-cli -- setup
```

### Default Configuration

| Setting | Default | Description |
|---------|---------|-------------|
| Port | 3000 | HTTP server port |
| Host | 0.0.0.0 | Bind address |
| Cache Size | 1000 | Account cache entries |
| Cache TTL | 30 sec | Cache expiration time |
| Rate Limit | 10/sec | Requests per second |
| Log Level | info | Logging verbosity |

---

**🎉 Congratulations! Your x402 Rust Facilitator is ready!**

For questions, issues, or contributions, visit our [GitHub repository](https://github.com/yourname/x402-facilitator-rust).


