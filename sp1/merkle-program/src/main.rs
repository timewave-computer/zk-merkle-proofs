#![no_main]
sp1_zkvm::entrypoint!(main);
use common::{Domain, ProofInput, ProofOutput};
use serde_json;
use verifier::verify_merkle_proof;
/// the logic that is to be proven
/// will likely call external functions, primarily verify_merkle_proof
/// enable sp1 as a feature to use keccak precompile
pub fn main() {
    let mut outputs: Vec<ProofOutput> = vec![];
    let proof_batch: ProofInput = serde_json::from_slice(&sp1_zkvm::io::read::<Vec<u8>>()).unwrap();
    // todo: factor this out
    for proof in proof_batch.proofs {
        outputs.push(verify_merkle_proof(proof));
    }
    // commit the serialized state roots and read values
    sp1_zkvm::io::commit_slice(&serde_json::to_vec(&outputs).unwrap());
}
