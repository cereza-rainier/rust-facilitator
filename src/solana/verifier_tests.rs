// Comprehensive unit tests for verification logic

#[cfg(test)]
mod tests {
    use super::super::*;
    use solana_sdk::{
        instruction::{AccountMeta, CompiledInstruction, Instruction},
        message::Message,
        pubkey::Pubkey,
        transaction::Transaction,
    };

    // Helper to create a mock message with compute budget program
    fn create_mock_message() -> Message {
        Message::new(
            &[Instruction::new_with_bytes(
                compute_budget_program_id(),
                &[],
                vec![],
            )],
            None,
        )
    }

    #[test]
    fn test_verify_instruction_count_valid_3() {
        // Create transaction with 3 instructions
        let instructions = vec![
            Instruction::new_with_bytes(Pubkey::new_unique(), &[], vec![]),
            Instruction::new_with_bytes(Pubkey::new_unique(), &[], vec![]),
            Instruction::new_with_bytes(Pubkey::new_unique(), &[], vec![]),
        ];
        let message = Message::new(&instructions, None);
        let tx = Transaction::new_unsigned(message);

        let result = verify_instruction_count(&tx);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false); // No CreateATA
    }

    #[test]
    fn test_verify_instruction_count_valid_4() {
        // Create transaction with 4 instructions
        let instructions = vec![
            Instruction::new_with_bytes(Pubkey::new_unique(), &[], vec![]),
            Instruction::new_with_bytes(Pubkey::new_unique(), &[], vec![]),
            Instruction::new_with_bytes(Pubkey::new_unique(), &[], vec![]),
            Instruction::new_with_bytes(Pubkey::new_unique(), &[], vec![]),
        ];
        let message = Message::new(&instructions, None);
        let tx = Transaction::new_unsigned(message);

        let result = verify_instruction_count(&tx);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true); // Has CreateATA
    }

    #[test]
    fn test_verify_instruction_count_invalid_2() {
        // Create transaction with only 2 instructions (invalid)
        let instructions = vec![
            Instruction::new_with_bytes(Pubkey::new_unique(), &[], vec![]),
            Instruction::new_with_bytes(Pubkey::new_unique(), &[], vec![]),
        ];
        let message = Message::new(&instructions, None);
        let tx = Transaction::new_unsigned(message);

        let result = verify_instruction_count(&tx);
        assert!(matches!(result, Err(VerificationError::InvalidInstructionCount)));
    }

    #[test]
    fn test_verify_instruction_count_invalid_5() {
        // Create transaction with 5 instructions (invalid)
        let instructions = vec![
            Instruction::new_with_bytes(Pubkey::new_unique(), &[], vec![]),
            Instruction::new_with_bytes(Pubkey::new_unique(), &[], vec![]),
            Instruction::new_with_bytes(Pubkey::new_unique(), &[], vec![]),
            Instruction::new_with_bytes(Pubkey::new_unique(), &[], vec![]),
            Instruction::new_with_bytes(Pubkey::new_unique(), &[], vec![]),
        ];
        let message = Message::new(&instructions, None);
        let tx = Transaction::new_unsigned(message);

        let result = verify_instruction_count(&tx);
        assert!(matches!(result, Err(VerificationError::InvalidInstructionCount)));
    }

    #[test]
    fn test_compute_price_at_exact_limit() {
        // Create instruction with price exactly at 5 lamports (5_000_000 micro-lamports)
        let mut data = vec![3u8]; // SetComputeUnitPrice discriminator
        data.extend_from_slice(&5_000_000u64.to_le_bytes());

        let instruction = CompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data,
        };

        let message = create_mock_message();
        let result = verify_compute_price_instruction(&instruction, &message);
        assert!(result.is_ok());
    }

    #[test]
    fn test_compute_price_too_high() {
        // Create instruction with price over 5 lamports
        let mut data = vec![3u8]; // SetComputeUnitPrice discriminator
        data.extend_from_slice(&5_000_001u64.to_le_bytes());

        let instruction = CompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data,
        };

        let message = create_mock_message();
        let result = verify_compute_price_instruction(&instruction, &message);
        assert!(matches!(result, Err(VerificationError::ComputePriceTooHigh)));
    }

    #[test]
    fn test_compute_price_way_too_high() {
        // Create instruction with absurdly high price
        let mut data = vec![3u8];
        data.extend_from_slice(&50_000_000u64.to_le_bytes());

        let instruction = CompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data,
        };

        let message = create_mock_message();
        let result = verify_compute_price_instruction(&instruction, &message);
        assert!(matches!(result, Err(VerificationError::ComputePriceTooHigh)));
    }

    #[test]
    fn test_compute_price_zero() {
        // Zero price should be valid
        let mut data = vec![3u8];
        data.extend_from_slice(&0u64.to_le_bytes());

        let instruction = CompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data,
        };

        let message = create_mock_message();
        let result = verify_compute_price_instruction(&instruction, &message);
        assert!(result.is_ok());
    }

    #[test]
    fn test_compute_price_wrong_discriminator() {
        // Wrong discriminator should fail
        let mut data = vec![99u8]; // Wrong discriminator
        data.extend_from_slice(&1_000_000u64.to_le_bytes());

        let instruction = CompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data,
        };

        let message = create_mock_message();
        let result = verify_compute_price_instruction(&instruction, &message);
        assert!(matches!(result, Err(VerificationError::InvalidComputePriceInstruction)));
    }

    #[test]
    fn test_compute_limit_wrong_discriminator() {
        // Wrong discriminator should fail
        let mut data = vec![99u8]; // Wrong discriminator

        let instruction = CompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data,
        };

        let message = create_mock_message();
        let result = verify_compute_limit_instruction(&instruction, &message);
        assert!(matches!(result, Err(VerificationError::InvalidComputeLimitInstruction)));
    }

    #[test]
    fn test_compute_limit_valid() {
        // Correct discriminator (2 = SetComputeUnitLimit)
        let data = vec![2u8, 0, 0, 0, 1, 0, 0, 0]; // discriminator + limit

        let instruction = CompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data,
        };

        let message = create_mock_message();
        let result = verify_compute_limit_instruction(&instruction, &message);
        assert!(result.is_ok());
    }

    #[test]
    fn test_fee_payer_safety_valid() {
        // Create transaction where fee payer is NOT in instruction accounts
        let fee_payer = Pubkey::new_unique();
        let other_account = Pubkey::new_unique();

        let instructions = vec![
            Instruction {
                program_id: Pubkey::new_unique(),
                accounts: vec![AccountMeta::new(other_account, false)],
                data: vec![],
            },
        ];

        let message = Message::new(&instructions, Some(&fee_payer));
        let tx = Transaction::new_unsigned(message);

        let result = verify_fee_payer_safety(&tx, &fee_payer);
        assert!(result.is_ok());
    }

    #[test]
    fn test_fee_payer_safety_invalid() {
        // Create transaction where fee payer IS in instruction accounts (should fail)
        let fee_payer = Pubkey::new_unique();

        // Create message with fee payer in account keys
        let instructions = vec![
            Instruction {
                program_id: Pubkey::new_unique(),
                accounts: vec![AccountMeta::new(fee_payer, true)], // Fee payer in accounts!
                data: vec![],
            },
        ];

        let message = Message::new(&instructions, Some(&fee_payer));
        let tx = Transaction::new_unsigned(message);

        let result = verify_fee_payer_safety(&tx, &fee_payer);
        assert!(matches!(result, Err(VerificationError::FeePayerInInstructionAccounts)));
    }

    #[test]
    fn test_fee_payer_safety_multiple_instructions() {
        // Fee payer in second instruction should also fail
        let fee_payer = Pubkey::new_unique();
        let other_account = Pubkey::new_unique();

        let instructions = vec![
            Instruction {
                program_id: Pubkey::new_unique(),
                accounts: vec![AccountMeta::new(other_account, false)],
                data: vec![],
            },
            Instruction {
                program_id: Pubkey::new_unique(),
                accounts: vec![AccountMeta::new(fee_payer, true)], // Fee payer in second instruction
                data: vec![],
            },
        ];

        let message = Message::new(&instructions, Some(&fee_payer));
        let tx = Transaction::new_unsigned(message);

        let result = verify_fee_payer_safety(&tx, &fee_payer);
        assert!(matches!(result, Err(VerificationError::FeePayerInInstructionAccounts)));
    }

    #[test]
    fn test_compute_budget_program_id() {
        let id = compute_budget_program_id();
        // Should be the known ComputeBudget program ID
        assert_eq!(id.to_string(), "ComputeBudget111111111111111111111111111111");
    }

    #[test]
    fn test_spl_token_program_ids() {
        let token_id = spl_token_program_id();
        let token_2022_id = spl_token_2022_program_id();

        // Should be different IDs
        assert_ne!(token_id, token_2022_id);

        // Should be valid Solana program IDs
        assert_ne!(token_id, Pubkey::default());
        assert_ne!(token_2022_id, Pubkey::default());
    }
}

