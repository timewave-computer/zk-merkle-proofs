#![cfg_attr(not(feature = "no-zkvm"), no_std)]
//! Ethereum-specific functionality for handling Merkle tree operations.
#[cfg(feature = "no-zkvm")]
pub mod ethereum_rpc;
pub mod merkle_lib;
