use serde::{Deserialize, Serialize};

pub trait MerkleProver {
    #[allow(async_fn_in_trait)]
    // Obtain a proof for a slot in the key value store
    async fn get_storage_proof(&self, keys: Vec<&str>, address: &str) -> Vec<u8>;
}

#[derive(Serialize, Deserialize, Clone, Debug)]
/// The target domain to tell the proving system which MerkleProver it should use
pub enum Domain {
    // supported
    ETHEREUM,
    // unsupported
    COSMOS,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MerkleProof {
    // a list of serialized nodes
    nodes: Vec<Vec<u8>>,
    // target domain
    domain: Domain,
    // serialized trie root
    root: Vec<u8>,
}

/// Circuit input - multiple proofs for multiple domains
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProofInput {
    proofs: Vec<MerkleProof>,
}
