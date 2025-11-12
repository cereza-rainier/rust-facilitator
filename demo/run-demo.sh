#!/bin/bash

# Complete Demo Runner
# This script runs the full demo: facilitator + API server + client

set -e

echo "╔════════════════════════════════════════════════╗"
echo "║   x402 Rust Facilitator - Complete Demo        ║"
echo "╚════════════════════════════════════════════════╝"
echo ""

# Check if facilitator is running
echo "🔍 Checking if facilitator is running..."
if ! curl -s http://localhost:3000/health > /dev/null 2>&1; then
    echo "❌ Facilitator is not running"
    echo ""
    echo "Please start the facilitator first:"
    echo "   cd .."
    echo "   ./deploy.sh"
    echo ""
    exit 1
fi
echo "✅ Facilitator is running"
echo ""

# Check dependencies
echo "🔍 Checking dependencies..."
if [ ! -d "node_modules" ]; then
    echo "📦 Installing dependencies..."
    npm install
    echo ""
fi
echo "✅ Dependencies ready"
echo ""

# Start API server in background
echo "🚀 Starting API server..."
node server.js > /tmp/demo-api.log 2>&1 &
API_PID=$!
echo "✅ API server started (PID: $API_PID)"
echo ""

# Wait for API to be ready
echo "⏳ Waiting for API server to be ready..."
for i in {1..10}; do
    if curl -s http://localhost:4000/health > /dev/null 2>&1; then
        echo "✅ API server is ready"
        break
    fi
    sleep 1
done
echo ""

# Run the client demo
echo "🎬 Running client demo..."
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
node client.js
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Cleanup
echo ""
echo "🧹 Stopping API server..."
kill $API_PID 2>/dev/null || true
echo "✅ Demo complete!"
echo ""
echo "📊 Next steps:"
echo "   - View metrics: curl http://localhost:3000/metrics"
echo "   - View logs: docker-compose logs facilitator"
echo "   - Try again: npm run demo"
echo ""

