//! Ethereum-specific functionality for handling Merkle tree operations.
#[cfg(feature = "no-zkvm")]
pub mod ethereum_rpc;
pub mod merkle_lib;
pub mod timewave_rlp;
pub mod timewave_trie;
