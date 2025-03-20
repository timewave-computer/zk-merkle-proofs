use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MerkleProofOutput {
    // commit to the encoded roots
    pub root: Vec<u8>,
    // the keys that were queried
    pub key: Vec<u8>,
    // commit to the encoded values
    pub value: Vec<u8>,
    // the domain
    pub domain: Domain,
}

pub trait MerkleProver {
    #[allow(async_fn_in_trait)]
    // Obtain a proof for a slot in the key value store
    async fn get_merkle_proof_from_rpc(&self, key: &str, address: &str, height: u64) -> Vec<u8>;
}

pub trait MerkleVerifiable {
    fn verify(&self, expected_root: &[u8]) -> MerkleProofOutput;
}

#[derive(Serialize, Deserialize, Clone, Debug)]
/// The target domain to tell the proving system which MerkleProver it should use
pub enum Domain {
    // supported
    ETHEREUM,
    // unsupported
    NEUTRON,
}
