// Demo Client - Pays for API Access
// This demonstrates the client side of x402 payments

const axios = require('axios');

const API_URL = process.env.API_URL || 'http://localhost:4000';

// Mock payment creation (for demo purposes)
// In production, this would create a real Solana transaction
function createMockPayment() {
  // This is a mock base64-encoded transaction for demo
  // In real use, you'd use @solana/web3.js to create actual transactions
  const mockTransaction = Buffer.from('mock_transaction_data_' + Date.now()).toString('base64');
  
  return {
    x402_version: 1,
    scheme: 'exact',
    network: 'solana-devnet',
    payload: {
      transaction: mockTransaction
    },
    timestamp: Math.floor(Date.now() / 1000)
  };
}

async function accessFreeEndpoint() {
  console.log('\n╔══════════════════════════════════════════════╗');
  console.log('║  Test 1: Accessing FREE endpoint             ║');
  console.log('╚══════════════════════════════════════════════╝\n');
  
  try {
    const response = await axios.get(`${API_URL}/free-data`);
    console.log('✅ Success! No payment required');
    console.log('📦 Response:', JSON.stringify(response.data, null, 2));
  } catch (error) {
    console.error('❌ Error:', error.message);
  }
}

async function accessPremiumEndpoint() {
  console.log('\n╔══════════════════════════════════════════════╗');
  console.log('║  Test 2: Accessing PREMIUM endpoint          ║');
  console.log('╚══════════════════════════════════════════════╝\n');
  
  console.log('💡 This endpoint requires payment...');
  console.log('');
  
  try {
    // Try without payment first
    console.log('📍 Step 1: Trying WITHOUT payment...');
    const noPaymentResponse = await axios.get(`${API_URL}/premium-data`);
    console.log('   Unexpected success:', noPaymentResponse.data);
  } catch (error) {
    if (error.response && error.response.status === 402) {
      console.log('   ✅ Correctly rejected - Payment Required!');
      console.log('   💵 Required:', error.response.data.required);
    } else {
      console.log('   ❌ Unexpected error:', error.message);
      return;
    }
  }
  
  // Now try with payment
  console.log('');
  console.log('📍 Step 2: Creating payment...');
  const payment = createMockPayment();
  console.log(`   ✅ Payment created (network: ${payment.network})`);
  
  console.log('');
  console.log('📍 Step 3: Sending payment to API...');
  const paymentHeader = Buffer.from(JSON.stringify(payment)).toString('base64');
  
  try {
    const startTime = Date.now();
    const response = await axios.get(`${API_URL}/premium-data`, {
      headers: {
        'X-Payment': paymentHeader
      }
    });
    const totalTime = Date.now() - startTime;
    
    console.log(`   ✅ Payment VERIFIED! (${totalTime}ms total)`);
    console.log('');
    console.log('🎉 SUCCESS! Got premium data:');
    console.log('─'.repeat(50));
    console.log(JSON.stringify(response.data, null, 2));
    console.log('─'.repeat(50));
    
    if (response.data.performance) {
      console.log('');
      console.log('⚡ Performance:');
      console.log(`   Verification: ${response.data.performance.verification_time_ms}ms`);
      console.log(`   Total time:   ${response.data.performance.total_time_ms}ms`);
    }
    
  } catch (error) {
    if (error.response) {
      console.log('   ❌ Payment verification failed');
      console.log('   Reason:', error.response.data);
    } else {
      console.log('   ❌ Error:', error.message);
      if (error.code === 'ECONNREFUSED') {
        console.log('   💡 Is the API server running? Start with: npm run server');
      }
    }
  }
}

async function demonstrateReplayProtection() {
  console.log('\n╔══════════════════════════════════════════════╗');
  console.log('║  Test 3: Replay Protection Demo              ║');
  console.log('╚══════════════════════════════════════════════╝\n');
  
  console.log('💡 Testing security: Can we reuse the same payment?');
  console.log('');
  
  // Create one payment
  const payment = createMockPayment();
  const paymentHeader = Buffer.from(JSON.stringify(payment)).toString('base64');
  
  // Try it twice
  console.log('📍 Attempt 1: Sending payment...');
  try {
    const response1 = await axios.get(`${API_URL}/premium-data`, {
      headers: { 'X-Payment': paymentHeader }
    });
    console.log('   ✅ First attempt succeeded');
  } catch (error) {
    console.log('   ⚠️  First attempt failed (mock transaction not recognized)');
  }
  
  console.log('');
  console.log('📍 Attempt 2: Reusing SAME payment (replay attack)...');
  try {
    const response2 = await axios.get(`${API_URL}/premium-data`, {
      headers: { 'X-Payment': paymentHeader }
    });
    console.log('   ❌ SECURITY ISSUE: Replay attack succeeded!');
  } catch (error) {
    if (error.response && error.response.data.reason?.includes('already been processed')) {
      console.log('   ✅ Replay attack BLOCKED!');
      console.log('   🔒 Facilitator prevented transaction reuse');
    } else {
      console.log('   ⚠️  Different error:', error.response?.data.reason || error.message);
    }
  }
}

async function runDemo() {
  console.log('');
  console.log('╔════════════════════════════════════════════════╗');
  console.log('║       x402 Rust Facilitator - Client Demo      ║');
  console.log('╚════════════════════════════════════════════════╝');
  
  console.log('');
  console.log('🎯 This demo shows:');
  console.log('   1. Free endpoint (no payment)');
  console.log('   2. Premium endpoint (requires payment)');
  console.log('   3. Security (replay protection)');
  console.log('');
  console.log('⏳ Starting in 2 seconds...');
  
  await new Promise(resolve => setTimeout(resolve, 2000));
  
  try {
    // Test 1: Free endpoint
    await accessFreeEndpoint();
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    // Test 2: Premium endpoint
    await accessPremiumEndpoint();
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    // Test 3: Replay protection (optional - comment out if causes issues)
    // await demonstrateReplayProtection();
    
    console.log('\n');
    console.log('╔════════════════════════════════════════════════╗');
    console.log('║              Demo Complete! 🎉                 ║');
    console.log('╚════════════════════════════════════════════════╝');
    console.log('');
    console.log('📊 View facilitator metrics:');
    console.log('   curl http://localhost:3000/metrics');
    console.log('');
    console.log('📋 View facilitator logs:');
    console.log('   docker-compose logs -f facilitator');
    console.log('');
    
  } catch (error) {
    console.error('\n❌ Demo failed:', error.message);
    console.log('');
    console.log('💡 Troubleshooting:');
    console.log('   1. Is the facilitator running? (./deploy.sh)');
    console.log('   2. Is the API server running? (npm run server)');
    console.log('   3. Check: curl http://localhost:3000/health');
    console.log('   4. Check: curl http://localhost:4000/health');
    console.log('');
  }
}

// Run the demo
runDemo().catch(console.error);

