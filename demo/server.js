// Demo API Server with Paywall
// This is a simple Express server that requires payment for premium endpoints

const express = require('express');
const axios = require('axios');

const app = express();
app.use(express.json());

// Configuration
const FACILITATOR_URL = process.env.FACILITATOR_URL || 'http://localhost:3000';
const YOUR_WALLET = process.env.YOUR_WALLET || 'DEMO_WALLET_ADDRESS';
const FEE_PAYER_PUBKEY = process.env.FEE_PAYER_PUBKEY || 'DEMO_FEE_PAYER';

console.log('🔧 Configuration:');
console.log(`   Facilitator: ${FACILITATOR_URL}`);
console.log(`   Your Wallet: ${YOUR_WALLET}`);
console.log('');

// Free endpoint - no payment required
app.get('/free-data', (req, res) => {
  console.log('✅ /free-data - No payment required');
  
  res.json({
    success: true,
    data: {
      message: 'This is free data',
      info: 'No payment required for this endpoint!',
      timestamp: new Date().toISOString()
    }
  });
});

// Premium endpoint - requires payment
app.get('/premium-data', async (req, res) => {
  const startTime = Date.now();
  console.log('💰 /premium-data - Payment required');
  
  // 1. Get payment from header
  const paymentHeader = req.headers['x-payment'];
  
  if (!paymentHeader) {
    console.log('   ❌ No payment provided');
    return res.status(402).json({
      error: 'Payment Required',
      message: 'Send payment in X-Payment header',
      required: {
        amount: '1000000 lamports (0.001 SOL)',
        recipient: YOUR_WALLET
      }
    });
  }

  // 2. Parse payment
  let payment;
  try {
    const decoded = Buffer.from(paymentHeader, 'base64').toString();
    payment = JSON.parse(decoded);
    console.log(`   📦 Payment received from network: ${payment.network}`);
  } catch (error) {
    console.log('   ❌ Invalid payment format');
    return res.status(400).json({
      error: 'Invalid Payment',
      message: 'Payment header must be base64-encoded JSON'
    });
  }

  // 3. Define payment requirements
  const requirements = {
    scheme: 'exact',
    network: 'solana-devnet',
    max_amount_required: '1000000', // 0.001 SOL
    asset: 'So11111111111111111111111111111111111111112', // Native SOL
    pay_to: YOUR_WALLET,
    resource: '/premium-data',
    description: 'Premium API Access - Demo',
    mime_type: 'application/json',
    max_timeout_seconds: 30,
    extra: {
      fee_payer: FEE_PAYER_PUBKEY
    }
  };

  // 4. Verify payment with facilitator
  try {
    console.log('   🔍 Verifying payment with facilitator...');
    const verifyStart = Date.now();
    
    const verifyResponse = await axios.post(`${FACILITATOR_URL}/verify`, {
      payment_payload: payment,
      payment_requirements: requirements
    });

    const verifyTime = Date.now() - verifyStart;
    console.log(`   ⏱️  Verification took ${verifyTime}ms`);

    if (!verifyResponse.data.is_valid) {
      console.log(`   ❌ Payment invalid: ${verifyResponse.data.invalid_reason}`);
      return res.status(402).json({
        error: 'Payment Invalid',
        reason: verifyResponse.data.invalid_reason,
        message: 'The payment did not meet requirements'
      });
    }

    // 5. Payment verified! Return premium data
    const totalTime = Date.now() - startTime;
    console.log(`   ✅ Payment verified! Payer: ${verifyResponse.data.payer}`);
    console.log(`   ⏱️  Total request time: ${totalTime}ms\n`);
    
    return res.json({
      success: true,
      data: {
        secret: '🎉 This is premium data worth 0.001 SOL!',
        message: 'Thanks for your payment!',
        payer: verifyResponse.data.payer,
        premium_info: {
          key: 'secret_api_key_123',
          access_level: 'premium',
          valid_until: new Date(Date.now() + 86400000).toISOString() // 24h
        }
      },
      performance: {
        verification_time_ms: verifyTime,
        total_time_ms: totalTime
      }
    });

  } catch (error) {
    console.log(`   ❌ Verification error: ${error.message}\n`);
    
    // Check if facilitator is running
    if (error.code === 'ECONNREFUSED') {
      return res.status(503).json({
        error: 'Facilitator Unavailable',
        message: 'Could not connect to payment facilitator',
        hint: 'Is the facilitator running on http://localhost:3000?'
      });
    }
    
    return res.status(500).json({
      error: 'Verification Failed',
      message: error.message
    });
  }
});

// Health check
app.get('/health', (req, res) => {
  res.json({
    status: 'healthy',
    service: 'demo-api',
    facilitator: FACILITATOR_URL
  });
});

// Start server
const PORT = process.env.API_PORT || 4000;
app.listen(PORT, () => {
  console.log('╔════════════════════════════════════════════════╗');
  console.log('║   💰 Demo API Server with x402 Paywall        ║');
  console.log('╚════════════════════════════════════════════════╝');
  console.log('');
  console.log(`🌐 Server running on http://localhost:${PORT}`);
  console.log('');
  console.log('📡 Endpoints:');
  console.log(`   GET  http://localhost:${PORT}/free-data     (Free)`);
  console.log(`   GET  http://localhost:${PORT}/premium-data  (Requires payment)`);
  console.log(`   GET  http://localhost:${PORT}/health        (Health check)`);
  console.log('');
  console.log('💡 Test with:');
  console.log('   curl http://localhost:4000/free-data');
  console.log('   node client.js  (for premium endpoint)');
  console.log('');
});

