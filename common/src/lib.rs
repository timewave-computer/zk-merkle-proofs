use serde::{Deserialize, Serialize};
pub mod types;
pub trait MerkleProver {
    #[allow(async_fn_in_trait)]
    // Obtain a proof for a slot in the key value store
    async fn get_storage_proof(&self, keys: Vec<&str>, address: &str, height: u64) -> Vec<u8>;
}

#[derive(Serialize, Deserialize, Clone, Debug)]
/// The target domain to tell the proving system which MerkleProver it should use
pub enum Domain {
    // supported
    ETHEREUM,
    // unsupported
    COSMOS,
}
