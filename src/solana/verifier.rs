use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::CompiledInstruction,
    message::Message,
    pubkey::Pubkey,
    transaction::Transaction,
};
use spl_associated_token_account::get_associated_token_address;

use crate::cache::AccountCache;
use crate::error::VerificationError;
use crate::types::requests::PaymentRequirements;

/// Verify that the transaction has the correct number of instructions (3 or 4)
/// Returns true if has CreateATA instruction (4 instructions), false if not (3 instructions)
pub fn verify_instruction_count(tx: &Transaction) -> Result<bool, VerificationError> {
    let count = tx.message.instructions.len();

    if count != 3 && count != 4 {
        return Err(VerificationError::InvalidInstructionCount);
    }

    Ok(count == 4) // true if has CreateATA instruction
}

/// Get the compute budget program ID
pub fn compute_budget_program_id() -> Pubkey {
    // ComputeBudget111111111111111111111111111111
    "ComputeBudget111111111111111111111111111111"
        .parse()
        .unwrap()
}

/// Verify that the compute limit instruction is valid
pub fn verify_compute_limit_instruction(
    instruction: &CompiledInstruction,
    message: &Message,
) -> Result<(), VerificationError> {
    // Check program ID
    let program_id = &message.account_keys[instruction.program_id_index as usize];
    let compute_budget_id = compute_budget_program_id();

    if program_id != &compute_budget_id {
        return Err(VerificationError::InvalidComputeLimitInstruction);
    }

    // Check discriminator (2 = SetComputeUnitLimit)
    if instruction.data.is_empty() || instruction.data[0] != 2 {
        return Err(VerificationError::InvalidComputeLimitInstruction);
    }

    Ok(())
}

/// Verify that the compute price instruction is valid and not too high
pub fn verify_compute_price_instruction(
    instruction: &CompiledInstruction,
    message: &Message,
) -> Result<(), VerificationError> {
    // Check program ID
    let program_id = &message.account_keys[instruction.program_id_index as usize];
    let compute_budget_id = compute_budget_program_id();

    if program_id != &compute_budget_id {
        return Err(VerificationError::InvalidComputePriceInstruction);
    }

    // Check discriminator (3 = SetComputeUnitPrice)
    if instruction.data.is_empty() || instruction.data[0] != 3 {
        return Err(VerificationError::InvalidComputePriceInstruction);
    }

    // Parse price (8 bytes after discriminator)
    if instruction.data.len() < 9 {
        return Err(VerificationError::InvalidComputePriceInstruction);
    }

    let price_bytes: [u8; 8] = instruction.data[1..9]
        .try_into()
        .map_err(|_| VerificationError::InvalidComputePriceInstruction)?;
    let micro_lamports = u64::from_le_bytes(price_bytes);

    // Check max price: 5 lamports = 5_000_000 micro-lamports
    // This protects the facilitator from gas price abuse
    if micro_lamports > 5_000_000 {
        return Err(VerificationError::ComputePriceTooHigh);
    }

    Ok(())
}

/// Verify that the fee payer is not included in any instruction's accounts
/// This is critical for security - prevents the facilitator from being tricked
/// into transferring their own funds
pub fn verify_fee_payer_safety(
    tx: &Transaction,
    fee_payer: &Pubkey,
) -> Result<(), VerificationError> {
    for instruction in &tx.message.instructions {
        // Check all account indices in this instruction
        for account_index in &instruction.accounts {
            let account = &tx.message.account_keys[*account_index as usize];
            if account == fee_payer {
                return Err(VerificationError::FeePayerInInstructionAccounts);
            }
        }
    }

    Ok(())
}

/// Get SPL Token program ID
pub fn spl_token_program_id() -> Pubkey {
    spl_token::ID
}

/// Get SPL Token-2022 program ID
pub fn spl_token_2022_program_id() -> Pubkey {
    spl_token_2022::ID
}

/// Check if an account exists (with caching)
pub async fn check_account_exists(
    rpc_client: &RpcClient,
    cache: &AccountCache,
    pubkey: &Pubkey,
) -> Result<bool, VerificationError> {
    // Try cache first
    if let Some(_account) = cache.get(pubkey).await {
        tracing::debug!("✅ Cache HIT for account: {}", pubkey);
        return Ok(true);
    }
    
    tracing::debug!("❌ Cache MISS for account: {}, checking RPC", pubkey);
    
    // Fallback to RPC
    match rpc_client.get_account(pubkey) {
        Ok(account) => {
            // Cache the result
            cache.insert(*pubkey, account).await;
            Ok(true)
        }
        Err(_) => Ok(false),
    }
}

