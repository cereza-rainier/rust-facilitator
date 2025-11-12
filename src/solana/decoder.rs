use anyhow::{anyhow, Result};
use base64::{engine::general_purpose, Engine as _};
use solana_sdk::{
    pubkey::Pubkey,
    signature::Signature,
    transaction::Transaction,
};

/// Decode a base64-encoded transaction
pub fn decode_transaction_from_base64(encoded: &str) -> Result<Transaction> {
    let bytes = general_purpose::STANDARD
        .decode(encoded)
        .map_err(|e| anyhow!("Failed to decode base64: {}", e))?;

    let transaction: Transaction = bincode::deserialize(&bytes)
        .map_err(|e| anyhow!("Failed to deserialize transaction: {}", e))?;

    Ok(transaction)
}

/// Get the fee payer (first signer) from a transaction
pub fn get_payer_from_transaction(tx: &Transaction) -> String {
    if let Some(first_key) = tx.message.account_keys.first() {
        first_key.to_string()
    } else {
        "unknown".to_string()
    }
}

/// Get all signers from a transaction
pub fn get_signers(tx: &Transaction) -> Vec<Pubkey> {
    let num_required_signatures = tx.message.header.num_required_signatures as usize;
    tx.message
        .account_keys
        .iter()
        .take(num_required_signatures)
        .cloned()
        .collect()
}

/// Check if transaction is signed by a specific pubkey
pub fn is_signed_by(tx: &Transaction, pubkey: &Pubkey) -> bool {
    let signers = get_signers(tx);
    signers.contains(pubkey)
}

/// Get transaction signature if fully signed
pub fn get_transaction_signature(tx: &Transaction) -> Option<Signature> {
    tx.signatures.first().cloned()
}

/// Check if transaction is fully signed
pub fn is_fully_signed(tx: &Transaction) -> bool {
    let num_required = tx.message.header.num_required_signatures as usize;
    tx.signatures
        .iter()
        .take(num_required)
        .all(|sig| sig != &Signature::default())
}

/// Check if transaction is partially signed (client signed, facilitator hasn't)
pub fn is_partially_signed(tx: &Transaction) -> bool {
    let num_required = tx.message.header.num_required_signatures as usize;
    if num_required < 2 {
        return false;
    }

    // First signature should be empty (facilitator/fee payer)
    // Second+ signatures should be present (client)
    let has_empty_first = tx.signatures.first() == Some(&Signature::default());
    let has_client_sigs = tx
        .signatures
        .iter()
        .skip(1)
        .take(num_required - 1)
        .all(|sig| sig != &Signature::default());

    has_empty_first && has_client_sigs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_payer_from_empty_tx() {
        use solana_sdk::message::Message;

        let tx = Transaction::new_unsigned(Message::new(&[], None));
        let payer = get_payer_from_transaction(&tx);
        assert_eq!(payer, "unknown");
    }
}
