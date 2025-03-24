//! Merkle proof functionality for Neutron blockchain state verification.
//!
//! This module provides the core functionality for working with Merkle proofs on the
//! Neutron blockchain, including:
//! - Types for representing and verifying Merkle proofs
//! - Functions for converting between different proof formats
//! - RPC interaction for retrieving proofs from the blockchain
//!
//! The implementation follows the ICS23 standard for Merkle proofs and provides
//! specific support for Neutron's implementation of this standard.

pub mod helpers;
pub mod tests;
pub mod types;
