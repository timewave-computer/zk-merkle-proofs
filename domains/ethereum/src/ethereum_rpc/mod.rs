//! Ethereum RPC client implementation for fetching Merkle proofs.
//! 
//! This crate provides functionality to interact with Ethereum nodes via RPC to fetch
//! various types of Merkle proofs, including account proofs, storage proofs, and receipt proofs.

/// RLP encoding utilities for Ethereum data structures.
pub mod rlp;

/// RPC client implementation for fetching Merkle proofs from Ethereum nodes.
pub mod rpc;
