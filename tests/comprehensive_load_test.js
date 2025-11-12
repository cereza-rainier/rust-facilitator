#!/usr/bin/env node

/**
 * Comprehensive Load Test - Realistic Payment Processing Workload
 * 
 * Simulates real-world usage:
 * - 70% /verify requests
 * - 20% /settle requests  
 * - 5% /supported requests
 * - 5% /health checks
 */

const RUST_PORT = 3000;
const TS_PORT = 3001;

// Realistic transaction payloads (will fail, but measure processing time)
const SAMPLE_VERIFY = {
  transaction: "AgAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==",
  payerAccount: "4zvwRjXUKGfvwnParsHAS3HuSVzV5cA4McphgmoCtajS"
};

const SAMPLE_SETTLE = {
  transaction: "AgAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=="
};

async function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

async function makeRequest(port, endpoint, method = 'GET', body = null) {
  const start = performance.now();
  try {
    const options = {
      method,
      headers: { 'Content-Type': 'application/json' }
    };
    if (body) {
      options.body = JSON.stringify(body);
    }
    
    const response = await fetch(`http://localhost:${port}${endpoint}`, options);
    const end = performance.now();
    
    return {
      success: true,
      status: response.status,
      latency: end - start,
      endpoint
    };
  } catch (error) {
    const end = performance.now();
    return {
      success: false,
      error: error.message,
      latency: end - start,
      endpoint
    };
  }
}

function selectEndpoint() {
  const rand = Math.random();
  if (rand < 0.70) {
    return { endpoint: '/verify', method: 'POST', body: SAMPLE_VERIFY };
  } else if (rand < 0.90) {
    return { endpoint: '/settle', method: 'POST', body: SAMPLE_SETTLE };
  } else if (rand < 0.95) {
    return { endpoint: '/supported', method: 'GET', body: null };
  } else {
    return { endpoint: '/health', method: 'GET', body: null };
  }
}

async function runLoadTest(name, port, duration, concurrency) {
  console.log(`\n🔥 ${name} - Load Test (${duration}s, ${concurrency} concurrent)`);
  
  const results = {
    verify: [],
    settle: [],
    supported: [],
    health: [],
    errors: 0,
    total: 0
  };
  
  const startTime = Date.now();
  const endTime = startTime + (duration * 1000);
  
  // Run concurrent workers
  const workers = [];
  for (let i = 0; i < concurrency; i++) {
    workers.push((async () => {
      while (Date.now() < endTime) {
        const { endpoint, method, body } = selectEndpoint();
        const result = await makeRequest(port, endpoint, method, body);
        
        results.total++;
        if (result.success) {
          const key = endpoint.substring(1); // Remove leading /
          if (results[key]) {
            results[key].push(result.latency);
          }
        } else {
          results.errors++;
        }
        
        // Small delay to avoid hammering
        await sleep(10);
      }
    })());
  }
  
  await Promise.all(workers);
  
  // Analyze results
  const analyze = (latencies) => {
    if (latencies.length === 0) return null;
    const sorted = [...latencies].sort((a, b) => a - b);
    return {
      count: latencies.length,
      min: sorted[0],
      mean: latencies.reduce((a, b) => a + b, 0) / latencies.length,
      p50: sorted[Math.floor(sorted.length * 0.50)],
      p95: sorted[Math.floor(sorted.length * 0.95)],
      p99: sorted[Math.floor(sorted.length * 0.99)],
      max: sorted[sorted.length - 1]
    };
  };
  
  console.log(`\n📊 ${name} Results:`);
  console.log(`   Total requests: ${results.total}`);
  console.log(`   Errors: ${results.errors} (${(results.errors/results.total*100).toFixed(1)}%)`);
  console.log(`   Throughput: ${(results.total / duration).toFixed(1)} req/s`);
  
  for (const endpoint of ['verify', 'settle', 'supported', 'health']) {
    const stats = analyze(results[endpoint]);
    if (stats && stats.count > 0) {
      console.log(`\n   /${endpoint}:`);
      console.log(`     Count: ${stats.count}`);
      console.log(`     Mean:  ${stats.mean.toFixed(2)}ms`);
      console.log(`     P50:   ${stats.p50.toFixed(2)}ms`);
      console.log(`     P95:   ${stats.p95.toFixed(2)}ms`);
      console.log(`     P99:   ${stats.p99.toFixed(2)}ms`);
    }
  }
  
  return results;
}

async function main() {
  console.log('╔═══════════════════════════════════════════════════════════╗');
  console.log('║  🔬 COMPREHENSIVE LOAD TEST - RUST vs TYPESCRIPT 🔬      ║');
  console.log('╚═══════════════════════════════════════════════════════════╝\n');
  
  console.log('📋 Test Configuration:');
  console.log('   Workload: 70% verify, 20% settle, 5% supported, 5% health');
  console.log('   Duration: 30 seconds per test');
  console.log('   Concurrency: 10 → 50 → 100 concurrent requests\n');
  
  // Test scenarios
  const scenarios = [
    { name: 'Low Load', duration: 30, concurrency: 10 },
    { name: 'Medium Load', duration: 30, concurrency: 50 },
    { name: 'High Load', duration: 30, concurrency: 100 }
  ];
  
  for (const scenario of scenarios) {
    console.log('\n' + '='.repeat(60));
    console.log(`\n📈 ${scenario.name} (${scenario.concurrency} concurrent)`);
    console.log('='.repeat(60));
    
    // Test Rust
    await runLoadTest('Rust', RUST_PORT, scenario.duration, scenario.concurrency);
    
    await sleep(2000); // Cooldown
    
    // Test TypeScript
    await runLoadTest('TypeScript', TS_PORT, scenario.duration, scenario.concurrency);
    
    await sleep(5000); // Longer cooldown between scenarios
  }
  
  console.log('\n✅ Comprehensive load test complete!\n');
}

main().catch(console.error);

