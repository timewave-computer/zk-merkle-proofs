//! A Rust crate for interacting with the Neutron blockchain, providing functionality for
//! Merkle proof verification and key management.

pub mod keys;
pub mod merkle_lib;
#[cfg(feature = "no-zkvm")]
pub mod rpc;
