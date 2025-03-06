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
    // the last node should be the queried value
    pub nodes: Vec<Vec<u8>>,
    // the key that we query
    pub key: Vec<u8>,
    // target domain
    pub domain: Domain,
    // serialized trie root
    pub root: Vec<u8>,
}

/// Circuit input - multiple proofs for multiple domains
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProofInput {
    pub proofs: Vec<MerkleProof>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProofOutput {
    // commit to the encoded roots
    pub root: (Domain, Vec<u8>),
    // the keys that were queried
    pub key: Vec<u8>,
    // commit to the encoded values
    pub value: Vec<u8>,
}
