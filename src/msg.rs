use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[cfg(feature = "vanilla")]
use cosmwasm_std::Uint128;

#[cfg(feature = "secret")]
use secret_std::Uint128;

use crate::{
    claims::Proof,
    state::{Epoch, Witness},
};

/// Message to instantiate the contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    /// The address that will own the contract and can add epochs
    pub owner: String,
}

/// Execute messages for contract interactions.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Verify a proof on-chain
    VerifyProof(ProofMsg),
    /// Add a new epoch with witnesses (owner only)
    AddEpoch {
        /// List of authorized witnesses for this epoch
        witness: Vec<Witness>,
        /// Minimum number of witness signatures required
        minimum_witness: Uint128,
    },
}

/// Query messages for reading contract state.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Get all epoch IDs (returns empty on Secret Network)
    GetAllEpoch {},
    /// Get a specific epoch by ID
    GetEpoch { id: u128 },
}

/// Response for GetAllEpoch query.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct GetAllEpochResponse {
    /// List of all epoch IDs
    pub ids: Vec<u128>,
}

/// Response for GetEpoch query.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct GetEpochResponse {
    /// The requested epoch data
    pub epoch: Epoch,
}

/// Message containing a proof to verify.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ProofMsg {
    /// The proof to verify
    pub proof: Proof,
}
