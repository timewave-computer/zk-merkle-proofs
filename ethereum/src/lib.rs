use alloy_primitives::{FixedBytes, B256};
use eth_trie::{EthTrie, MemoryDB, Trie, DB};
use keccak::digest_keccak;
use std::sync::Arc;
pub mod keccak;
pub mod mock;
mod tests;

#[cfg(feature = "web")]
use {
    alloy::hex::FromHex,
    alloy::providers::{Provider, ProviderBuilder},
    alloy::rpc::types::EIP1186AccountProofResponse,
    alloy_primitives::Address,
    common::MerkleProver,
    std::str::FromStr,
    url::Url,
};
#[cfg(feature = "web")]
pub struct EvmProver {
    pub rpc_url: String,
}
#[cfg(feature = "web")]
impl MerkleProver for EvmProver {
    /// returns an account proof object for the requested address
    /// that contains a list of storage proofs for the requested keys
    /// we can verify the combined proof or extract the account proof
    /// and individual storage proofs
    async fn get_storage_proof(&self, keys: Vec<&str>, address: &str, height: u64) -> Vec<u8> {
        let address_object = Address::from_hex(&address).unwrap();
        let provider = ProviderBuilder::new().on_http(Url::from_str(&self.rpc_url).unwrap());
        let proof: EIP1186AccountProofResponse = provider
            .get_proof(
                address_object,
                keys.iter()
                    .map(|k| FixedBytes::from_hex(k).unwrap())
                    .collect(),
            )
            // use this in production!
            //.block_id(height.try_into().unwrap())
            .await
            .expect("Failed to get storage proof!");

        serde_json::to_vec(&proof).expect("Failed to serialize proof!")
    }
}

pub fn verify_merkle_proof(root_hash: Vec<u8>, proof: Vec<Vec<u8>>, key: &[u8]) -> Vec<u8> {
    let root_hash = FixedBytes::from_slice(&root_hash);
    let proof_db = Arc::new(MemoryDB::new(true));
    for node_encoded in proof.clone().into_iter() {
        let hash: B256 = digest_keccak(&node_encoded).into();
        proof_db.insert(hash.as_slice(), node_encoded).unwrap();
    }
    let mut trie = EthTrie::from(proof_db, root_hash).expect("Invalid merkle proof");
    assert_eq!(root_hash, trie.root_hash().unwrap());
    trie.verify_proof(root_hash, key, proof)
        .expect("Failed to verify Merkle Proof")
        .expect("Key does not exist!")
}
