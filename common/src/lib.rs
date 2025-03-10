use serde::{Deserialize, Serialize};
use types::MerkleProofOutput;
pub mod types;
pub trait MerkleProver {
    #[allow(async_fn_in_trait)]
    // Obtain a proof for a slot in the key value store
    async fn get_storage_proof(&self, keys: Vec<&str>, address: &str, height: u64) -> Vec<u8>;
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
