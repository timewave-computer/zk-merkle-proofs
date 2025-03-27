//! Ethereum-specific functionality for handling Merkle tree operations.
//! This module provides utilities for decoding and processing Ethereum's Merkle Patricia Trie leaf nodes.

pub mod merkle_lib;
#[cfg(feature = "no-zkvm")]
pub mod rpc;
