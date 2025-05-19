//! Ethereum Merkle proof library.

use anyhow::{Context, Result};
use num_bigint::BigUint;
use types::EthereumAccount;

use crate::timewave_rlp;

mod tests;
pub mod types;

// Keccak-256 hash function implementation.
//
// This module provides functionality for computing Keccak-256 hashes, which is the
// hash function used in Ethereum. It supports both standard and SP1-specific
// implementations through feature flags.

use tiny_keccak::{Hasher, Keccak};

/// Computes the Keccak-256 hash of the input bytes.
///
/// This function implements the Keccak-256 hash function as specified in the
/// Ethereum protocol. It is used for computing hashes of various Ethereum data
/// structures, including transaction hashes, block hashes, and state root hashes.
///
/// # Arguments
/// * `bytes` - The input bytes to hash
///
/// # Returns
/// A 32-byte array containing the Keccak-256 hash of the input
///
/// # Note
/// The implementation uses either the standard or SP1-specific Keccak implementation
/// depending on the feature flags enabled at compile time.
pub fn digest_keccak(bytes: &[u8]) -> [u8; 32] {
    let mut hasher = Keccak::v256();
    let mut output = [0u8; 32];
    hasher.update(bytes);
    hasher.finalize(&mut output);
    output
}

/// Decodes an RLP-encoded Ethereum account into an `EthereumAccount` struct.
///
/// # Arguments
/// * `account_rlp` - The RLP-encoded account data to decode
///
/// # Returns
/// A `Result` containing either the decoded `EthereumAccount` or an error if decoding fails
///
/// # Errors
/// Returns an error if the RLP decoding fails or if any of the required fields are missing
pub fn rlp_decode_account(account_rlp: &[u8]) -> Result<EthereumAccount> {
    let account_rlp_bytes = rlp_decode_bytes(account_rlp)?;
    let nonce = if let Some(nonce_bytes) = account_rlp_bytes.first() {
        if nonce_bytes.is_empty() {
            0u64
        } else {
            u64::from_be_bytes({
                let mut padded = [0u8; 8];
                let nonce_slice = nonce_bytes.as_ref();
                let start = 8 - nonce_slice.len();
                padded[start..].copy_from_slice(nonce_slice);
                padded
            })
        }
    } else {
        0u64
    };
    let balance = BigUint::from_bytes_be(
        account_rlp_bytes
            .get(1)
            .context("Failed to get balance")?
            .as_ref(),
    );

    let storage_root = account_rlp_bytes
        .get(2)
        .context("Failed to get storage root")?
        .to_vec();

    let code_hash = account_rlp_bytes
        .get(3)
        .context("Failed to get code hash")?
        .to_vec();

    Ok(EthereumAccount::new(
        nonce,
        balance,
        storage_root,
        code_hash,
    ))
}

/// Decodes RLP-encoded bytes into a vector of bytes.
///
/// # Arguments
/// * `bytes` - The RLP-encoded bytes to decode
///
/// # Returns
/// A vector of decoded bytes
///
/// # Panics
/// Panics if the bytes cannot be decoded
pub fn rlp_decode_bytes(bytes: &[u8]) -> Result<Vec<timewave_rlp::Bytes>> {
    let decoded = timewave_rlp::decode_exact(bytes)
        .map_err(|e| anyhow::anyhow!("Failed to decode RLP bytes: {:?}", e))?;
    Ok(decoded)
}
