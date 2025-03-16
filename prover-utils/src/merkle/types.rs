use ethereum::merkle_lib::types::EthereumMerkleProof;
use neutron::merkle_lib::types::NeutronMerkleProofWithRoot;
use serde::{Deserialize, Serialize};

/// Circuit input - multiple proofs for multiple domains
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MerkleProofInput {
    pub ethereum_proofs: Vec<EthereumMerkleProof>,
    pub neutron_proofs: Vec<NeutronMerkleProofWithRoot>,
}