/// Verify transfer instruction
pub fn verify_transfer_instruction(
    instruction: &CompiledInstruction,
    message: &Message,
    requirements: &PaymentRequirements,
    fee_payer: &Pubkey,
    has_create_ata: bool,
    rpc_client: &RpcClient,
) -> Result<(), VerificationError> {
    // Check if it's a token transfer instruction
    let program_id = &message.account_keys[instruction.program_id_index as usize];
    let token_program = spl_token_program_id();
    let token_2022_program = spl_token_2022_program_id();

    if program_id != &token_program && program_id != &token_2022_program {
        return Err(VerificationError::NotATransferInstruction);
    }

    // Parse transfer instruction
    // TransferChecked format: discriminator(1) + amount(8) + decimals(1)
    if instruction.data.len() < 10 || instruction.data[0] != 12 {
        return Err(VerificationError::NotATransferInstruction);
    }

    // Get amount from instruction
    let amount_bytes: [u8; 8] = instruction.data[1..9]
        .try_into()
        .map_err(|_| VerificationError::NotATransferInstruction)?;
    let amount = u64::from_le_bytes(amount_bytes);

    // Verify amount matches exactly
    let required_amount: u64 = requirements
        .max_amount_required
        .parse()
        .map_err(|_| VerificationError::AmountMismatch)?;

    if amount != required_amount {
        return Err(VerificationError::AmountMismatch);
    }

    // Get accounts from transfer instruction
    // TransferChecked accounts: [source, mint, destination, authority, ...]
    if instruction.accounts.len() < 4 {
        return Err(VerificationError::NotATransferInstruction);
    }

    let source_idx = instruction.accounts[0] as usize;
    let destination_idx = instruction.accounts[2] as usize;
    let authority_idx = instruction.accounts[3] as usize;

    let source = &message.account_keys[source_idx];
    let destination = &message.account_keys[destination_idx];
    let authority = &message.account_keys[authority_idx];

    // Verify fee payer is not the authority (critical security check!)
    if authority == fee_payer {
        return Err(VerificationError::FeePayerTransferringFunds);
    }

    // Calculate expected destination ATA
    let pay_to: Pubkey = requirements
        .pay_to
        .parse()
        .map_err(|_| VerificationError::TransferToIncorrectATA)?;
    let asset: Pubkey = requirements
        .asset
        .parse()
        .map_err(|_| VerificationError::TransferToIncorrectATA)?;

    let expected_destination = get_associated_token_address(&pay_to, &asset);

    // Verify destination is correct ATA
    if destination != &expected_destination {
        return Err(VerificationError::TransferToIncorrectATA);
    }

    // Check account existence
    // Source ATA must exist
    if rpc_client.get_account(source).is_err() {
        return Err(VerificationError::SenderATANotFound);
    }

    // Destination ATA must exist if no CreateATA instruction
    if !has_create_ata && rpc_client.get_account(&expected_destination).is_err() {
        return Err(VerificationError::ReceiverATANotFound);
    }

    Ok(())
}

/// Verify CreateATA instruction (if present)
pub fn verify_create_ata_instruction(
    instruction: &CompiledInstruction,
    message: &Message,
    requirements: &PaymentRequirements,
) -> Result<(), VerificationError> {
    // Check program ID is associated token program
    let program_id = &message.account_keys[instruction.program_id_index as usize];
    let ata_program = spl_associated_token_account::ID;

    if program_id != &ata_program {
        return Err(VerificationError::InvalidCreateATAInstruction);
    }

    // CreateATA has no data (instruction discriminator is in program)
    // Accounts: [payer, ata, owner, mint, system_program, token_program]
    if instruction.accounts.len() < 6 {
        return Err(VerificationError::InvalidCreateATAInstruction);
    }

    let owner_idx = instruction.accounts[2] as usize;
    let mint_idx = instruction.accounts[3] as usize;

    let owner = &message.account_keys[owner_idx];
    let mint = &message.account_keys[mint_idx];

    // Verify owner matches pay_to
    let pay_to: Pubkey = requirements
        .pay_to
        .parse()
        .map_err(|_| VerificationError::CreateATAIncorrectPayee)?;

    if owner != &pay_to {
        return Err(VerificationError::CreateATAIncorrectPayee);
    }

    // Verify mint matches asset
    let asset: Pubkey = requirements
        .asset
        .parse()
        .map_err(|_| VerificationError::CreateATAIncorrectAsset)?;

    if mint != &asset {
        return Err(VerificationError::CreateATAIncorrectAsset);
    }

    Ok(())
}

// Include comprehensive unit tests
#[cfg(test)]
#[path = "verifier_tests.rs"]
mod verifier_tests;
