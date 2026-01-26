//! Event builder abstraction for cross-feature compatibility.
//!
//! This module provides helper functions for building events
//! that work across both "vanilla" and "secret" features.
//! The main difference is that Secret Network uses `add_attribute_plaintext`
//! while vanilla CosmWasm uses `add_attribute`.

#[cfg(feature = "vanilla")]
use cosmwasm_std::{Event, Response};

#[cfg(feature = "secret")]
use secret_std::{Event, Response};

/// Adds a signer event to the response.
///
/// # Arguments
/// * `response` - The response to add the event to
/// * `signer` - The signer address to include in the event
///
/// # Returns
/// The response with the signer event added
#[cfg(feature = "vanilla")]
pub fn add_signer_event(response: Response, signer: &str) -> Response {
    let event = Event::new("signer").add_attribute("address", signer);
    response.add_event(event)
}

#[cfg(feature = "secret")]
pub fn add_signer_event(response: Response, signer: &str) -> Response {
    let event = Event::new("signer").add_attribute_plaintext("address", signer);
    response.add_event(event)
}

/// Adds an epoch creation event to the response.
///
/// # Arguments
/// * `response` - The response to add the event to
/// * `epoch_id` - The ID of the created epoch
/// * `minimum_witness` - The minimum witness count for the epoch
///
/// # Returns
/// The response with the epoch event added
#[cfg(feature = "vanilla")]
pub fn add_epoch_event(response: Response, epoch_id: u128, minimum_witness: u128) -> Response {
    let event = Event::new("add_epoch")
        .add_attribute("epoch_id", epoch_id.to_string())
        .add_attribute("minimum_witness", minimum_witness.to_string());
    response.add_event(event)
}

#[cfg(feature = "secret")]
pub fn add_epoch_event(response: Response, epoch_id: u128, minimum_witness: u128) -> Response {
    let event = Event::new("add_epoch")
        .add_attribute_plaintext("epoch_id", epoch_id.to_string())
        .add_attribute_plaintext("minimum_witness", minimum_witness.to_string());
    response.add_event(event)
}

/// Adds an instantiate event to the response.
///
/// # Arguments
/// * `response` - The response to add the event to
/// * `owner` - The contract owner address
///
/// # Returns
/// The response with the instantiate event added
#[cfg(feature = "vanilla")]
pub fn add_instantiate_event(response: Response, owner: &str) -> Response {
    let event = Event::new("instantiate").add_attribute("owner", owner);
    response.add_event(event)
}

#[cfg(feature = "secret")]
pub fn add_instantiate_event(response: Response, owner: &str) -> Response {
    let event = Event::new("instantiate").add_attribute_plaintext("owner", owner);
    response.add_event(event)
}

/// Adds a verify proof success event to the response.
///
/// # Arguments
/// * `response` - The response to add the event to
/// * `epoch_id` - The epoch ID that was verified against
///
/// # Returns
/// The response with the verify event added
#[cfg(feature = "vanilla")]
pub fn add_verify_event(response: Response, epoch_id: u64) -> Response {
    let event = Event::new("verify_proof")
        .add_attribute("result", "success")
        .add_attribute("epoch_id", epoch_id.to_string());
    response.add_event(event)
}

#[cfg(feature = "secret")]
pub fn add_verify_event(response: Response, epoch_id: u64) -> Response {
    let event = Event::new("verify_proof")
        .add_attribute_plaintext("result", "success")
        .add_attribute_plaintext("epoch_id", epoch_id.to_string());
    response.add_event(event)
}
