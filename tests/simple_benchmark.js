#!/usr/bin/env node
/**
 * Simple Side-by-Side Benchmark
 * Tests /supported endpoint on both Rust and TypeScript implementations
 */

const http = require('http');
const { performance } = require('perf_hooks');

const RUST_PORT = 3000;
const TS_PORT = 3001;
const WARMUP_REQUESTS = 50;
const BENCHMARK_REQUESTS = 500;

// Colors
const colors = {
  reset: '\x1b[0m',
  bright: '\x1b[1m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  cyan: '\x1b[36m',
};

function log(message, color = 'reset') {
  console.log(`${colors[color]}${message}${colors.reset}`);
}

/**
 * Make HTTP request and measure timing
 */
function makeRequest(port, endpoint) {
  return new Promise((resolve, reject) => {
    const start = performance.now();
    
    const req = http.request({
      hostname: 'localhost',
      port,
      path: endpoint,
      method: 'GET',
    }, (res) => {
      let data = '';
      res.on('data', (chunk) => data += chunk);
      res.on('end', () => {
        const duration = performance.now() - start;
        resolve({ status: res.statusCode, duration });
      });
    });

    req.on('error', reject);
    req.end();
  });
}

/**
 * Run benchmark
 */
async function runBenchmark(name, port, endpoint) {
  const results = [];

  // Warmup
  for (let i = 0; i < WARMUP_REQUESTS; i++) {
    try {
      await makeRequest(port, endpoint);
    } catch (e) {
      // Ignore warmup errors
    }
  }

  // Benchmark
  for (let i = 0; i < BENCHMARK_REQUESTS; i++) {
    try {
      const result = await makeRequest(port, endpoint);
      results.push(result.duration);
    } catch (e) {
      console.error(`Error on ${name}:`, e.message);
    }
  }

  if (results.length === 0) {
    return null;
  }

  // Calculate statistics
  results.sort((a, b) => a - b);
  const min = results[0];
  const max = results[results.length - 1];
  const mean = results.reduce((a, b) => a + b, 0) / results.length;
  const p50 = results[Math.floor(results.length * 0.5)];
  const p95 = results[Math.floor(results.length * 0.95)];
  const p99 = results[Math.floor(results.length * 0.99)];

  return { name, count: results.length, min, max, mean, p50, p95, p99 };
}

/**
 * Print comparison table
 */
function printComparison(rust, ts) {
  if (!rust || !ts) {
    log('⚠️  Missing results', 'yellow');
    return;
  }

  const speedup = (ts.mean / rust.mean).toFixed(2);
  const speedupColor = speedup > 1 ? 'green' : 'yellow';

  console.log('\n╔═══════════════════════════════════════════════════════════╗');
  console.log('║         RUST vs TYPESCRIPT - ACTUAL BENCHMARK RESULTS      ║');
  console.log('╚═══════════════════════════════════════════════════════════╝\n');

  console.log('┌─────────────────┬──────────┬──────────┬──────────┐');
  console.log('│ Metric          │ Rust     │ TS       │ Speedup  │');
  console.log('├─────────────────┼──────────┼──────────┼──────────┤');
  console.log(`│ Min             │ ${rust.min.toFixed(2).padStart(6)}ms │ ${ts.min.toFixed(2).padStart(6)}ms │ ${(ts.min / rust.min).toFixed(2).padStart(6)}x │`);
  console.log(`│ Mean            │ ${rust.mean.toFixed(2).padStart(6)}ms │ ${ts.mean.toFixed(2).padStart(6)}ms │ ${speedup.padStart(6)}x │`);
  console.log(`│ P50 (median)    │ ${rust.p50.toFixed(2).padStart(6)}ms │ ${ts.p50.toFixed(2).padStart(6)}ms │ ${(ts.p50 / rust.p50).toFixed(2).padStart(6)}x │`);
  console.log(`│ P95             │ ${rust.p95.toFixed(2).padStart(6)}ms │ ${ts.p95.toFixed(2).padStart(6)}ms │ ${(ts.p95 / rust.p95).toFixed(2).padStart(6)}x │`);
  console.log(`│ P99             │ ${rust.p99.toFixed(2).padStart(6)}ms │ ${ts.p99.toFixed(2).padStart(6)}ms │ ${(ts.p99 / rust.p99).toFixed(2).padStart(6)}x │`);
  console.log(`│ Max             │ ${rust.max.toFixed(2).padStart(6)}ms │ ${ts.max.toFixed(2).padStart(6)}ms │ ${(ts.max / rust.max).toFixed(2).padStart(6)}x │`);
  console.log('└─────────────────┴──────────┴──────────┴──────────┘\n');

  log(`\n🎯 RESULT: Rust is ${speedup}x faster on average!`, speedupColor);
  
  // Memory comparison note
  log('\n📊 Memory Usage (measure separately with ps/htop):', 'cyan');
  log('   Run: ps aux | grep -E "x402-facilitator|tsx"');
  log('   Rust typically: ~9-15 MB');
  log('   Node.js typically: ~50-150 MB\n');
}

/**
 * Main
 */
async function main() {
  log('\n╔═══════════════════════════════════════════════════════════╗', 'bright');
  log('║    🏁 RUST vs TYPESCRIPT - SIDE-BY-SIDE BENCHMARK 🏁     ║', 'bright');
  log('╚═══════════════════════════════════════════════════════════╝\n', 'bright');

  log('📋 Configuration:', 'cyan');
  log(`   Endpoint: GET /supported`);
  log(`   Warmup requests: ${WARMUP_REQUESTS}`);
  log(`   Benchmark requests: ${BENCHMARK_REQUESTS}`);
  log(`   Rust port: ${RUST_PORT}`);
  log(`   TypeScript port: ${TS_PORT}\n`);

  log('🔥 Warming up and benchmarking Rust...', 'yellow');
  const rustResults = await runBenchmark('Rust', RUST_PORT, '/supported');
  log('✅ Rust complete\n', 'green');

  log('🔥 Warming up and benchmarking TypeScript...', 'yellow');
  const tsResults = await runBenchmark('TypeScript', TS_PORT, '/supported');
  log('✅ TypeScript complete\n', 'green');

  printComparison(rustResults, tsResults);

  log('✅ Benchmark complete!', 'green');
  log('\nSave these results! This is real, measured data.\n');
}

main().catch(console.error);


