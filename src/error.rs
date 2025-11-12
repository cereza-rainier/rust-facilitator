use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Internal server error")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::Config(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}

// Verification errors (will be expanded in Day 4)
#[derive(Debug, Error)]
pub enum VerificationError {
    #[error("unsupported_scheme")]
    UnsupportedScheme,

    #[error("invalid_network")]
    InvalidNetwork,

    #[error("invalid_exact_svm_payload_transaction_instructions_length")]
    InvalidInstructionCount,

    #[error("invalid_exact_svm_payload_transaction_instructions_compute_limit_instruction")]
    InvalidComputeLimitInstruction,

    #[error("invalid_exact_svm_payload_transaction_instructions_compute_price_instruction")]
    InvalidComputePriceInstruction,

    #[error("invalid_exact_svm_payload_transaction_instructions_compute_price_instruction_too_high")]
    ComputePriceTooHigh,

    #[error("invalid_exact_svm_payload_transaction_fee_payer_included_in_instruction_accounts")]
    FeePayerInInstructionAccounts,

    #[error("invalid_exact_svm_payload_transaction_fee_payer_transferring_funds")]
    FeePayerTransferringFunds,

    #[error("invalid_exact_svm_payload_transaction_amount_mismatch")]
    AmountMismatch,

    #[error("invalid_exact_svm_payload_transaction_create_ata_instruction")]
    InvalidCreateATAInstruction,

    #[error("invalid_exact_svm_payload_transaction_create_ata_instruction_incorrect_payee")]
    CreateATAIncorrectPayee,

    #[error("invalid_exact_svm_payload_transaction_create_ata_instruction_incorrect_asset")]
    CreateATAIncorrectAsset,

    #[error("invalid_exact_svm_payload_transaction_transfer_to_incorrect_ata")]
    TransferToIncorrectATA,

    #[error("invalid_exact_svm_payload_transaction_sender_ata_not_found")]
    SenderATANotFound,

    #[error("invalid_exact_svm_payload_transaction_receiver_ata_not_found")]
    ReceiverATANotFound,

    #[error("invalid_exact_svm_payload_transaction_not_a_transfer_instruction")]
    NotATransferInstruction,

    #[error("unexpected_verify_error")]
    UnexpectedError(#[from] anyhow::Error),
}

impl VerificationError {
    pub fn as_str(&self) -> &str {
        match self {
            Self::UnsupportedScheme => "unsupported_scheme",
            Self::InvalidNetwork => "invalid_network",
            Self::InvalidInstructionCount => "invalid_exact_svm_payload_transaction_instructions_length",
            Self::InvalidComputeLimitInstruction => "invalid_exact_svm_payload_transaction_instructions_compute_limit_instruction",
            Self::InvalidComputePriceInstruction => "invalid_exact_svm_payload_transaction_instructions_compute_price_instruction",
            Self::ComputePriceTooHigh => "invalid_exact_svm_payload_transaction_instructions_compute_price_instruction_too_high",
            Self::FeePayerInInstructionAccounts => "invalid_exact_svm_payload_transaction_fee_payer_included_in_instruction_accounts",
            Self::FeePayerTransferringFunds => "invalid_exact_svm_payload_transaction_fee_payer_transferring_funds",
            Self::AmountMismatch => "invalid_exact_svm_payload_transaction_amount_mismatch",
            Self::InvalidCreateATAInstruction => "invalid_exact_svm_payload_transaction_create_ata_instruction",
            Self::CreateATAIncorrectPayee => "invalid_exact_svm_payload_transaction_create_ata_instruction_incorrect_payee",
            Self::CreateATAIncorrectAsset => "invalid_exact_svm_payload_transaction_create_ata_instruction_incorrect_asset",
            Self::TransferToIncorrectATA => "invalid_exact_svm_payload_transaction_transfer_to_incorrect_ata",
            Self::SenderATANotFound => "invalid_exact_svm_payload_transaction_sender_ata_not_found",
            Self::ReceiverATANotFound => "invalid_exact_svm_payload_transaction_receiver_ata_not_found",
            Self::NotATransferInstruction => "invalid_exact_svm_payload_transaction_not_a_transfer_instruction",
            Self::UnexpectedError(_) => "unexpected_verify_error",
        }
    }
}

