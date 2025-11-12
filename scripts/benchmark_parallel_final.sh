#!/bin/bash
set -e

echo "🔬 Parallel Processing Benchmark"
echo "=================================="
echo ""

# Check if facilitator is running
if ! curl -s http://localhost:8080/health > /dev/null 2>&1; then
    echo "❌ Facilitator not running on port 8080"
    exit 1
fi

echo "✅ Facilitator is running"
echo ""

# Create simple test payload
cat > /tmp/test_verify.json << 'EOF'
{
  "paymentPayload": {
    "x402Version": 1,
    "scheme": "exact",
    "network": "solana-devnet",
    "payload": {
      "transaction": "AQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAEDArczbMia1tLmq7zz4DinMNN0pJ1JtLdqIJPUw3YrGCzYAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAgIAAQwCAAAAKgAAAAAAAAA="
    },
    "timestamp": 1699000000
  },
  "paymentRequirements": {
    "scheme": "exact",
    "network": "solana-devnet",
    "maxAmountRequired": "1000000",
    "asset": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
    "payTo": "8VzycpqZpqYXMqKSZqYXMqKSZqYXMqKS",
    "resource": "/api/resource",
    "description": "Test Payment",
    "mimeType": "application/json",
    "maxTimeoutSeconds": 30,
    "extra": {
      "feePayer": "FeePayerPublicKeyHere123456789"
    }
  }
}
EOF

echo "📊 Test 1: Sequential Processing (50 requests)"
echo "----------------------------------------------"
START=$(date +%s%N)
for i in {1..50}; do
  curl -s -X POST http://localhost:8080/verify \
    -H "Content-Type: application/json" \
    -d @/tmp/test_verify.json > /dev/null 2>&1
done
END=$(date +%s%N)
SEQUENTIAL_TIME=$(( ($END - $START) / 1000000 ))
echo "Time: ${SEQUENTIAL_TIME}ms"
echo ""

echo "📊 Test 2: Parallel Batch Processing (50 requests)"
echo "--------------------------------------------------"
# Create batch of 50
cat > /tmp/test_batch.json << EOF
[
$(for i in {1..50}; do 
  cat /tmp/test_verify.json
  if [ $i -lt 50 ]; then echo ","; fi
done)
]
EOF

START=$(date +%s%N)
curl -s -X POST http://localhost:8080/verify/batch \
  -H "Content-Type: application/json" \
  -d @/tmp/test_batch.json > /dev/null 2>&1
END=$(date +%s%N)
PARALLEL_TIME=$(( ($END - $START) / 1000000 ))
echo "Time: ${PARALLEL_TIME}ms"
echo ""

echo "📊 Results"
echo "=========="
echo "Sequential (50 items): ${SEQUENTIAL_TIME}ms"
echo "Parallel (50 items):   ${PARALLEL_TIME}ms"
if [ $PARALLEL_TIME -gt 0 ]; then
  SPEEDUP=$(echo "scale=2; $SEQUENTIAL_TIME / $PARALLEL_TIME" | bc)
  echo "Speedup:               ${SPEEDUP}x"
else
  echo "Speedup:               N/A (parallel too fast to measure)"
fi
echo ""

# Cleanup
rm -f /tmp/test_verify.json /tmp/test_batch.json

echo "✅ Benchmark complete!"

