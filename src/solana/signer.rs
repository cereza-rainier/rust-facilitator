use anyhow::{anyhow, Result};
use solana_sdk::{
    signature::{Keypair, Signer as SolanaSigner},
    transaction::Transaction,
};
use bs58;

/// Load keypair from base58-encoded private key
pub fn load_keypair_from_base58(private_key: &str) -> Result<Keypair> {
    let decoded = bs58::decode(private_key)
        .into_vec()
        .map_err(|e| anyhow!("Failed to decode base58 private key: {}", e))?;

    Keypair::from_bytes(&decoded)
        .map_err(|e| anyhow!("Failed to create keypair from bytes: {}", e))
}

/// Sign a transaction with the fee payer keypair
pub fn sign_transaction_as_fee_payer(
    transaction: &mut Transaction,
    fee_payer: &Keypair,
) -> Result<()> {
    // The fee payer should be the first signer
    // Client has already signed (second+ signers)
    
    // Get recent blockhash (should already be in transaction)
    let message = &transaction.message;
    
    // Sign the transaction
    let signature = fee_payer.sign_message(message.serialize().as_slice());
    
    // Set the fee payer signature (first position)
    if transaction.signatures.is_empty() {
        transaction.signatures.push(signature);
    } else {
        transaction.signatures[0] = signature;
    }
    
    Ok(())
}

/// Check if transaction is fully signed
pub fn is_transaction_fully_signed(transaction: &Transaction) -> bool {
    let num_required = transaction.message.header.num_required_signatures as usize;
    
    if transaction.signatures.len() < num_required {
        return false;
    }
    
    // Check that all required signatures are present (not default)
    transaction.signatures
        .iter()
        .take(num_required)
        .all(|sig| sig.as_ref() != &[0u8; 64])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_keypair() {
        // Generate a test keypair
        let keypair = Keypair::new();
        let base58_key = bs58::encode(&keypair.to_bytes()).into_string();
        
        // Load it back
        let loaded = load_keypair_from_base58(&base58_key).unwrap();
        
        assert_eq!(keypair.pubkey(), loaded.pubkey());
    }
}

