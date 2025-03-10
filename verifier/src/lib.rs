use common::{types::MerkleProofOutput, Verifiable};
use ethereum::EthereumProof;
use neutron::types::NeutronProof;
use serde::{Deserialize, Serialize};

pub fn verify_merkle_proof<T: Verifiable>(proof: T, expected_root: &[u8]) -> MerkleProofOutput {
    proof.verify(&expected_root)
}

/// Circuit input - multiple proofs for multiple domains
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MerkleProofInput {
    pub ethereum_proofs: Vec<EthereumProof>,
    pub neutron_proofs: Vec<NeutronProof>,
}
