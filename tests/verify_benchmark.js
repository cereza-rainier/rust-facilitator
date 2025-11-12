#!/usr/bin/env node

/**
 * Benchmark /verify endpoint - MORE COMPLEX than /supported
 * This endpoint:
 * - Makes RPC calls to Solana
 * - Verifies signatures
 * - Validates transaction structure
 * - Checks account existence
 * 
 * This is where optimizations SHOULD help!
 */

const RUST_PORT = 3000;
const TS_PORT = 3001;
const WARMUP = 20;
const REQUESTS = 200;

// Sample partially-signed transaction for verification
// This is a mock transaction - won't actually work without valid data
const SAMPLE_VERIFY_REQUEST = {
  transaction: "AgAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==",
  payerAccount: "4zvwRjXUKGfvwnParsHAS3HuSVzV5cA4McphgmoCtajS"
};

async function fetchWithRetry(url, options, retries = 3) {
  for (let i = 0; i < retries; i++) {
    try {
      const response = await fetch(url, options);
      return response;
    } catch (error) {
      if (i === retries - 1) throw error;
      await new Promise(resolve => setTimeout(resolve, 100));
    }
  }
}

async function measureLatency(port, endpoint, data, requests) {
  const latencies = [];
  const url = `http://localhost:${port}${endpoint}`;
  
  for (let i = 0; i < requests; i++) {
    const start = performance.now();
    try {
      await fetchWithRetry(url, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(data)
      });
    } catch (error) {
      // Expected to fail with invalid transaction
      // We're measuring latency, not success
    }
    const end = performance.now();
    latencies.push(end - start);
  }
  
  return latencies;
}

function analyzeLatencies(latencies) {
  const sorted = [...latencies].sort((a, b) => a - b);
  return {
    min: sorted[0],
    mean: latencies.reduce((a, b) => a + b, 0) / latencies.length,
    p50: sorted[Math.floor(sorted.length * 0.50)],
    p95: sorted[Math.floor(sorted.length * 0.95)],
    p99: sorted[Math.floor(sorted.length * 0.99)],
    max: sorted[sorted.length - 1]
  };
}

async function main() {
  console.log('\n╔═══════════════════════════════════════════════════════════╗');
  console.log('║    🔍 /verify ENDPOINT BENCHMARK - COMPLEX WORKLOAD 🔍    ║');
  console.log('╚═══════════════════════════════════════════════════════════╝\n');
  
  console.log('📋 Configuration:');
  console.log(`   Endpoint: POST /verify`);
  console.log(`   Warmup requests: ${WARMUP}`);
  console.log(`   Benchmark requests: ${REQUESTS}`);
  console.log(`   Rust port: ${RUST_PORT}`);
  console.log(`   TypeScript port: ${TS_PORT}`);
  console.log('   Note: Requests will fail (invalid tx), measuring latency only\n');
  
  // Warmup and benchmark Rust
  console.log('🔥 Warming up Rust...');
  await measureLatency(RUST_PORT, '/verify', SAMPLE_VERIFY_REQUEST, WARMUP);
  
  console.log('⚡ Benchmarking Rust (optimized)...');
  const rustLatencies = await measureLatency(RUST_PORT, '/verify', SAMPLE_VERIFY_REQUEST, REQUESTS);
  const rustStats = analyzeLatencies(rustLatencies);
  console.log('✅ Rust complete\n');
  
  // Warmup and benchmark TypeScript
  console.log('🔥 Warming up TypeScript...');
  await measureLatency(TS_PORT, '/verify', SAMPLE_VERIFY_REQUEST, WARMUP);
  
  console.log('⚡ Benchmarking TypeScript...');
  const tsLatencies = await measureLatency(TS_PORT, '/verify', SAMPLE_VERIFY_REQUEST, REQUESTS);
  const tsStats = analyzeLatencies(tsLatencies);
  console.log('✅ TypeScript complete\n');
  
  // Display results
  console.log('╔═══════════════════════════════════════════════════════════╗');
  console.log('║      /verify BENCHMARK - RUST vs TYPESCRIPT RESULTS       ║');
  console.log('╚═══════════════════════════════════════════════════════════╝\n');
  
  console.log('┌─────────────────┬──────────┬──────────┬──────────┐');
  console.log('│ Metric          │ Rust     │ TS       │ Speedup  │');
  console.log('├─────────────────┼──────────┼──────────┼──────────┤');
  
  const metrics = [
    ['Min', 'min'],
    ['Mean', 'mean'],
    ['P50 (median)', 'p50'],
    ['P95', 'p95'],
    ['P99', 'p99'],
    ['Max', 'max']
  ];
  
  metrics.forEach(([label, key]) => {
    const rust = rustStats[key];
    const ts = tsStats[key];
    const speedup = (ts / rust).toFixed(2);
    console.log(`│ ${label.padEnd(15)} │ ${rust.toFixed(2).padStart(6)}ms │ ${ts.toFixed(2).padStart(6)}ms │ ${speedup.padStart(6)}x │`);
  });
  
  console.log('└─────────────────┴──────────┴──────────┴──────────┘\n');
  
  const avgSpeedup = (tsStats.mean / rustStats.mean).toFixed(2);
  console.log(`🎯 RESULT: Rust is ${avgSpeedup}x faster on average for /verify!\n`);
  
  console.log('💡 This endpoint includes:');
  console.log('   • RPC calls to Solana');
  console.log('   • Transaction deserialization');
  console.log('   • Signature verification');
  console.log('   • Account existence checks\n');
  
  console.log('🔍 This is where optimizations SHOULD matter:\n');
  console.log('   • Connection pooling (RPC reuse)');
  console.log('   • Account caching (Moka cache)');
  console.log('   • mimalloc (allocation-heavy)');
  console.log('   • LTO (better inlining)\n');
  
  console.log('✅ Benchmark complete!\n');
}

main().catch(console.error);

