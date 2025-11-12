# API Quick Reference

**x402 Rust Facilitator - All Endpoints & Interfaces**

---

## 🚀 HTTP API Endpoints

### Base URL
```
http://localhost:3000
```

---

### 1. Health Check
```http
GET /health
```

**Response:**
```json
{
  "status": "healthy",
  "version": "2.0.0",
  "uptime_seconds": 3600,
  "rpc_status": "connected",
  "network": "solana-devnet"
}
```

---

### 2. Supported Schemes
```http
GET /supported
```

**Response:**
```json
{
  "schemes": [
    {
      "scheme": "exact",
      "networks": ["solana", "solana-devnet"],
      "description": "Exact amount verification on Solana"
    }
  ]
}
```

---

### 3. Verify Payment (Single)
```http
POST /verify
Content-Type: application/json
```

**Request Body:**
```json
{
  "payment_payload": {
    "x402_version": 1,
    "scheme": "exact",
    "network": "solana-devnet",
    "payload": {
      "transaction": "base64_encoded_transaction"
    },
    "timestamp": 1699000000
  },
  "payment_requirements": {
    "scheme": "exact",
    "network": "solana-devnet",
    "max_amount_required": "1000000",
    "asset": "SOL",
    "pay_to": "recipient_address",
    "resource": "/api/data",
    "description": "Premium API Access",
    "mime_type": "application/json",
    "max_timeout_seconds": 30,
    "extra": {
      "fee_payer": "fee_payer_address"
    }
  }
}
```

**Response:**
```json
{
  "is_valid": true,
  "payer": "wallet_address"
}
```

---

### 4. Verify Payment (Batch) 🆕
```http
POST /verify/batch
Content-Type: application/json
```

**Request Body:**
```json
[
  {
    "payment_payload": { ... },
    "payment_requirements": { ... }
  },
  {
    "payment_payload": { ... },
    "payment_requirements": { ... }
  }
]
```

**Response:**
```json
[
  {
    "is_valid": true,
    "payer": "wallet_address_1"
  },
  {
    "is_valid": false,
    "invalid_reason": "Insufficient amount"
  }
]
```

**Performance:**
- 8x faster than sequential on 8-core machines
- Utilizes all CPU cores simultaneously
- Ideal for bulk verification

**Example:**
```bash
curl -X POST http://localhost:3000/verify/batch \
  -H "Content-Type: application/json" \
  -d '[
    {"payment_payload": {...}, "payment_requirements": {...}},
    {"payment_payload": {...}, "payment_requirements": {...}}
  ]'
```

---

### 5. Settle Payment
```http
POST /settle
Content-Type: application/json
```

**Request Body:**
```json
{
  "payment_payload": { ... },
  "payment_requirements": { ... }
}
```

**Response:**
```json
{
  "is_valid": true,
  "transaction_signature": "5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp...",
  "payer": "wallet_address"
}
```

---

### 6. Prometheus Metrics
```http
GET /metrics
```

**Response:** Prometheus-format metrics
```
# HELP verify_requests_total Total verification requests
# TYPE verify_requests_total counter
verify_requests_total{network="solana-devnet"} 1234

# HELP verification_duration_seconds Verification duration
# TYPE verification_duration_seconds histogram
verification_duration_seconds_bucket{le="0.005",network="solana-devnet"} 950
...
```

---

### 7. Swagger UI
```http
GET /swagger-ui/
```

Interactive API documentation

---

## 🔗 FFI Interface (Multi-Language)

### Python
```python
from x402_ffi import X402Facilitator

facilitator = X402Facilitator()

payment = {
    "x402_version": 1,
    "scheme": "exact",
    "network": "solana-devnet",
    "payload": {"transaction": "..."}
}

requirements = {
    "scheme": "exact",
    "network": "solana-devnet",
    # ... other fields
}

result = facilitator.verify(payment, requirements)

if result.is_valid:
    print(f"✅ Payer: {result.payer}")
else:
    print(f"❌ Error: {result.error_message}")
```

**Performance:** 0.14ms (vs 5-50ms HTTP)

---

### C Header
```c
#include "x402_facilitator.h"

// Initialize
int status = x402_init();

// Verify
CVerifyResult result = x402_verify_payment(
    payment_json,
    requirements_json
);

// Use result
if (result.is_valid) {
    printf("Payer: %s\n", result.payer);
}

// Free memory
x402_free_result(result);
```

---

