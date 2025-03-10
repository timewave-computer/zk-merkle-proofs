use common::{merkle::types::MerkleProofOutput, merkle::types::MerkleVerifiable};
use ethereum::merkle_lib::types::EthereumProof;
use neutron::merkle_lib::types::NeutronProofWithRoot;
use serde::{Deserialize, Serialize};

pub fn verify_merkle_proof<T: MerkleVerifiable>(
    proof: T,
    expected_root: &[u8],
) -> MerkleProofOutput {
    proof.verify(expected_root)
}

/// Circuit input - multiple proofs for multiple domains
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MerkleProofInput {
    pub ethereum_proofs: Vec<EthereumProof>,
    pub neutron_proofs: Vec<NeutronProofWithRoot>,
}
