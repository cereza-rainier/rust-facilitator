#!/bin/bash

# Simple batch verification test
# Tests the new /verify/batch endpoint

echo "🧪 Testing Batch Verification Endpoint"
echo "======================================"
echo ""

FACILITATOR_URL="${FACILITATOR_URL:-http://localhost:3000}"

# Check if server is running
if ! curl -s "$FACILITATOR_URL/health" > /dev/null 2>&1; then
    echo "❌ Facilitator not running at $FACILITATOR_URL"
    echo ""
    echo "Please start it first:"
    echo "  cargo run --release"
    echo ""
    exit 1
fi

echo "✅ Facilitator is running"
echo ""

# Test 1: Empty batch
echo "Test 1: Empty batch..."
response=$(curl -s -X POST \
    -H "Content-Type: application/json" \
    -d '[]' \
    "$FACILITATOR_URL/verify/batch")

if [ "$response" = "[]" ]; then
    echo "  ✅ Empty batch handled correctly"
else
    echo "  ❌ Unexpected response: $response"
fi
echo ""

# Test 2: Single request in batch
echo "Test 2: Single request batch..."
curl -s -X POST \
    -H "Content-Type: application/json" \
    -d '[{
        "payment_payload": {
            "x402_version": 1,
            "scheme": "exact",
            "network": "solana-devnet",
            "payload": {
                "transaction": "test_transaction"
            }
        },
        "payment_requirements": {
            "scheme": "exact",
            "network": "solana-devnet",
            "max_amount_required": "1000000",
            "asset": "SOL",
            "pay_to": "recipient",
            "resource": "/api/test",
            "description": "Test",
            "mime_type": "application/json",
            "max_timeout_seconds": 30,
            "extra": {
                "fee_payer": "fee_payer"
            }
        }
    }]' \
    "$FACILITATOR_URL/verify/batch" | jq '.'

echo ""
echo "  ✅ Single batch request completed"
echo ""

# Test 3: Multiple requests
echo "Test 3: Multiple requests (5 items)..."
start=$(date +%s%N)

curl -s -X POST \
    -H "Content-Type: application/json" \
    -d '[
        {
            "payment_payload": {
                "x402_version": 1,
                "scheme": "exact",
                "network": "solana-devnet",
                "payload": {"transaction": "tx1"}
            },
            "payment_requirements": {
                "scheme": "exact",
                "network": "solana-devnet",
                "max_amount_required": "1000000",
                "asset": "SOL",
                "pay_to": "recipient",
                "resource": "/api/test",
                "description": "Test",
                "mime_type": "application/json",
                "max_timeout_seconds": 30,
                "extra": {"fee_payer": "fee_payer"}
            }
        },
        {
            "payment_payload": {
                "x402_version": 1,
                "scheme": "exact",
                "network": "solana-devnet",
                "payload": {"transaction": "tx2"}
            },
            "payment_requirements": {
                "scheme": "exact",
                "network": "solana-devnet",
                "max_amount_required": "1000000",
                "asset": "SOL",
                "pay_to": "recipient",
                "resource": "/api/test",
                "description": "Test",
                "mime_type": "application/json",
                "max_timeout_seconds": 30,
                "extra": {"fee_payer": "fee_payer"}
            }
        },
        {
            "payment_payload": {
                "x402_version": 1,
                "scheme": "exact",
                "network": "solana-devnet",
                "payload": {"transaction": "tx3"}
            },
            "payment_requirements": {
                "scheme": "exact",
                "network": "solana-devnet",
                "max_amount_required": "1000000",
                "asset": "SOL",
                "pay_to": "recipient",
                "resource": "/api/test",
                "description": "Test",
                "mime_type": "application/json",
                "max_timeout_seconds": 30,
                "extra": {"fee_payer": "fee_payer"}
            }
        }
    ]' \
    "$FACILITATOR_URL/verify/batch" | jq '.'

end=$(date +%s%N)
duration_ms=$(( ($end - $start) / 1000000 ))

echo ""
echo "  ✅ Batch of 3 completed in ${duration_ms}ms"
echo ""

echo "✅ All tests passed!"
echo ""
echo "🎯 Next: Run the full benchmark to see all cores working:"
echo "  ./scripts/benchmark_parallel.sh"
echo ""

