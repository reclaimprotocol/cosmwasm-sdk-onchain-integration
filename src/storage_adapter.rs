//! Storage abstraction layer for cross-feature compatibility.
//!
//! This module provides a unified interface for storage operations
//! that works across both "vanilla" and "secret" features.

use crate::state::Epoch;
use crate::ContractError;

#[cfg(feature = "vanilla")]
use cosmwasm_std::{StdResult, Storage};

#[cfg(feature = "secret")]
use secret_std::{StdResult, Storage};

/// Loads an epoch by ID from storage.
///
/// # Arguments
/// * `storage` - Reference to the contract storage
/// * `id` - The epoch ID to load
///
/// # Returns
/// * `Ok(Epoch)` - The epoch if found
/// * `Err(ContractError::NotFoundErr)` - If epoch doesn't exist
#[cfg(feature = "vanilla")]
pub fn load_epoch(storage: &dyn Storage, id: u128) -> Result<Epoch, ContractError> {
    use crate::state_vanilla::EPOCHS;
    EPOCHS.load(storage, id).map_err(|_| ContractError::NotFoundErr {})
}

#[cfg(feature = "secret")]
pub fn load_epoch(storage: &dyn Storage, id: u128) -> Result<Epoch, ContractError> {
    use crate::state_secret::EPOCHS;
    EPOCHS.get(storage, &id).ok_or(ContractError::NotFoundErr {})
}

/// Checks if an epoch exists in storage.
///
/// # Arguments
/// * `storage` - Reference to the contract storage
/// * `id` - The epoch ID to check
///
/// # Returns
/// `true` if the epoch exists, `false` otherwise
#[cfg(feature = "vanilla")]
pub fn epoch_exists(storage: &dyn Storage, id: u128) -> bool {
    use crate::state_vanilla::EPOCHS;
    EPOCHS.may_load(storage, id).ok().flatten().is_some()
}

#[cfg(feature = "secret")]
pub fn epoch_exists(storage: &dyn Storage, id: u128) -> bool {
    use crate::state_secret::EPOCHS;
    EPOCHS.get(storage, &id).is_some()
}

/// Saves a new epoch to storage. Fails if the epoch already exists.
///
/// # Arguments
/// * `storage` - Mutable reference to the contract storage
/// * `id` - The epoch ID to save
/// * `epoch` - The epoch data to save
///
/// # Returns
/// * `Ok(())` - If the epoch was saved successfully
/// * `Err(ContractError::AlreadyExists)` - If an epoch with this ID already exists
#[cfg(feature = "vanilla")]
pub fn save_epoch_new(
    storage: &mut dyn Storage,
    id: u128,
    epoch: &Epoch,
) -> Result<(), ContractError> {
    use crate::state_vanilla::EPOCHS;
    EPOCHS.update(storage, id, |existing| match existing {
        None => Ok(epoch.clone()),
        Some(..) => Err(ContractError::AlreadyExists {}),
    })?;
    Ok(())
}

#[cfg(feature = "secret")]
pub fn save_epoch_new(
    storage: &mut dyn Storage,
    id: u128,
    epoch: &Epoch,
) -> Result<(), ContractError> {
    use crate::state_secret::EPOCHS;
    if epoch_exists(storage, id) {
        return Err(ContractError::AlreadyExists {});
    }
    EPOCHS
        .insert(storage, &id, epoch)
        .map_err(|e| ContractError::Std(e))?;
    Ok(())
}

/// Gets all epoch IDs from storage.
///
/// # Arguments
/// * `storage` - Reference to the contract storage
///
/// # Returns
/// A vector of all epoch IDs. Returns empty vec for Secret Network
/// as it doesn't support key iteration.
#[cfg(feature = "vanilla")]
pub fn get_all_epoch_ids(storage: &dyn Storage) -> StdResult<Vec<u128>> {
    crate::state_vanilla::get_all_epochs(storage)
}

#[cfg(feature = "secret")]
pub fn get_all_epoch_ids(_storage: &dyn Storage) -> StdResult<Vec<u128>> {
    // Secret Network doesn't support iteration over keys
    Ok(vec![])
}
