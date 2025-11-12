#!/bin/bash
# Comparative Benchmark Runner
# Starts both Rust and TypeScript facilitators and runs benchmarks

set -e

echo "🏎️  Rust vs TypeScript Facilitator Benchmark Setup"
echo "════════════════════════════════════════════════════"
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
RUST_PORT=3000
TS_PORT=3001
BENCHMARK_RESULTS_DIR="benchmark_results"

# Check if Node.js is installed
if ! command -v node &> /dev/null; then
    echo -e "${RED}❌ Node.js not found. Please install Node.js 20+${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Node.js found: $(node --version)${NC}"

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}❌ Cargo not found. Please install Rust${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Rust found: $(rustc --version)${NC}"

# Build Rust facilitator
echo ""
echo -e "${CYAN}📦 Building Rust facilitator...${NC}"
cargo build --release
if [ $? -ne 0 ]; then
    echo -e "${RED}❌ Rust build failed${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Rust build complete${NC}"

# Check if TypeScript facilitator exists
TS_FACILITATOR_PATH="../x402/examples/typescript/facilitator"
if [ ! -d "$TS_FACILITATOR_PATH" ]; then
    echo -e "${RED}❌ TypeScript facilitator not found at $TS_FACILITATOR_PATH${NC}"
    echo -e "${YELLOW}   Please ensure the x402 repository is cloned in the parent directory${NC}"
    exit 1
fi

# Setup TypeScript facilitator
echo ""
echo -e "${CYAN}📦 Setting up TypeScript facilitator...${NC}"
cd "$TS_FACILITATOR_PATH"

# Install pnpm if not present
if ! command -v pnpm &> /dev/null; then
    echo -e "${YELLOW}⚠️  pnpm not found, installing...${NC}"
    npm install -g pnpm
fi

# Install dependencies from root
cd ../..
echo -e "${CYAN}   Installing dependencies from root...${NC}"
pnpm install

# Build packages
echo -e "${CYAN}   Building packages...${NC}"
pnpm build

cd "examples/typescript/facilitator"

# Check for .env file
if [ ! -f ".env" ]; then
    echo -e "${YELLOW}⚠️  No .env file found for TypeScript facilitator${NC}"
    echo -e "${YELLOW}   Creating a minimal .env file (you may need to configure it)${NC}"
    cat > .env << EOF
# EVM Private Key (optional for testing)
EVM_PRIVATE_KEY=0x0000000000000000000000000000000000000000000000000000000000000001

# SVM Private Key (base58 encoded)
SVM_PRIVATE_KEY=YourBase58PrivateKeyHere

# Optional: Custom Solana RPC URL
# SVM_RPC_URL=https://api.devnet.solana.com

# Port (different from Rust to avoid conflict)
PORT=${TS_PORT}
EOF
fi

echo -e "${GREEN}✅ TypeScript setup complete${NC}"

# Go back to rust-facilitator directory
cd - > /dev/null
cd ../../../rust-facilitator

# Create results directory
mkdir -p "$BENCHMARK_RESULTS_DIR"

# Start Rust facilitator in background
echo ""
echo -e "${CYAN}🚀 Starting Rust facilitator on port $RUST_PORT...${NC}"
./target/release/x402-facilitator > "$BENCHMARK_RESULTS_DIR/rust.log" 2>&1 &
RUST_PID=$!
echo -e "${GREEN}✅ Rust facilitator started (PID: $RUST_PID)${NC}"

# Wait for Rust to be ready
echo -e "${CYAN}   Waiting for Rust server to be ready...${NC}"
sleep 3
if ! curl -s http://localhost:$RUST_PORT/health > /dev/null; then
    echo -e "${RED}❌ Rust server failed to start${NC}"
    kill $RUST_PID 2>/dev/null || true
    exit 1
fi
echo -e "${GREEN}✅ Rust server ready${NC}"

# Start TypeScript facilitator in background
echo ""
echo -e "${CYAN}🚀 Starting TypeScript facilitator on port $TS_PORT...${NC}"
cd "$TS_FACILITATOR_PATH"
PORT=$TS_PORT pnpm dev > "../../../rust-facilitator/$BENCHMARK_RESULTS_DIR/typescript.log" 2>&1 &
TS_PID=$!
echo -e "${GREEN}✅ TypeScript facilitator started (PID: $TS_PID)${NC}"

# Go back to rust-facilitator
cd - > /dev/null
cd ../../../rust-facilitator

# Wait for TypeScript to be ready
echo -e "${CYAN}   Waiting for TypeScript server to be ready...${NC}"
for i in {1..30}; do
    if curl -s http://localhost:$TS_PORT/health > /dev/null 2>&1; then
        echo -e "${GREEN}✅ TypeScript server ready${NC}"
        break
    fi
    if [ $i -eq 30 ]; then
        echo -e "${RED}❌ TypeScript server failed to start${NC}"
        echo -e "${YELLOW}   Check logs at $BENCHMARK_RESULTS_DIR/typescript.log${NC}"
        kill $RUST_PID $TS_PID 2>/dev/null || true
        exit 1
    fi
    sleep 2
done

# Run benchmarks
echo ""
echo -e "${CYAN}🏃 Running benchmarks...${NC}"
echo ""
node tests/comparative_benchmark.js | tee "$BENCHMARK_RESULTS_DIR/benchmark_$(date +%Y%m%d_%H%M%S).txt"

# Cleanup function
cleanup() {
    echo ""
    echo -e "${CYAN}🧹 Cleaning up...${NC}"
    kill $RUST_PID 2>/dev/null || true
    kill $TS_PID 2>/dev/null || true
    
    # Kill any remaining processes on these ports
    lsof -ti:$RUST_PORT | xargs kill -9 2>/dev/null || true
    lsof -ti:$TS_PORT | xargs kill -9 2>/dev/null || true
    
    echo -e "${GREEN}✅ Cleanup complete${NC}"
    
    echo ""
    echo -e "${CYAN}📝 Logs saved to:${NC}"
    echo -e "   Rust:       $BENCHMARK_RESULTS_DIR/rust.log"
    echo -e "   TypeScript: $BENCHMARK_RESULTS_DIR/typescript.log"
    echo -e "   Benchmark:  $BENCHMARK_RESULTS_DIR/benchmark_*.txt"
}

# Register cleanup on script exit
trap cleanup EXIT

# Wait for user input to keep servers running for manual testing
echo ""
echo -e "${YELLOW}🎯 Servers are running:${NC}"
echo -e "   Rust:       http://localhost:$RUST_PORT"
echo -e "   TypeScript: http://localhost:$TS_PORT"
echo ""
echo -e "${YELLOW}Press Ctrl+C to stop servers and exit${NC}"

# Keep script running
wait

