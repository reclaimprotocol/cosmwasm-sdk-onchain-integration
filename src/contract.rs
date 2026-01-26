//! Reclaim Protocol CosmWasm Smart Contract
//!
//! This contract enables on-chain verification of Reclaim Protocol proofs.
//! It supports both vanilla CosmWasm and Secret Network deployments.

#[cfg(not(feature = "library"))]
#[cfg(feature = "vanilla")]
use cosmwasm_std::entry_point;

#[cfg(feature = "vanilla")]
use {
    crate::state_vanilla::CONFIG,
    cosmwasm_std::{to_json_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Timestamp, Uint128},
};

#[cfg(feature = "secret")]
use {
    crate::state_secret::CONFIG,
    secret_std::{entry_point, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Timestamp, Uint128},
};

use crate::event_builder::{add_epoch_event, add_instantiate_event, add_signer_event, add_verify_event};
use crate::msg::{ExecuteMsg, GetAllEpochResponse, GetEpochResponse, InstantiateMsg, ProofMsg, QueryMsg};
use crate::state::{Config, Epoch, Witness};
use crate::storage_adapter::{get_all_epoch_ids, load_epoch, save_epoch_new};
use crate::ContractError;
use sha2::{Digest, Sha256};

// cw2 is only available for vanilla CosmWasm, not Secret Network
#[cfg(feature = "vanilla")]
use cw2::set_contract_version;

/// Contract name for version tracking
#[cfg(feature = "vanilla")]
const CONTRACT_NAME: &str = "crates.io:reclaim-cosmwasm";
/// Contract version from Cargo.toml
#[cfg(feature = "vanilla")]
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Instantiates the contract with an owner address.
///
/// # Arguments
/// * `deps` - Mutable dependencies for storage access
/// * `_env` - Environment information (unused)
/// * `_info` - Message info (unused)
/// * `msg` - Instantiation message containing the owner address
///
/// # Returns
/// * `Ok(Response)` - On successful initialization
/// * `Err(ContractError)` - If address validation fails
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // Set contract version for migration support (vanilla only, cw2 not available on Secret)
    #[cfg(feature = "vanilla")]
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let addr = deps.api.addr_validate(&msg.owner)?;
    let config = Config {
        owner: addr.clone(),
        current_epoch: Uint128::zero(),
    };

    CONFIG.save(deps.storage, &config)?;

    let resp = Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("owner", addr.as_str());
    let resp = add_instantiate_event(resp, addr.as_str());

    Ok(resp)
}

/// Routes execute messages to appropriate handlers.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::VerifyProof(msg) => verify_proof(deps, msg, env),
        ExecuteMsg::AddEpoch {
            witness,
            minimum_witness,
        } => add_epoch(deps, env, witness, minimum_witness, info.sender),
    }
}

/// Generates a pseudo-random u32 from a hash at a given offset.
///
/// # Arguments
/// * `bytes` - The hash bytes to extract randomness from
/// * `offset` - The byte offset to start reading from
///
/// # Returns
/// A u32 value derived from 4 bytes of the hash
fn generate_random_seed(bytes: &[u8], offset: usize) -> u32 {
    let hash_slice = &bytes[offset..offset + 4];
    let mut seed = 0u32;
    for (i, &byte) in hash_slice.iter().enumerate() {
        seed |= u32::from(byte) << (i * 8);
    }
    seed
}

/// Fetches the expected witnesses for a claim based on deterministic selection.
///
/// Uses a hash of the identifier, epoch, and timestamp to deterministically
/// select witnesses from the epoch's witness list.
///
/// # Arguments
/// * `epoch` - The epoch configuration containing the witness list
/// * `identifier` - The claim identifier hash
/// * `timestamp` - The block timestamp
///
/// # Returns
/// A vector of selected witnesses
pub fn fetch_witness_for_claim(
    epoch: Epoch,
    identifier: String,
    timestamp: Timestamp,
) -> Vec<Witness> {
    let mut selected_witnesses = vec![];

    // Create a hash from identifier+epoch+minimum+timestamp
    let hash_str = format!(
        "{}\n{}\n{}\n{}",
        hex::encode(&identifier),
        epoch.minimum_witness_for_claim_creation,
        timestamp.nanos(),
        epoch.id
    );

    let mut hasher = Sha256::new();
    hasher.update(hash_str.as_bytes());
    let hash_result = hasher.finalize().to_vec();

    let witnesses_list = &epoch.witness;
    let witnesses_count = witnesses_list.len();
    let hash_len = hash_result.len();
    let mut byte_offset = 0;

    for _ in 0..epoch.minimum_witness_for_claim_creation.u128() {
        let random_seed = generate_random_seed(&hash_result, byte_offset) as usize;
        let witness_index = random_seed % witnesses_count;

        if let Some(witness) = witnesses_list.get(witness_index) {
            selected_witnesses.push(witness.clone());
        }

        byte_offset = (byte_offset + 4) % hash_len;
    }

    selected_witnesses
}

