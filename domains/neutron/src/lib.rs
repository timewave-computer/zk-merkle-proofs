//! A Rust crate for interacting with the Neutron blockchain, providing functionality for
//! Merkle proof verification and key management.
//!
//! This crate implements the necessary types and functions to:
//! - Generate and verify Merkle proofs for Neutron blockchain state
//! - Manage Neutron-specific keys for various storage types (bank, wasm, etc.)
//! - Interact with Neutron RPC endpoints for proof retrieval
//!
//! The crate is designed to work with the Neutron blockchain's specific implementation
//! of the ICS23 standard for Merkle proofs.

pub mod keys;
pub mod merkle_lib;
