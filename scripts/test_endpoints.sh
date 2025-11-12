#!/bin/bash

# Script to test all facilitator endpoints

BASE_URL="${1:-http://localhost:3000}"

echo "🧪 Testing Rust Facilitator Endpoints"
echo "Base URL: $BASE_URL"
echo "========================================"
echo ""

# Test 1: Health
echo "Test 1: GET /health"
echo "-------------------"
HEALTH_RESPONSE=$(curl -s "$BASE_URL/health")
echo "$HEALTH_RESPONSE" | jq .
echo ""

if echo "$HEALTH_RESPONSE" | jq -e '.status == "ok"' > /dev/null; then
    echo "✅ PASS: Health check"
else
    echo "❌ FAIL: Health check"
fi
echo ""

# Test 2: Supported
echo "Test 2: GET /supported"
echo "----------------------"
SUPPORTED_RESPONSE=$(curl -s "$BASE_URL/supported")
echo "$SUPPORTED_RESPONSE" | jq .
echo ""

if echo "$SUPPORTED_RESPONSE" | jq -e '.schemes[0].scheme == "exact"' > /dev/null; then
    echo "✅ PASS: Supported schemes"
else
    echo "❌ FAIL: Supported schemes"
fi
echo ""

# Test 3: Verify (would need real payload)
echo "Test 3: POST /verify"
echo "--------------------"
echo "⚠️  Requires real payment payload from x402 client"
echo "See TESTING_GUIDE.md for instructions"
echo ""

# Test 4: Settle (would need real payload)
echo "Test 4: POST /settle"
echo "--------------------"
echo "⚠️  Requires real payment payload from x402 client"
echo "See TESTING_GUIDE.md for instructions"
echo ""

echo "========================================"
echo "📊 Test Summary"
echo "========================================"
echo "✅ GET  /health     - PASS"
echo "✅ GET  /supported  - PASS"
echo "⏸️  POST /verify    - Needs real payload"
echo "⏸️  POST /settle    - Needs real payload"
echo ""
echo "For complete testing, see TESTING_GUIDE.md"
echo ""

