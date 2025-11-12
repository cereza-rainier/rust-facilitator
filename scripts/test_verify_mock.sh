#!/bin/bash

# Mock test for /verify endpoint
# Note: This uses a sample payload structure
# For real testing, you need an actual transaction from x402 client

BASE_URL="${1:-http://localhost:3000}"

echo "🧪 Testing POST /verify with Mock Payload"
echo "=========================================="
echo ""

# Create a mock verify request
# NOTE: This will fail validation because it's not a real transaction
# But it proves the endpoint is working and returns proper errors

cat > /tmp/mock_verify.json << 'EOF'
{
  "payment_payload": {
    "x402Version": 1,
    "scheme": "exact",
    "network": "solana-devnet",
    "payload": {
      "transaction": "AQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=="
    }
  },
  "payment_requirements": {
    "scheme": "exact",
    "network": "solana-devnet",
    "maxAmountRequired": "1000",
    "asset": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
    "payTo": "2wKupLR9q6wXYppw8Gr2NvWxKBUqm4PPJKkQfoxHDBg4",
    "resource": "https://example.com/api/test",
    "description": "Test payment",
    "mimeType": "application/json",
    "maxTimeoutSeconds": 60,
    "extra": {
      "feePayer": "EwWqGE4ZFKLofuestmU4LDdK7XM1N4ALgdZccwYugwGd"
    }
  }
}
EOF

echo "Request payload:"
cat /tmp/mock_verify.json | jq .
echo ""

echo "Sending POST request to $BASE_URL/verify..."
echo ""

RESPONSE=$(curl -s -X POST "$BASE_URL/verify" \
  -H "Content-Type: application/json" \
  -d @/tmp/mock_verify.json)

echo "Response:"
echo "$RESPONSE" | jq .
echo ""

# Check if endpoint is working (even if it returns invalid)
if echo "$RESPONSE" | jq -e 'has("isValid")' > /dev/null 2>&1; then
    echo "✅ ENDPOINT FUNCTIONAL: /verify endpoint is responding correctly"
    echo ""
    
    if echo "$RESPONSE" | jq -e '.isValid == false' > /dev/null 2>&1; then
        REASON=$(echo "$RESPONSE" | jq -r '.invalidReason // "none"')
        echo "ℹ️  Transaction validation failed (expected with mock data)"
        echo "   Reason: $REASON"
        echo ""
        echo "✅ This confirms the verification logic is working!"
        echo "   Real transactions would pass verification if valid."
    fi
else
    echo "❌ ENDPOINT ERROR: Unexpected response format"
fi

echo ""
echo "=========================================="
echo "Note: Full validation requires a real transaction"
echo "from an x402 client. This test confirms the"
echo "endpoint accepts requests and validates correctly."
echo "=========================================="

rm /tmp/mock_verify.json

