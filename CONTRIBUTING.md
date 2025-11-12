# Contributing to Rust x402 Facilitator

Thank you for your interest in contributing! This project was built for the Solana x402 Hackathon.

## Development Setup

1. **Prerequisites:**
   - Rust 1.70+ ([Install](https://rustup.rs/))
   - Solana CLI ([Install](https://docs.solana.com/cli/install-solana-cli-tools))

2. **Clone and Build:**
   ```bash
   git clone https://github.com/cereza-rainier/rust-facilitator
   cd rust-facilitator
   cargo build --release
   ```

3. **Configure:**
   ```bash
   cp env.example .env
   # Edit .env with your private key
   ```

4. **Run:**
   ```bash
   cargo run --release --bin x402-facilitator
   ```

## Project Structure

- `src/` - Core facilitator implementation
- `demo/` - Demo applications showing usage
- `examples/` - FFI examples (Python, WASM)
- `tests/` - Integration tests
- `k8s/` - Kubernetes deployment configs

## Testing

```bash
# Run tests
cargo test

# Run benchmarks
./scripts/benchmark_basic.sh

# Demo
cd demo && npm install && npm run demo
```

## Code Style

- Run `cargo fmt` before committing
- Run `cargo clippy` to check for issues
- Follow Rust naming conventions

## Performance

This facilitator uses Rayon for true parallel processing. Key files:
- `src/parallel.rs` - Parallel batch verification
- `src/handlers/batch.rs` - Batch endpoint

## Submitting Changes

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests
5. Submit a pull request

## Questions?

Open an issue or check the [GETTING_STARTED.md](GETTING_STARTED.md) guide.

---

Built for the **Solana x402 Hackathon** 🏆

