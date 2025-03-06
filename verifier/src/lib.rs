use common::{Domain, MerkleProof, ProofOutput};
use ethereum::verify_merkle_proof as verify_ethereum_merkle_proof;

pub fn verify_merkle_proof(proof: MerkleProof) -> ProofOutput {
    match proof.domain {
        Domain::ETHEREUM => {
            // verify an ethereum proof
            verify_ethereum_merkle_proof(proof.root.clone(), proof.nodes.clone(), &proof.key);
            ProofOutput {
                root: (proof.domain, proof.root.clone()),
                key: proof.key.clone(),
                value: proof.nodes.last().unwrap().clone(),
            }
        }
        Domain::COSMOS => {
            todo!("Cosmos support is pending, try with Ethereum for now!")
            // verify a cosmos proof
        }
    }
}
