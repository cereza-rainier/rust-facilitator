#!/bin/bash
set -e

echo "🔬 Memory Profiling Benchmark"
echo "=============================="
echo ""

# Check if facilitator is running
PID=$(pgrep -f "x402-facilitator" | head -1)
if [ -z "$PID" ]; then
    echo "❌ Facilitator not running"
    exit 1
fi

echo "✅ Monitoring PID: $PID"
echo ""

# Helper function to get memory
get_memory() {
  ps -p $PID -o rss= 2>/dev/null | awk '{printf "%.1f", $1/1024}' || echo "0"
}

echo "📊 Test 1: Baseline (Idle Memory)"
echo "---------------------------------"
IDLE_MEM=$(get_memory)
echo "Idle Memory: ${IDLE_MEM} MB"
echo ""

echo "📊 Test 2: Light Load (100 requests)"
echo "------------------------------------"
for i in {1..100}; do
  curl -s http://localhost:8080/health > /dev/null 2>&1 &
done
wait
sleep 1
LIGHT_MEM=$(get_memory)
echo "After 100 requests: ${LIGHT_MEM} MB"
echo ""

echo "📊 Test 3: Medium Load (500 requests)"
echo "-------------------------------------"
for i in {1..500}; do
  curl -s http://localhost:8080/health > /dev/null 2>&1 &
  if [ $((i % 50)) -eq 0 ]; then
    wait
  fi
done
wait
sleep 1
MEDIUM_MEM=$(get_memory)
echo "After 500 requests: ${MEDIUM_MEM} MB"
echo ""

echo "📊 Test 4: Heavy Load (1000 requests)"
echo "-------------------------------------"
for i in {1..1000}; do
  curl -s http://localhost:8080/health > /dev/null 2>&1 &
  if [ $((i % 100)) -eq 0 ]; then
    wait
  fi
done
wait
sleep 1
HEAVY_MEM=$(get_memory)
echo "After 1000 requests: ${HEAVY_MEM} MB"
echo ""

echo "📊 Test 5: Stabilization (wait 30s)"
echo "------------------------------------"
echo "Waiting for memory to stabilize..."
sleep 30
STABLE_MEM=$(get_memory)
echo "After stabilization: ${STABLE_MEM} MB"
echo ""

echo "📊 Test 6: Second Load Cycle (1000 requests)"
echo "--------------------------------------------"
for i in {1..1000}; do
  curl -s http://localhost:8080/health > /dev/null 2>&1 &
  if [ $((i % 100)) -eq 0 ]; then
    wait
  fi
done
wait
sleep 1
FINAL_MEM=$(get_memory)
echo "After 2nd cycle: ${FINAL_MEM} MB"
echo ""

echo "📊 Results Summary"
echo "=================="
echo "Idle:                ${IDLE_MEM} MB"
echo "Light Load (100):    ${LIGHT_MEM} MB"
echo "Medium Load (500):   ${MEDIUM_MEM} MB"
echo "Heavy Load (1000):   ${HEAVY_MEM} MB"
echo "After Wait:          ${STABLE_MEM} MB"
echo "Second Cycle (1000): ${FINAL_MEM} MB"
echo ""

# Calculate growth
GROWTH=$(echo "$FINAL_MEM - $IDLE_MEM" | bc)
echo "Total Growth: ${GROWTH} MB"
echo ""

if (( $(echo "$GROWTH < 10" | bc -l) )); then
  echo "✅ Memory stable (growth < 10MB)"
  echo "✅ No memory leak detected"
elif (( $(echo "$GROWTH < 20" | bc -l) )); then
  echo "⚠️  Moderate growth (10-20MB)"
  echo "⚠️  Monitor for leaks in production"
else
  echo "❌ High growth (>20MB)"
  echo "❌ Potential memory leak"
fi

echo ""
echo "✅ Memory profiling complete!"

