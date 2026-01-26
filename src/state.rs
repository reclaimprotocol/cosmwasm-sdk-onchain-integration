use crate::ContractError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[cfg(feature = "vanilla")]
use cosmwasm_std::{Addr, Uint128};

#[cfg(feature = "secret")]
use secret_std::{Addr, Uint128};

/// Contract configuration stored on-chain.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    /// The owner address who can add epochs
    pub owner: Addr,
    /// The current epoch number
    pub current_epoch: Uint128,
}

/// Represents a witness node that can sign claims.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Witness {
    /// The Ethereum address of the witness (hex string with 0x prefix)
    pub address: String,
    /// The host URL of the witness node
    pub host: String,
}

impl Witness {
    /// Extracts all addresses from a list of witnesses.
    ///
    /// # Arguments
    /// * `witnesses` - The list of witnesses
    ///
    /// # Returns
    /// A vector of address strings
    pub fn get_addresses(witnesses: Vec<Witness>) -> Vec<String> {
        witnesses.into_iter().map(|w| w.address).collect()
    }

    /// Validates that a witness has a properly formatted Ethereum address.
    ///
    /// # Arguments
    /// * `address` - The address string to validate
    ///
    /// # Returns
    /// * `Ok(())` - If the address is valid
    /// * `Err(ContractError)` - If the address format is invalid
    pub fn validate_eth_address(address: &str) -> Result<(), ContractError> {
        let without_prefix = address.strip_prefix("0x").unwrap_or(address);

        // Ethereum addresses are 20 bytes = 40 hex characters
        if without_prefix.len() != 40 {
            return Err(ContractError::InvalidAddressLength {
                expected: 40,
                actual: without_prefix.len(),
            });
        }

        // Validate all characters are hex
        if !without_prefix.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ContractError::InvalidAddressFormat {
                reason: "Address contains non-hex characters".to_string(),
            });
        }

        Ok(())
    }
}

/// Represents an epoch configuration with authorized witnesses.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Epoch {
    /// Unique identifier for this epoch
    pub id: Uint128,
    /// Unix timestamp (nanoseconds) when this epoch started
    pub timestamp_start: u64,
    /// Unix timestamp (nanoseconds) when this epoch ends
    pub timestamp_end: u64,
    /// Minimum number of witness signatures required for claim verification
    pub minimum_witness_for_claim_creation: Uint128,
    /// List of authorized witnesses for this epoch
    pub witness: Vec<Witness>,
}
