//! Ethereum Merkle proof library.
//!
//! This library provides functionality for working with Ethereum Merkle proofs,
//! including:
//!
//! - Computing and verifying Merkle proofs for accounts, storage, and receipts
//! - RLP encoding and decoding of Ethereum data structures
//! - Keccak-256 hashing (Ethereum's hash function)
//! - Transaction receipt handling and proof generation
//!
//! # Modules
//!
//! - [`keccak`] - Keccak-256 hash function implementation
//! - [`logs`] - Transaction receipt logs handling
//! - [`types`] - Core types for Merkle proofs and verification
//! - [`tests`] - Test utilities and test cases

pub mod keccak;
#[cfg(feature = "no-zkvm")]
pub mod logs;
mod macros;
pub mod tests;
pub mod types;
