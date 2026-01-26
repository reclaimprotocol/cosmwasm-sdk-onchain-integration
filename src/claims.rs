#![allow(non_snake_case)]

use crate::ContractError;
mod identity_digest;
#[cfg(feature = "vanilla")]
use cosmwasm_std::DepsMut;
use k256::ecdsa::{RecoveryId, Signature, VerifyingKey};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};

#[cfg(feature = "secret")]
use secret_std::DepsMut;

/// Prepends "0x" to a hex string.
///
/// # Arguments
/// * `content` - The hex string to prepend to
///
/// # Returns
/// The string with "0x" prepended
pub fn append_0x(content: &str) -> String {
    format!("0x{}", content)
}

/// Computes the Ethereum signed message hash (EIP-191).
///
/// # Arguments
/// * `message` - The message to hash
///
/// # Returns
/// The keccak256 hash of the Ethereum signed message
pub fn keccak256(message: &str) -> Vec<u8> {
    let message: &[u8] = message.as_ref();

    let mut eth_message = format!("\x19Ethereum Signed Message:\n{}", message.len()).into_bytes();
    eth_message.extend_from_slice(message);
    let mut hasher = Keccak256::new();
    hasher.update(&eth_message);

    hasher.finalize().to_vec()
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ClaimInfo {
    pub provider: String,
    pub parameters: String,
    pub context: String,
}

impl ClaimInfo {
    pub fn hash(&self) -> String {
        let mut hasher = Keccak256::new();
        let hash_str = format!(
            "{}\n{}\n{}",
            &self.provider, &self.parameters, &self.context
        );
        hasher.update(&hash_str);

        let hash = hasher.finalize().to_vec();
        append_0x(hex::encode(hash).as_str())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CompleteClaimData {
    pub identifier: String,
    pub owner: String,
    pub epoch: u64,
    pub timestampS: u64,
}

impl CompleteClaimData {
    pub fn serialise(&self) -> String {
        format!(
            "{}\n{}\n{}\n{}",
            &self.identifier,
            &self.owner.to_string(),
            &self.timestampS.to_string(),
            &self.epoch.to_string()
        )
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct SignedClaim {
    pub claim: CompleteClaimData,
    pub signatures: Vec<String>,
}

impl SignedClaim {
    /// Recovers the Ethereum addresses of all signers from their signatures.
    ///
    /// # Arguments
    /// * `_deps` - Dependencies (unused, kept for API compatibility)
    ///
    /// # Returns
    /// * `Ok(Vec<String>)` - List of recovered Ethereum addresses with "0x" prefix
    /// * `Err(ContractError)` - If any signature is invalid or recovery fails
    pub fn recover_signers_of_signed_claim(
        self,
        _deps: DepsMut,
    ) -> Result<Vec<String>, ContractError> {
        let serialised_claim = self.claim.serialise();
        let message_hash = keccak256(serialised_claim.as_str());

        let mut expected = Vec::with_capacity(self.signatures.len());

        for complete_signature in &self.signatures {
            let recovered_address = Self::recover_single_signer(complete_signature, &message_hash)?;
            expected.push(recovered_address);
        }

        Ok(expected)
    }

    /// Recovers a single signer's Ethereum address from their signature.
    ///
    /// The signature is expected to be a hex string with "0x" prefix,
    /// containing r (32 bytes) + s (32 bytes) + v (1 byte) = 65 bytes total.
    ///
    /// # Arguments
    /// * `complete_signature` - The hex-encoded signature with "0x" prefix
    /// * `message_hash` - The keccak256 hash of the signed message
    ///
    /// # Returns
    /// * `Ok(String)` - The recovered Ethereum address with "0x" prefix
    /// * `Err(ContractError)` - If the signature is invalid or recovery fails
    fn recover_single_signer(
        complete_signature: &str,
        message_hash: &[u8],
    ) -> Result<String, ContractError> {
        use crate::claims::identity_digest::Identity256;
        use digest::Update;

        // Validate and strip "0x" prefix
        let sig_without_prefix = complete_signature
            .strip_prefix("0x")
            .ok_or_else(|| ContractError::InvalidSignatureFormat {
                reason: "Missing 0x prefix".to_string(),
            })?;

        // Validate minimum length: 64 bytes for r+s = 128 hex chars + 2 for v
        if sig_without_prefix.len() < 130 {
            return Err(ContractError::InvalidSignatureFormat {
                reason: format!(
                    "Signature too short: expected at least 130 hex chars, got {}",
                    sig_without_prefix.len()
                ),
            });
        }

        // Extract recovery parameter (last byte) and r+s components
        let sig_len = sig_without_prefix.len();
        let rec_param_hex = &sig_without_prefix[sig_len - 2..];
        let r_s_hex = &sig_without_prefix[..sig_len - 2];

        // Decode recovery parameter
        let rec_dec = hex::decode(rec_param_hex).map_err(|e| {
            ContractError::HexDecodeError(format!("Recovery param decode failed: {}", e))
        })?;

        let rec_byte = *rec_dec.first().ok_or_else(|| ContractError::InvalidSignatureFormat {
            reason: "Empty recovery parameter".to_string(),
        })?;

        // Normalize recovery parameter (Ethereum uses 27/28, we need 0/1)
        if rec_byte < 27 || rec_byte > 28 {
            return Err(ContractError::InvalidRecoveryParam { value: rec_byte });
        }
        let rec_norm = rec_byte - 27;

        // Decode r and s components
        let r_s = hex::decode(r_s_hex).map_err(|e| {
            ContractError::HexDecodeError(format!("Signature r,s decode failed: {}", e))
        })?;

        // Create recovery ID
        let id = match rec_norm {
            0 => RecoveryId::new(false, false),
            1 => RecoveryId::new(true, false),
            _ => return Err(ContractError::SignatureErr {}),
        };

        // Parse signature bytes
        let signature = Signature::from_bytes(r_s.as_slice().into())
            .map_err(|_| ContractError::SignatureConversionError {})?;

        let message_digest = Identity256::new().chain(message_hash);

        // Recover the public key
        let verkey = VerifyingKey::recover_from_digest(message_digest, &signature, id)
            .map_err(|_| ContractError::KeyRecoveryError {})?;

        // Convert public key to Ethereum address
        let key: Vec<u8> = verkey.to_encoded_point(false).as_bytes().into();
        let hasher = Keccak256::new_with_prefix(&key[1..]);
        let hash = hasher.finalize().to_vec();

        // Validate hash length and extract address (last 20 bytes)
        if hash.len() < 32 {
            return Err(ContractError::InvalidHashLength {
                expected: 32,
                actual: hash.len(),
            });
        }

        let address_bytes = &hash[12..];
        Ok(append_0x(&hex::encode(address_bytes)))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Proof {
    pub claimInfo: ClaimInfo,
    pub signedClaim: SignedClaim,
}
