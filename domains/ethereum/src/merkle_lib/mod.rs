//! Ethereum Merkle proof library.
//!
//! This library provides functionality for working with Ethereum Merkle proofs,
//! including computing and verifying proofs for accounts, storage, and receipts.

pub mod keccak;
#[cfg(feature = "no-zkvm")]
mod macros;
pub mod tests;
pub mod types;
