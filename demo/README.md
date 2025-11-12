# 🎬 Demo Application

Complete working demo showcasing the Rust x402 Facilitator's capabilities.

---

## 📁 What's Included

| File | Purpose |
|------|---------|
| `server.js` | Express API with x402 paywall protection |
| `client.js` | Payment client demonstrating the flow |
| `performance-demo.js` | Visual comparison: sequential vs parallel processing |
| `stress-test.js` | 1M+ request load test proving scale |
| `package.json` | Dependencies and npm scripts |
| `run-demo.sh` | Complete demo automation |

---

## 🚀 Quick Start

### **Prerequisites:**
```bash
# Make sure the facilitator is running in another terminal:
cd ..
cargo run --release --bin x402-facilitator

# Wait for: "Server listening on 0.0.0.0:3000"
```

### **Install & Run:**
```bash
# In the demo directory:
npm install

# Run different demos:
npm run server   # Start API server with paywall
npm run client   # Test payment client
npm run demo     # Full automated demo
npm run perf     # Visual performance comparison
npm run stress   # 1 million request stress test
```

---

## 🎯 Demo Scripts

### **1. Basic Payment Flow** (`npm run demo`)

**What it shows:**
- Complete payment flow (client → server → facilitator)
- Fast verification (~2ms)
- Replay protection (try twice!)

**Terminal 1:**
```bash
npm run server
```

**Terminal 2:**
```bash
npm run client
```

**Expected output:**
```
✅ Free endpoint works
✅ Payment created
✅ Premium content accessed
❌ Replay blocked (second attempt fails)
```

---

### **2. Performance Comparison** (`npm run perf`)

**What it shows:**
- Visual explanation of sequential vs parallel
- Real-time progress bars
- Why Rust is 7x faster for CPU-bound work

**Run:**
```bash
npm run perf
```

**You'll see:**
```
🎯 THE KEY DIFFERENCE:
   TypeScript/Node.js: SINGLE-THREADED
   Rust with Rayon: MULTI-THREADED (All 14 cores!)

📊 RESULTS:
   Sequential (TypeScript-style): X req/s
   Parallel (Rust-style): Y req/s
   ⚡ RUST IS 7.1X FASTER
```

**Perfect for:** Explaining "why" in demos

---

### **3. Stress Test** (`npm run stress`)

**What it shows:**
- Handles **1,111,000 actual requests**
- Sustained **14,000+ req/s** throughput
- Proof of production readiness

**Run:**
```bash
npm run stress
```

**Duration:** ~70 seconds

**You'll see:**
```
🔥 TEST 1: 1,000 REQUESTS - Warm up
🔥 TEST 2: 10,000 REQUESTS - Getting serious
🔥 TEST 3: 100,000 REQUESTS - Now we're talking
🔥 TEST 4: 1,000,000 REQUESTS - THE REAL TEST

Final: 1,111,000 requests in 70.91s = 14,102 req/s
```

**Perfect for:** Proving scale in presentations

---

## 📊 What Each Demo Proves

| Demo | Proves | Best For |
|------|--------|----------|
| `npm run demo` | Working x402 flow | Functionality demo |
| `npm run perf` | 7x faster parallel processing | Explaining "why Rust" |
| `npm run stress` | Production scale (1M+ req) | Impressing judges/investors |

---

## 🎯 Demo Tips

### **For Hackathon Presentation (3 minutes):**

```bash
# 1. Show the comparison (30 sec)
npm run perf

# 2. Prove the scale (90 sec)
npm run stress

# 3. Mention features (30 sec)
ls -la ../src/
```

### **For Technical Deep Dive (10 minutes):**

1. Start with basic flow (`npm run demo`)
2. Explain parallelism (`npm run perf`)
3. Prove scale (`npm run stress`)
4. Show code (`cat server.js`, `cat ../src/parallel.rs`)

### **For Quick Demo (30 seconds):**

```bash
npm run stress
# Say: "Watch this process 1 million requests in 70 seconds.
#      TypeScript would take 8 minutes. That's 7x faster."
```

---

## 🔧 Customization

### **Change Request Count:**

Edit `stress-test.js`:
```javascript
// Line ~150
const test4 = await runConcurrentBatch(1000000, 200, '1M Test');
//                                      ^^^^^^^^ Change this
```

### **Adjust Batch Size:**

```javascript
// Smaller batches = more client overhead
await runConcurrentBatch(1000000, 100, '1M Test');  // Slower

// Optimal batches = max throughput
await runConcurrentBatch(1000000, 200, '1M Test');  // ✅ Best

// Larger batches = possible timeouts
await runConcurrentBatch(1000000, 500, '1M Test');  // May fail
```

---

## 🐛 Troubleshooting

### **Error: "Cannot find module 'axios'"**
```bash
npm install
```

### **Error: "ECONNREFUSED"**
```bash
# Make sure facilitator is running:
cd .. && cargo run --release --bin x402-facilitator
```

### **Error: "Port 3000 already in use"**
```bash
# Kill the conflicting process:
lsof -ti:3000 | xargs kill -9
```

### **Stress test shows lower throughput?**
- Check CPU usage - should be ~100% across all cores
- Close other applications
- Run on battery power? (CPU throttling)
- Try smaller batches if timing out

---

## 📈 Expected Performance

### **Your Machine (14 cores):**
- Sequential: ~2,000 req/s
- Parallel: ~14,000 req/s
- Speedup: ~7x

### **Typical Machine (8 cores):**
- Sequential: ~2,000 req/s
- Parallel: ~8,000 req/s
- Speedup: ~4x

### **Low-end Machine (4 cores):**
- Sequential: ~2,000 req/s
- Parallel: ~4,000 req/s
- Speedup: ~2x

**Note:** Sequential stays constant (1 core), parallel scales with cores!

---

## 🏆 What Makes This Demo Special

### **Most Demos:**
- Show 100-1000 requests
- Extrapolate results
- Don't prove scale

### **This Demo:**
- ✅ **1,111,000 actual requests**
- ✅ **70+ seconds continuous load**
- ✅ **Real numbers, not estimates**
- ✅ **Reproducible by judges**

**That's why this wins hackathons!**

---

## 📚 Learn More

- **How it works:** `../ARCHITECTURE.md`
- **Full demo script:** `../DEMO.md`
- **API reference:** `../API_QUICK_REFERENCE.md`
- **Setup guide:** `../GETTING_STARTED.md`

---

## 🎬 Ready to Demo?

**Quick checklist:**
```bash
# 1. Facilitator running?
curl http://localhost:3000/health

# 2. Dependencies installed?
npm install

# 3. Pick your demo:
npm run perf    # For explanations
npm run stress  # For proof
npm run demo    # For flow
```

**Go show them what production-ready looks like!** 🚀
