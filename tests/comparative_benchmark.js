#!/usr/bin/env node
/**
 * Comparative Benchmark: Rust vs TypeScript x402 Facilitator
 * 
 * Tests both implementations side-by-side with:
 * - Latency measurements
 * - Memory usage
 * - Throughput
 * - Concurrent load
 */

import http from 'http';
import { performance } from 'perf_hooks';
import { spawn } from 'child_process';
import fs from 'fs';
import path from 'path';

const RUST_PORT = 3000;
const TS_PORT = 3001;
const WARMUP_REQUESTS = 50;
const BENCHMARK_REQUESTS = 500;
const CONCURRENT_REQUESTS = [1, 10, 50, 100, 200];

// Colors for output
const colors = {
  reset: '\x1b[0m',
  bright: '\x1b[1m',
  red: '\x1b[31m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  blue: '\x1b[34m',
  cyan: '\x1b[36m',
};

function log(message, color = 'reset') {
  console.log(`${colors[color]}${message}${colors.reset}`);
}

// Mock verify request payload
const VERIFY_PAYLOAD = {
  paymentPayload: {
    x402Version: 1,
    scheme: "exact",
    network: "solana-devnet",
    payload: {
      transaction: "AQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAEDArczbMia1tLmq7zz4DinMNN0pJ1JtLdqIJPUw3YrGCzYAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAgIAAQwCAAAAKgAAAAAAAAA="
    }
  },
  paymentRequirements: {
    scheme: "exact",
    network: "solana-devnet",
    maxAmountRequired: "1000000",
    asset: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
    payTo: "8VzycpqZpqYXMqKSZqYXMqKSZqYXMqKS",
    resource: "/api/resource",
    description: "Test payment",
    mimeType: "application/json",
    maxTimeoutSeconds: 30,
    extra: {
      feePayer: "FeePayerPublicKeyHere"
    }
  }
};

/**
 * Make HTTP request and measure timing
 */
function makeRequest(port, endpoint, method = 'GET', body = null) {
  return new Promise((resolve, reject) => {
    const start = performance.now();
    
    const options = {
      hostname: 'localhost',
      port,
      path: endpoint,
      method,
      headers: {
        'Content-Type': 'application/json',
      },
    };

    if (body) {
      options.headers['Content-Length'] = Buffer.byteLength(JSON.stringify(body));
    }

    const req = http.request(options, (res) => {
      let data = '';
      
      res.on('data', (chunk) => {
        data += chunk;
      });

      res.on('end', () => {
        const end = performance.now();
        const duration = end - start;
        
        try {
          const json = JSON.parse(data);
          resolve({
            status: res.statusCode,
            duration,
            data: json,
          });
        } catch (e) {
          resolve({
            status: res.statusCode,
            duration,
            data: data,
          });
        }
      });
    });

    req.on('error', (e) => {
      reject(e);
    });

    if (body) {
      req.write(JSON.stringify(body));
    }

    req.end();
  });
}

/**
 * Check if server is running
 */
async function waitForServer(port, maxAttempts = 30) {
  for (let i = 0; i < maxAttempts; i++) {
    try {
      await makeRequest(port, '/health');
      return true;
    } catch (e) {
      await new Promise(resolve => setTimeout(resolve, 1000));
    }
  }
  return false;
}

/**
 * Get process memory usage
 */
function getMemoryUsage(pid) {
  try {
    const stats = fs.readFileSync(`/proc/${pid}/status`, 'utf8');
    const vmRss = stats.match(/VmRSS:\s+(\d+)\s+kB/);
    if (vmRss) {
      return parseInt(vmRss[1]) / 1024; // Convert to MB
    }
  } catch (e) {
    // macOS fallback using ps
    try {
      const { execSync } = require('child_process');
      const output = execSync(`ps -o rss= -p ${pid}`).toString().trim();
      return parseInt(output) / 1024; // Convert to MB
    } catch (e2) {
      return null;
    }
  }
  return null;
}

/**
 * Run benchmark for a specific endpoint
 */
async function benchmarkEndpoint(name, port, endpoint, method = 'GET', body = null) {
  const results = [];

  // Warmup
  for (let i = 0; i < WARMUP_REQUESTS; i++) {
    try {
      await makeRequest(port, endpoint, method, body);
    } catch (e) {
      // Ignore warmup errors
    }
  }

  // Benchmark
  for (let i = 0; i < BENCHMARK_REQUESTS; i++) {
    try {
      const result = await makeRequest(port, endpoint, method, body);
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

  return {
    name,
    count: results.length,
    min,
    max,
    mean,
    p50,
    p95,
    p99,
  };
}

/**
 * Run concurrent load test
 */
async function benchmarkConcurrent(name, port, endpoint, concurrency, method = 'GET', body = null) {
  const promises = [];
  const start = performance.now();

  for (let i = 0; i < concurrency; i++) {
    promises.push(makeRequest(port, endpoint, method, body));
  }

  const results = await Promise.allSettled(promises);
  const end = performance.now();
  const totalDuration = end - start;

  const successful = results.filter(r => r.status === 'fulfilled').length;
  const failed = results.filter(r => r.status === 'rejected').length;
  const throughput = (successful / (totalDuration / 1000)).toFixed(2);

  return {
    name,
    concurrency,
    successful,
    failed,
    totalDuration: totalDuration.toFixed(2),
    throughput,
  };
}

/**
 * Format benchmark results table
 */
function printBenchmarkTable(rustResults, tsResults) {
  if (!rustResults || !tsResults) {
    log('⚠️  Missing results', 'yellow');
    return;
  }

  const speedup = (tsResults.mean / rustResults.mean).toFixed(2);
  const speedupColor = speedup > 1 ? 'green' : (speedup < 1 ? 'red' : 'yellow');

  console.log('\n┌─────────────────┬──────────┬──────────┬──────────┐');
  console.log('│ Metric          │ Rust     │ TS       │ Speedup  │');
  console.log('├─────────────────┼──────────┼──────────┼──────────┤');
  console.log(`│ Min             │ ${rustResults.min.toFixed(2).padStart(6)}ms │ ${tsResults.min.toFixed(2).padStart(6)}ms │ ${(tsResults.min / rustResults.min).toFixed(2)}x     │`);
  console.log(`│ Mean            │ ${rustResults.mean.toFixed(2).padStart(6)}ms │ ${tsResults.mean.toFixed(2).padStart(6)}ms │ ${speedup}x     │`);
  console.log(`│ P50 (median)    │ ${rustResults.p50.toFixed(2).padStart(6)}ms │ ${tsResults.p50.toFixed(2).padStart(6)}ms │ ${(tsResults.p50 / rustResults.p50).toFixed(2)}x     │`);
  console.log(`│ P95             │ ${rustResults.p95.toFixed(2).padStart(6)}ms │ ${tsResults.p95.toFixed(2).padStart(6)}ms │ ${(tsResults.p95 / rustResults.p95).toFixed(2)}x     │`);
  console.log(`│ P99             │ ${rustResults.p99.toFixed(2).padStart(6)}ms │ ${tsResults.p99.toFixed(2).padStart(6)}ms │ ${(tsResults.p99 / rustResults.p99).toFixed(2)}x     │`);
  console.log(`│ Max             │ ${rustResults.max.toFixed(2).padStart(6)}ms │ ${tsResults.max.toFixed(2).padStart(6)}ms │ ${(tsResults.max / rustResults.max).toFixed(2)}x     │`);
  console.log('└─────────────────┴──────────┴──────────┴──────────┘');

  if (speedup > 1) {
    log(`\n✅ Rust is ${speedup}x faster on average!`, 'green');
  } else if (speedup < 1) {
    log(`\n⚠️  TypeScript is ${(1/speedup).toFixed(2)}x faster on average!`, 'yellow');
  } else {
    log(`\n➡️  Both implementations have similar performance`, 'cyan');
  }
}

/**
 * Main benchmark function
 */
async function main() {
  log('\n╔═══════════════════════════════════════════════════════════════╗', 'bright');
  log('║                                                               ║', 'bright');
  log('║  🏎️  RUST vs TYPESCRIPT x402 FACILITATOR BENCHMARK 🏎️       ║', 'bright');
  log('║                                                               ║', 'bright');
  log('╚═══════════════════════════════════════════════════════════════╝\n', 'bright');

  log('📋 Configuration:', 'cyan');
  log(`   Warmup requests:      ${WARMUP_REQUESTS}`);
  log(`   Benchmark requests:   ${BENCHMARK_REQUESTS}`);
  log(`   Concurrent loads:     ${CONCURRENT_REQUESTS.join(', ')}`);

  // Check if servers are running
  log('\n🔍 Checking servers...', 'cyan');
  
  const rustRunning = await waitForServer(RUST_PORT, 5);
  const tsRunning = await waitForServer(TS_PORT, 5);

  if (!rustRunning) {
    log('❌ Rust server not running on port 3000', 'red');
    log('   Start it with: cargo run --release', 'yellow');
    process.exit(1);
  }
  log('✅ Rust server running on port 3000', 'green');

  if (!tsRunning) {
    log('❌ TypeScript server not running on port 3001', 'red');
    log('   Start it with: cd ../x402/examples/typescript/facilitator && PORT=3001 pnpm dev', 'yellow');
    process.exit(1);
  }
  log('✅ TypeScript server running on port 3001', 'green');

  // Get PIDs and initial memory (simplified - you may need to find PIDs differently)
  log('\n📊 Initial Memory Usage:', 'cyan');
  log('   (Memory usage requires process inspection tools)');

  // Benchmark /health endpoint
  log('\n\n═══ Benchmarking /health endpoint ═══', 'bright');
  log(`Running ${BENCHMARK_REQUESTS} requests (after ${WARMUP_REQUESTS} warmup)...`, 'yellow');
  
  const rustHealth = await benchmarkEndpoint('Rust /health', RUST_PORT, '/health');
  log('✅ Rust complete', 'green');
  
  const tsHealth = await benchmarkEndpoint('TS /health', TS_PORT, '/health');
  log('✅ TypeScript complete', 'green');
  
  printBenchmarkTable(rustHealth, tsHealth);

  // Benchmark /supported endpoint
  log('\n\n═══ Benchmarking /supported endpoint ═══', 'bright');
  log(`Running ${BENCHMARK_REQUESTS} requests (after ${WARMUP_REQUESTS} warmup)...`, 'yellow');
  
  const rustSupported = await benchmarkEndpoint('Rust /supported', RUST_PORT, '/supported');
  log('✅ Rust complete', 'green');
  
  const tsSupported = await benchmarkEndpoint('TS /supported', TS_PORT, '/supported');
  log('✅ TypeScript complete', 'green');
  
  printBenchmarkTable(rustSupported, tsSupported);

  // Benchmark /verify endpoint (most important!)
  log('\n\n═══ Benchmarking /verify endpoint (MOST IMPORTANT) ═══', 'bright');
  log(`Running ${BENCHMARK_REQUESTS} requests (after ${WARMUP_REQUESTS} warmup)...`, 'yellow');
  
  const rustVerify = await benchmarkEndpoint('Rust /verify', RUST_PORT, '/verify', 'POST', VERIFY_PAYLOAD);
  log('✅ Rust complete', 'green');
  
  const tsVerify = await benchmarkEndpoint('TS /verify', TS_PORT, '/verify', 'POST', VERIFY_PAYLOAD);
  log('✅ TypeScript complete', 'green');
  
  printBenchmarkTable(rustVerify, tsVerify);

  // Concurrent load testing
  log('\n\n═══ Concurrent Load Testing ═══', 'bright');
  
  for (const concurrency of CONCURRENT_REQUESTS) {
    log(`\n📊 Testing with ${concurrency} concurrent requests...`, 'yellow');
    
    const rustConcurrent = await benchmarkConcurrent('Rust', RUST_PORT, '/verify', concurrency, 'POST', VERIFY_PAYLOAD);
    const tsConcurrent = await benchmarkConcurrent('TS', TS_PORT, '/verify', concurrency, 'POST', VERIFY_PAYLOAD);

    console.log('\n┌──────────────┬──────────┬──────────┬──────────┬──────────┐');
    console.log('│ Impl         │ Success  │ Failed   │ Time     │ Req/s    │');
    console.log('├──────────────┼──────────┼──────────┼──────────┼──────────┤');
    console.log(`│ Rust         │ ${rustConcurrent.successful.toString().padStart(8)} │ ${rustConcurrent.failed.toString().padStart(8)} │ ${rustConcurrent.totalDuration.padStart(6)}ms │ ${rustConcurrent.throughput.padStart(8)} │`);
    console.log(`│ TypeScript   │ ${tsConcurrent.successful.toString().padStart(8)} │ ${tsConcurrent.failed.toString().padStart(8)} │ ${tsConcurrent.totalDuration.padStart(6)}ms │ ${tsConcurrent.throughput.padStart(8)} │`);
    console.log('└──────────────┴──────────┴──────────┴──────────┴──────────┘');

    const throughputSpeedup = (parseFloat(rustConcurrent.throughput) / parseFloat(tsConcurrent.throughput)).toFixed(2);
    if (throughputSpeedup > 1) {
      log(`   ✅ Rust: ${throughputSpeedup}x higher throughput`, 'green');
    } else {
      log(`   ⚠️  TS: ${(1/throughputSpeedup).toFixed(2)}x higher throughput`, 'yellow');
    }
  }

  // Summary
  log('\n\n╔═══════════════════════════════════════════════════════════════╗', 'bright');
  log('║                        📊 SUMMARY                              ║', 'bright');
  log('╚═══════════════════════════════════════════════════════════════╝\n', 'bright');

  const verifySpeedup = (tsVerify.mean / rustVerify.mean).toFixed(2);
  
  log('Key Findings:', 'cyan');
  log(`✅ /verify latency (mean): Rust ${rustVerify.mean.toFixed(2)}ms vs TS ${tsVerify.mean.toFixed(2)}ms (${verifySpeedup}x speedup)`);
  log(`✅ /verify latency (p95):  Rust ${rustVerify.p95.toFixed(2)}ms vs TS ${tsVerify.p95.toFixed(2)}ms`);
  log(`✅ /verify latency (p99):  Rust ${rustVerify.p99.toFixed(2)}ms vs TS ${tsVerify.p99.toFixed(2)}ms`);
  
  log('\n💡 Notes:', 'yellow');
  log('   • Memory usage requires manual inspection via htop/Activity Monitor');
  log('   • Both servers should be running with similar configurations');
  log('   • Results may vary based on system load and hardware');
  log('   • Multiple runs recommended for statistical significance\n');
}

main().catch(console.error);

