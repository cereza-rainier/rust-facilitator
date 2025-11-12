#!/bin/bash
# Run load testing with k6

set -e

echo "🚀 Starting load test for x402 Rust Facilitator..."
echo ""

# Check if k6 is installed
if ! command -v k6 &> /dev/null; then
    echo "❌ k6 is not installed"
    echo ""
    echo "Install k6:"
    echo "  macOS:   brew install k6"
    echo "  Linux:   sudo gpg -k && sudo gpg --no-default-keyring --keyring /usr/share/keyrings/k6-archive-keyring.gpg --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys C5AD17C747E3415A3642D57D77C6C491D6AC1D69 && echo \"deb [signed-by=/usr/share/keyrings/k6-archive-keyring.gpg] https://dl.k6.io/deb stable main\" | sudo tee /etc/apt/sources.list.d/k6.list && sudo apt-get update && sudo apt-get install k6"
    echo "  Windows: choco install k6"
    echo ""
    exit 1
fi

# Check if server is running
BASE_URL="${BASE_URL:-http://localhost:3000}"
if ! curl -sf "$BASE_URL/health" > /dev/null 2>&1; then
    echo "⚠️  Server is not running at $BASE_URL"
    echo ""
    echo "Starting server in background..."
    
    # Build release if not exists
    if [ ! -f "target/release/x402-facilitator" ]; then
        echo "Building release binary..."
        cargo build --release
    fi
    
    # Start server
    ./target/release/x402-facilitator &
    SERVER_PID=$!
    STARTED_SERVER=true
    
    echo "Waiting for server to start..."
    sleep 3
    
    # Verify server started
    if ! curl -sf "$BASE_URL/health" > /dev/null 2>&1; then
        echo "❌ Failed to start server"
        kill $SERVER_PID 2>/dev/null || true
        exit 1
    fi
    
    echo "✅ Server started (PID: $SERVER_PID)"
    echo ""
fi

# Run load test
echo "Running k6 load test..."
echo "Target: $BASE_URL"
echo ""

k6 run tests/load_test.js

# Stop server if we started it
if [ "$STARTED_SERVER" = true ]; then
    echo ""
    echo "Stopping server..."
    kill $SERVER_PID
    echo "✅ Server stopped"
fi

echo ""
echo "✅ Load test complete!"
echo ""
echo "Results saved to: load-test-results.json"
echo ""
echo "Key Metrics:"
if [ -f "load-test-results.json" ]; then
    echo "  - Check load-test-results.json for detailed metrics"
fi

