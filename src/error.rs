#[cfg(feature = "vanilla")]
use cosmwasm_std::StdError;
#[cfg(feature = "secret")]
use secret_std::StdError;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Epoch ID already exists")]
    AlreadyExists {},

    #[error("Key recovery error")]
    PubKeyErr {},

    #[error("Signature not appropriate")]
    SignatureErr {},

    #[error("Hash mismatch")]
    HashMismatchErr {},

    #[error("Not enough witnesses")]
    WitnessMismatchErr {},

    #[error("Epoch not found")]
    NotFoundErr {},

    // New error types for proper error handling
    #[error("Invalid signature format: {reason}")]
    InvalidSignatureFormat { reason: String },

    #[error("Hex decode error: {0}")]
    HexDecodeError(String),

    #[error("Invalid recovery parameter: {value}")]
    InvalidRecoveryParam { value: u8 },

    #[error("Signature byte conversion failed")]
    SignatureConversionError {},

    #[error("Public key recovery failed")]
    KeyRecoveryError {},

    #[error("Invalid hash length: expected at least {expected} bytes, got {actual}")]
    InvalidHashLength { expected: usize, actual: usize },

    #[error("Invalid address length: expected {expected} hex chars, got {actual}")]
    InvalidAddressLength { expected: usize, actual: usize },

    #[error("Invalid address format: {reason}")]
    InvalidAddressFormat { reason: String },

    #[error("Invalid witness configuration: {reason}")]
    InvalidWitnessConfig { reason: String },
}
