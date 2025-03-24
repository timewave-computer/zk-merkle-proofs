//! Keccak-256 hash function implementation.
//!
//! This module provides functionality for computing Keccak-256 hashes,
//! which is the hash function used in Ethereum.

#[cfg(not(feature = "sp1"))]
use tiny_keccak::{Hasher, Keccak};
#[cfg(feature = "sp1")]
use tiny_keccak_sp1::{Hasher, Keccak};

/// Computes the Keccak-256 hash of the input bytes.
///
/// This function is used throughout Ethereum for various hashing operations,
/// including computing transaction hashes, block hashes, and state root hashes.
///
/// # Arguments
/// * `bytes` - The input bytes to hash
///
/// # Returns
/// A 32-byte array containing the Keccak-256 hash
pub fn digest_keccak(bytes: &[u8]) -> [u8; 32] {
    let mut hasher = Keccak::v256();
    let mut output = [0u8; 32];
    hasher.update(bytes);
    hasher.finalize(&mut output);
    output
}
