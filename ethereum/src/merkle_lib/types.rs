use alloy_primitives::{FixedBytes, B256};
use common::{types::MerkleProofOutput, MerkleVerifiable};
use eth_trie::{EthTrie, MemoryDB, Trie, DB};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EthereumProof {
    pub proof: Vec<Vec<u8>>,
    pub key: Vec<u8>,
    pub root: Vec<u8>,
}

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
    #[allow(unused)]
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

impl MerkleVerifiable for EthereumProof {
    fn verify(&self, expected_root: &[u8]) -> MerkleProofOutput {
        let root_hash = FixedBytes::from_slice(&expected_root);
        let proof_db = Arc::new(MemoryDB::new(true));
        for node_encoded in &self.proof.clone() {
            let hash: B256 = crate::merkle_lib::keccak::digest_keccak(&node_encoded).into();
            proof_db
                .insert(hash.as_slice(), node_encoded.to_vec())
                .unwrap();
        }
        let mut trie = EthTrie::from(proof_db, root_hash).expect("Invalid merkle proof");
        assert_eq!(root_hash, trie.root_hash().unwrap());
        trie.verify_proof(root_hash, &self.key, self.proof.clone())
            .expect("Failed to verify Merkle Proof")
            .expect("Key does not exist!");

        MerkleProofOutput {
            root: expected_root.to_vec(),
            key: self.key.clone(),
            // for Ethereum the value is the last node (a leaf) in the proof
            value: self.proof.last().unwrap().to_vec(),
            domain: common::Domain::ETHEREUM,
        }
    }
}
