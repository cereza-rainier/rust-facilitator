#!/usr/bin/env node

/**
 * Memory Leak Test - Does memory stabilize or keep growing?
 * 
 * This test monitors memory over time to detect leaks.
 */

const PORT = 3000;
const TEST_DURATION = 60; // seconds
const REQUEST_RATE = 100; // req/s

async function makeRequest() {
  try {
    await fetch(`http://localhost:${PORT}/health`);
  } catch (e) {
    // Ignore errors
  }
}

async function getMemoryUsage(pid) {
  const { execSync } = require('child_process');
  try {
    const output = execSync(`ps -p ${pid} -o rss=`).toString().trim();
    return parseInt(output) / 1024; // Convert KB to MB
  } catch (e) {
    return null;
  }
}

async function findRustPid() {
  const { execSync } = require('child_process');
  try {
    const output = execSync('pgrep -f x402-facilitator').toString().trim();
    return parseInt(output.split('\n')[0]);
  } catch (e) {
    return null;
  }
}

async function main() {
  console.log('╔═══════════════════════════════════════════════════════════╗');
  console.log('║           🔍 MEMORY LEAK DETECTION TEST 🔍                ║');
  console.log('╚═══════════════════════════════════════════════════════════╝\n');
  
  const pid = await findRustPid();
  if (!pid) {
    console.error('❌ Rust server not running!');
    console.error('   Start with: ./target/release/x402-facilitator');
    process.exit(1);
  }
  
  console.log(`📊 Found Rust server (PID: ${pid})`);
  console.log(`⏱️  Test duration: ${TEST_DURATION} seconds`);
  console.log(`📈 Request rate: ${REQUEST_RATE} req/s`);
  console.log(`🔬 Monitoring memory every 5 seconds...\n`);
  
  const measurements = [];
  const startTime = Date.now();
  
  // Start making requests
  const requestInterval = setInterval(async () => {
    for (let i = 0; i < REQUEST_RATE / 10; i++) {
      makeRequest();
    }
  }, 100);
  
  // Measure memory every 5 seconds
  const measureInterval = setInterval(async () => {
    const memory = await getMemoryUsage(pid);
    const elapsed = Math.floor((Date.now() - startTime) / 1000);
    
    if (memory) {
      measurements.push({ time: elapsed, memory });
      console.log(`   ${elapsed}s: ${memory.toFixed(1)} MB`);
    }
  }, 5000);
  
  // Initial measurement
  const initialMemory = await getMemoryUsage(pid);
  console.log(`   0s: ${initialMemory.toFixed(1)} MB (baseline)\n`);
  measurements.push({ time: 0, memory: initialMemory });
  
  // Wait for test duration
  await new Promise(resolve => setTimeout(resolve, TEST_DURATION * 1000));
  
  // Stop making requests
  clearInterval(requestInterval);
  clearInterval(measureInterval);
  
  // Wait 10 seconds for memory to stabilize
  console.log('\n⏸️  Waiting 10 seconds for memory to stabilize...');
  await new Promise(resolve => setTimeout(resolve, 10000));
  
  const finalMemory = await getMemoryUsage(pid);
  console.log(`   Final: ${finalMemory.toFixed(1)} MB\n`);
  measurements.push({ time: TEST_DURATION + 10, memory: finalMemory });
  
  // Analysis
  console.log('═══════════════════════════════════════════════════════════\n');
  console.log('📊 ANALYSIS:\n');
  
  const growth = finalMemory - initialMemory;
  const growthPercent = ((growth / initialMemory) * 100).toFixed(1);
  
  console.log(`   Initial:  ${initialMemory.toFixed(1)} MB`);
  console.log(`   Peak:     ${Math.max(...measurements.map(m => m.memory)).toFixed(1)} MB`);
  console.log(`   Final:    ${finalMemory.toFixed(1)} MB`);
  console.log(`   Growth:   ${growth > 0 ? '+' : ''}${growth.toFixed(1)} MB (${growthPercent}%)\n`);
  
  // Check if memory stabilized
  const lastFive = measurements.slice(-5);
  const avgLast = lastFive.reduce((a, b) => a + b.memory, 0) / lastFive.length;
  const variance = lastFive.reduce((sum, m) => sum + Math.pow(m.memory - avgLast, 2), 0) / lastFive.length;
  const stddev = Math.sqrt(variance);
  
  console.log(`   Stability (last 25s):`);
  console.log(`     Average: ${avgLast.toFixed(1)} MB`);
  console.log(`     StdDev:  ${stddev.toFixed(2)} MB\n`);
  
  // Verdict
  if (stddev < 0.5) {
    console.log('✅ VERDICT: Memory is STABLE (likely caching, not leaking)\n');
    console.log('   Why memory grew:');
    console.log('   • AccountCache filled up (~1000 entries)');
    console.log('   • Connection pooling established');
    console.log('   • Tokio runtime warmed up');
    console.log('   • Prometheus metrics accumulated\n');
    console.log('   This is NORMAL and EXPECTED behavior! 👍\n');
  } else if (stddev < 2) {
    console.log('⚠️  VERDICT: Memory is FLUCTUATING but stable\n');
    console.log('   Possible causes:');
    console.log('   • Cache eviction and refill');
    console.log('   • HTTP connections opening/closing');
    console.log('   • Normal runtime behavior\n');
    console.log('   Likely OK, but monitor in production 📊\n');
  } else {
    console.log('❌ VERDICT: Potential MEMORY LEAK!\n');
    console.log('   Memory keeps growing without stabilizing.');
    console.log('   Investigate with: cargo flamegraph or heaptrack\n');
  }
  
  // Growth rate
  const growthRate = growth / (TEST_DURATION / 60); // MB per minute
  console.log(`   Growth rate: ${growthRate.toFixed(2)} MB/minute\n`);
  
  if (growthRate < 0.5) {
    console.log('   ✅ Growth rate is acceptable (<0.5 MB/min)\n');
  } else if (growthRate < 2) {
    console.log('   ⚠️  Growth rate is moderate (0.5-2 MB/min)\n');
  } else {
    console.log('   ❌ Growth rate is concerning (>2 MB/min)\n');
  }
  
  console.log('═══════════════════════════════════════════════════════════\n');
  console.log('✅ Test complete!\n');
}

main().catch(console.error);

