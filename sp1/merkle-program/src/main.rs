#![no_main]
sp1_zkvm::entrypoint!(main);

/// the logic that is to be proven
/// will likely call external functions, primarily verify_merkle_proof
/// enable sp1 as a feature to use keccak precompile
pub fn main() {}
