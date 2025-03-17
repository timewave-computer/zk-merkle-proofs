#![no_main]
sp1_zkvm::entrypoint!(main);
use common::merkle::types::MerkleProofOutput;
use prover_utils::merkle::{types::MerkleProofInput, verify_merkle_proof};
pub fn main() {
    let mut outputs: Vec<MerkleProofOutput> = vec![];
    let proof_batch: MerkleProofInput =
        serde_json::from_slice(&sp1_zkvm::io::read::<Vec<u8>>()).unwrap();
    // verify and commit a batch of Ethereum merkle proofs
    for proof in proof_batch.ethereum_proofs {
        outputs.push(verify_merkle_proof(proof.clone(), &proof.root.clone()));
    }
    // verify and commit a batch of neutron storage proofs
    for proof in proof_batch.neutron_proofs {
        outputs.push(verify_merkle_proof(proof.clone(), &proof.root));
    }
    sp1_zkvm::io::commit_slice(&serde_json::to_vec(&outputs).unwrap());
}
