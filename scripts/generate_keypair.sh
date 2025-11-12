#!/bin/bash

# Script to generate a Solana keypair for the facilitator

echo "🔑 Generating Solana Keypair for Facilitator..."
echo ""

# Check if solana-keygen is installed
if ! command -v solana-keygen &> /dev/null; then
    echo "❌ solana-keygen not found. Please install Solana CLI tools."
    echo "Install with: sh -c \"\$(curl -sSfL https://release.solana.com/stable/install)\""
    exit 1
fi

# Generate keypair
KEYPAIR_PATH="${1:-~/.config/solana/facilitator-devnet.json}"
mkdir -p "$(dirname "$KEYPAIR_PATH")"

echo "Generating keypair at: $KEYPAIR_PATH"
solana-keygen new --outfile "$KEYPAIR_PATH" --no-bip39-passphrase

echo ""
echo "✅ Keypair generated!"
echo ""

# Get public key
PUBKEY=$(solana-keygen pubkey "$KEYPAIR_PATH")
echo "📍 Public Key: $PUBKEY"
echo ""

# Instructions for funding
echo "💰 Next steps:"
echo ""
echo "1. Fund your wallet with devnet SOL:"
echo "   solana airdrop 2 -k $KEYPAIR_PATH --url devnet"
echo ""
echo "2. Check balance:"
echo "   solana balance -k $KEYPAIR_PATH --url devnet"
echo ""
echo "3. Get base58 private key for .env:"
echo "   solana-keygen pubkey $KEYPAIR_PATH"
echo ""
echo "4. Add to .env file:"
echo "   FEE_PAYER_PRIVATE_KEY=<use_the_full_keypair_bytes_as_base58>"
echo ""
echo "⚠️  Note: Converting the JSON array to base58 requires additional tools."
echo "For now, you can use the keypair file directly in testing."
echo ""