/// Verifies a proof submitted by a user.
///
/// This function:
/// 1. Loads the epoch configuration from storage
/// 2. Validates the claim hash matches the identifier
/// 3. Fetches expected witnesses for the claim
/// 4. Recovers signer addresses from the signatures
/// 5. Verifies all signers are expected witnesses
///
/// # Arguments
/// * `deps` - Mutable dependencies for storage access
/// * `msg` - The proof message containing claim and signatures
/// * `env` - Environment information including block time
///
/// # Returns
/// * `Ok(Response)` - On successful verification with signer events
/// * `Err(ContractError)` - If verification fails
pub fn verify_proof(deps: DepsMut, msg: ProofMsg, env: Env) -> Result<Response, ContractError> {
    // Extract values we need before moving signedClaim
    let claim_epoch = msg.proof.signedClaim.claim.epoch;
    let claim_identifier = msg.proof.signedClaim.claim.identifier.clone();
    let epoch_id: u128 = claim_epoch.into();

    // Load epoch using the storage abstraction
    let epoch = load_epoch(deps.storage, epoch_id)?;

    // Hash the claims and verify with identifier hash
    let hashed = msg.proof.claimInfo.hash();
    if claim_identifier != hashed {
        return Err(ContractError::HashMismatchErr {});
    }

    // Fetch expected witnesses for this claim
    let expected_witness = fetch_witness_for_claim(epoch, claim_identifier, env.block.time);

    let expected_witness_addresses = Witness::get_addresses(expected_witness);

    // Recover witness addresses from signatures
    let signed_witness = msg.proof.signedClaim.recover_signers_of_signed_claim(deps)?;

    // Verify the minimum witness requirement is satisfied
    if expected_witness_addresses.len() != signed_witness.len() {
        return Err(ContractError::WitnessMismatchErr {});
    }

    // Build response with signer events
    let mut resp = Response::new().add_attribute("action", "verify_proof");

    // Verify each signer is an expected witness
    for signed in &signed_witness {
        resp = add_signer_event(resp, signed);
        if !expected_witness_addresses.contains(signed) {
            return Err(ContractError::SignatureErr {});
        }
    }

    // Add success event
    resp = add_verify_event(resp, claim_epoch);

    Ok(resp)
}

/// Adds a new epoch with the specified witnesses. Only callable by the contract owner.
///
/// # Arguments
/// * `deps` - Mutable dependencies for storage access
/// * `env` - Environment information including block time
/// * `witness` - List of authorized witnesses for this epoch
/// * `minimum_witness` - Minimum number of signatures required
/// * `sender` - Address of the message sender
///
/// # Returns
/// * `Ok(Response)` - On successful epoch creation
/// * `Err(ContractError)` - If unauthorized or validation fails
pub fn add_epoch(
    deps: DepsMut,
    env: Env,
    witness: Vec<Witness>,
    minimum_witness: Uint128,
    sender: Addr,
) -> Result<Response, ContractError> {
    // Load current config
    let mut config = CONFIG.load(deps.storage)?;

    // Check if sender is owner
    if config.owner != sender {
        return Err(ContractError::Unauthorized {});
    }

    // Validate witness configuration
    if witness.is_empty() {
        return Err(ContractError::InvalidWitnessConfig {
            reason: "At least one witness is required".to_string(),
        });
    }

    if minimum_witness.is_zero() {
        return Err(ContractError::InvalidWitnessConfig {
            reason: "Minimum witness must be greater than zero".to_string(),
        });
    }

    if minimum_witness > Uint128::from(witness.len() as u128) {
        return Err(ContractError::InvalidWitnessConfig {
            reason: "Minimum witness cannot exceed total witness count".to_string(),
        });
    }

    // Validate all witness addresses
    for wit in &witness {
        Witness::validate_eth_address(&wit.address)?;
        if wit.host.is_empty() {
            return Err(ContractError::InvalidWitnessConfig {
                reason: "Witness host cannot be empty".to_string(),
            });
        }
    }

    // Increment epoch number
    let new_epoch = config.current_epoch + Uint128::one();

    // Calculate epoch end timestamp
    let timestamp_end = calculate_epoch_end(&env);

    // Create the new epoch
    let epoch = Epoch {
        id: new_epoch,
        witness,
        timestamp_start: env.block.time.nanos(),
        timestamp_end,
        minimum_witness_for_claim_creation: minimum_witness,
    };

    // Save the epoch (fails if already exists)
    save_epoch_new(deps.storage, new_epoch.u128(), &epoch)?;

    // Update config with new epoch number
    config.current_epoch = new_epoch;
    CONFIG.save(deps.storage, &config)?;

    // Build response with event
    let resp = Response::new()
        .add_attribute("action", "add_epoch")
        .add_attribute("epoch_id", new_epoch.to_string());
    let resp = add_epoch_event(resp, new_epoch.u128(), minimum_witness.u128());

    Ok(resp)
}

/// Calculates the epoch end timestamp (24 hours from now).
#[cfg(feature = "vanilla")]
fn calculate_epoch_end(env: &Env) -> u64 {
    env.block.time.plus_days(1).nanos()
}

#[cfg(feature = "secret")]
fn calculate_epoch_end(env: &Env) -> u64 {
    env.block.time.plus_seconds(86400).nanos()
}

/// Routes query messages to appropriate handlers.
#[cfg(feature = "vanilla")]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetEpoch { id } => to_json_binary(&query_epoch_id(deps, id)?),
        QueryMsg::GetAllEpoch {} => to_json_binary(&query_all_epoch_ids(deps)?),
    }
}

#[cfg(feature = "secret")]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetEpoch { id } => to_binary(&query_epoch_id(deps, id)?),
        QueryMsg::GetAllEpoch {} => to_binary(&query_all_epoch_ids(deps)?),
    }
}

/// Queries all epoch IDs.
fn query_all_epoch_ids(deps: Deps) -> StdResult<GetAllEpochResponse> {
    Ok(GetAllEpochResponse {
        ids: get_all_epoch_ids(deps.storage)?,
    })
}

/// Queries a specific epoch by ID.
fn query_epoch_id(deps: Deps, id: u128) -> StdResult<GetEpochResponse> {
    let epoch = load_epoch(deps.storage, id).map_err(|e| StdError::generic_err(e.to_string()))?;
    Ok(GetEpochResponse { epoch })
}

#[cfg(test)]
mod tests {}