### Go (CGO)
```go
// #cgo LDFLAGS: -L./target/release -lx402_facilitator
// #include "x402_facilitator.h"
import "C"

result := C.x402_verify_payment(
    C.CString(paymentJSON),
    C.CString(requirementsJSON)
)
defer C.x402_free_result(result)

if result.is_valid {
    fmt.Printf("Valid! Payer: %s\n", C.GoString(result.payer))
}
```

---

## 🌐 WebAssembly Interface

### JavaScript (Browser)
```javascript
import init, { WasmVerifier } from './wasm-pkg/x402_facilitator.js';

// Initialize WASM
await init();
const verifier = new WasmVerifier();

// Verify payment
const payment = {
    x402_version: 1,
    scheme: "exact",
    network: "solana-devnet",
    payload: { transaction: "..." }
};

const requirements = {
    scheme: "exact",
    network: "solana-devnet",
    // ... other fields
};

const result = verifier.verify(payment, requirements);

if (result.is_valid) {
    console.log(`✅ Payer: ${result.payer}`);
}

// Get info
console.log(`Version: ${verifier.version()}`);
console.log(`Supports exact: ${verifier.supports_scheme("exact")}`);
```

**Performance:** <1ms, zero network latency, works offline

---

## 📊 Performance Comparison

| Method | Latency | Use Case |
|--------|---------|----------|
| **HTTP Single** | 5-10ms | Standard API usage |
| **HTTP Batch** | 5-10ms | Bulk verification (8x faster) |
| **FFI (Python/Go/Java)** | <1ms | High-performance services |
| **WASM (Browser)** | <1ms | Client-side, offline apps |

---

## 🎯 Quick Examples

### Simple Verification
```bash
curl -X POST http://localhost:3000/verify \
  -H "Content-Type: application/json" \
  -d @payment.json
```

### Batch Verification
```bash
curl -X POST http://localhost:3000/verify/batch \
  -H "Content-Type: application/json" \
  -d @batch_payments.json
```

### Python FFI
```bash
cd examples/ffi/python
python3 x402_ffi.py
```

### Build WASM
```bash
./scripts/build-wasm.sh
python3 -m http.server 8000
# Open: http://localhost:8000/examples/wasm/
```

---

## 🔧 Configuration

### Environment Variables
```bash
SOLANA_RPC_URL=https://api.devnet.solana.com
NETWORK=solana-devnet
FEE_PAYER_PRIVATE_KEY=your_private_key
PORT=3000
LOG_LEVEL=info
CACHE_SIZE=10000
CACHE_TTL_SECONDS=60
RATE_LIMIT_PER_SECOND=100
```

### Docker
```bash
docker-compose up -d
```

---

## 📚 Documentation Links

| Topic | Link |
|-------|------|
| **Full Architecture** | [ARCHITECTURE.md](./ARCHITECTURE.md) |
| **Getting Started** | [QUICKSTART.md](./QUICKSTART.md) |
| **Rust Superpowers** | [RUST_SUPERPOWERS_SUMMARY.md](./RUST_SUPERPOWERS_SUMMARY.md) |
| **Phase 1 Complete** | [PHASE1_COMPLETION.md](./PHASE1_COMPLETION.md) |
| **Test Guide** | [TEST_PHASE1.md](./TEST_PHASE1.md) |
| **Parallel Demo** | [PARALLEL_DEMO.md](./PARALLEL_DEMO.md) |
| **FFI Python** | [examples/ffi/python/README.md](./examples/ffi/python/README.md) |
| **WASM Demo** | [examples/wasm/README.md](./examples/wasm/README.md) |

---

## 🆘 Common Issues

### "Connection refused"
```bash
# Start the facilitator first
cargo run --release
```

### "Library not found" (FFI)
```bash
# Build release first
cargo build --release
export LD_LIBRARY_PATH=./target/release  # Linux
export DYLD_LIBRARY_PATH=./target/release  # macOS
```

### "WASM module not found"
```bash
# Build WASM first
./scripts/build-wasm.sh
# Serve via HTTP (not file://)
python3 -m http.server 8000
```

---

## 🎯 Quick Decision Matrix

**Choose HTTP API if:**
- Standard REST integration
- Language has good HTTP client
- Network latency acceptable

**Choose FFI if:**
- Need maximum performance
- Local/embedded usage
- Multi-language microservices

**Choose WASM if:**
- Browser-based application
- Need offline support
- Decentralized architecture
- Edge computing

---

**Updated:** November 8, 2025  
**Version:** 2.0.0  
**All Interfaces:** HTTP, FFI, WASM ✅

