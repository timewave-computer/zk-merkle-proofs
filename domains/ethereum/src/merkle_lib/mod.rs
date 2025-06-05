//! Ethereum Merkle proof library.
extern crate alloc;
use alloc::vec::Vec;

use anyhow::Result;

mod tests;
pub mod types;

pub use types::RlpDecodable;

// Keccak-256 hash function implementation.
//
// This module provides functionality for computing Keccak-256 hashes, which is the
// hash function used in Ethereum. It supports both standard and SP1-specific
// implementations through feature flags.

use sha3::{Digest, Keccak256};

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
    let mut hasher = Keccak256::new();
    hasher.update(bytes);
    hasher.finalize().into()
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
pub fn rlp_decode_bytes(bytes: &[u8]) -> Result<Vec<alloy_rlp::Bytes>> {
    let decoded = alloy_rlp::decode_exact(bytes)
        .map_err(|e| anyhow::anyhow!("Failed to decode RLP bytes: {:?}", e))?;
    Ok(decoded)
}
