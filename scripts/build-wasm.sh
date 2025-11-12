#!/bin/bash

# Build WebAssembly module
# This script compiles the Rust facilitator to WASM for browser usage

set -e

echo "🔨 Building x402 Facilitator for WebAssembly"
echo "============================================="
echo ""

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check for wasm-pack
if ! command -v wasm-pack &> /dev/null; then
    echo -e "${YELLOW}⚠️  wasm-pack not found. Installing...${NC}"
    cargo install wasm-pack
    echo ""
fi

# Check for wasm32 target
if ! rustup target list | grep -q "wasm32-unknown-unknown (installed)"; then
    echo -e "${YELLOW}⚠️  wasm32 target not installed. Adding...${NC}"
    rustup target add wasm32-unknown-unknown
    echo ""
fi

# Build for web
echo -e "${BLUE}📦 Building WASM module for web...${NC}"
wasm-pack build \
    --target web \
    --out-dir wasm-pkg \
    --release

echo ""
echo -e "${GREEN}✅ WASM build complete!${NC}"
echo ""

# Show output files
echo "📂 Generated files:"
ls -lh wasm-pkg/
echo ""

# Show file sizes
wasm_size=$(du -h wasm-pkg/x402_facilitator_bg.wasm | cut -f1)
echo "📊 WASM binary size: $wasm_size"
echo ""

echo "🎯 Next steps:"
echo "  1. Open examples/wasm/index.html in a browser"
echo "  2. Or serve with: python3 -m http.server 8000"
echo "  3. Navigate to: http://localhost:8000/examples/wasm/"
echo ""

echo -e "${GREEN}🚀 Your facilitator can now run in ANY browser!${NC}"

