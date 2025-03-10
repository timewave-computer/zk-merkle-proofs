#![no_main]
sp1_zkvm::entrypoint!(main);
use common::types::MerkleProofOutput;
use serde_json;
use verifier::{verify_merkle_proof, MerkleProofInput};
/// the logic that is to be proven
/// will likely call external functions, primarily verify_merkle_proof
/// enable sp1 as a feature to use keccak precompile
pub fn main() {
    let mut outputs: Vec<MerkleProofOutput> = vec![];
    let proof_batch: MerkleProofInput =
        serde_json::from_slice(&sp1_zkvm::io::read::<Vec<u8>>()).unwrap();

    // verify ethereum proofs
    for proof in proof_batch.ethereum_proofs {
        outputs.push(verify_merkle_proof(proof.clone(), &proof.root.clone()));
    }

    // verify neutron proofs
    /*for proof in proof_batch.neutron_proofs {
        outputs.push(verify_merkle_proof(proof, &proof.root));
    }*/
    // commit the serialized state roots and read values
    sp1_zkvm::io::commit_slice(&serde_json::to_vec(&outputs).unwrap());
}
