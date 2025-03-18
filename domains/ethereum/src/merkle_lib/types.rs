use alloy_primitives::{FixedBytes, B256};
use common::{merkle::types::MerkleProofOutput, merkle::types::MerkleVerifiable};
use eth_trie::{EthTrie, MemoryDB, Trie, DB};
use serde::{Deserialize, Serialize};
use std::{io::Read, sync::Arc};

#[cfg(feature = "no-sp1")]
use {
    crate::merkle_lib::logs::insert_receipt,
    alloy::hex::{self, FromHex},
    alloy::providers::{Provider, ProviderBuilder},
    alloy::rpc::types::EIP1186AccountProofResponse,
    alloy::{consensus::ReceiptEnvelope, rpc::types::TransactionReceipt, serde::JsonStorageKey},
    alloy_primitives::Address,
    alloy_rlp::BufMut,
    common::merkle::types::MerkleProver,
    std::str::FromStr,
    url::Url,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EthereumMerkleProof {
    pub proof: Vec<Vec<u8>>,
    pub key: Vec<u8>,
    pub root: Vec<u8>,
    // the rlp encoded value
    pub value: Vec<u8>,
}

impl EthereumMerkleProof {
    pub fn hash_key(&mut self) {
        self.key = digest_keccak(&self.key).to_vec()
    }
}

use super::keccak::digest_keccak;
#[cfg(feature = "no-sp1")]
pub struct MerkleProverEvm {
    pub rpc_url: String,
}
#[cfg(feature = "no-sp1")]
impl MerkleProver for MerkleProverEvm {
    /// returns an account proof object for the requested address
    /// that contains a list of storage proofs for the requested keys
    /// we can verify the combined proof or extract the account proof
    /// and individual storage proofs
    async fn get_merkle_proof_from_rpc(&self, key: &str, address: &str, height: u64) -> Vec<u8> {
        let address_object = Address::from_hex(address).unwrap();
        let provider = ProviderBuilder::new().on_http(Url::from_str(&self.rpc_url).unwrap());
        let proof: EIP1186AccountProofResponse = provider
            .get_proof(address_object, vec![FixedBytes::from_hex(key).unwrap()])
            .block_id(height.into())
            .await
            .expect("Failed to get storage proof!");
        serde_json::to_vec(&proof).expect("Failed to serialize proof!")
    }
}

#[cfg(feature = "no-sp1")]
impl MerkleProverEvm {
    pub async fn get_account_and_storage_proof(
        &self,
        key: &str,
        address: &str,
        height: u64,
        block_state_root: &[u8],
        storage_hash: Vec<u8>,
    ) -> (EthereumMerkleProof, EthereumMerkleProof) {
        let proof = self.get_merkle_proof_from_rpc(key, address, height).await;
        let proof_deserialized: EIP1186AccountProofResponse =
            serde_json::from_slice(&proof).unwrap();
        let account_proof: Vec<Vec<u8>> = proof_deserialized
            .account_proof
            .iter()
            .map(|b| b.to_vec())
            .collect();
        let account_proof = EthereumMerkleProof {
            root: block_state_root.to_vec(),
            proof: account_proof.clone(),
            key: hex::decode(address).unwrap(),
            value: storage_hash,
        };
        let raw_storage_proofs: Vec<(Vec<Vec<u8>>, JsonStorageKey)> = proof_deserialized
            .storage_proof
            .iter()
            .cloned()
            .map(|p| (p.proof.into_iter().map(|b| b.to_vec()).collect(), p.key))
            .collect();
        let first_storage_proof = raw_storage_proofs.first().unwrap();
        let storage_proof = EthereumMerkleProof {
            root: proof_deserialized.storage_hash.to_vec(),
            proof: first_storage_proof.0.clone(),
            key: first_storage_proof
                .1
                .as_b256()
                .bytes()
                .collect::<Result<Vec<u8>, _>>()
                .unwrap()
                .to_vec(),
            value: alloy_rlp::encode(proof_deserialized.storage_proof.first().unwrap().value),
        };
        (account_proof, storage_proof)
    }

    pub async fn get_receipt_proof(
        &self,
        block_hash: &str,
        target_index: u32,
    ) -> EthereumMerkleProof {
        let provider = ProviderBuilder::new().on_http(Url::from_str(&self.rpc_url).unwrap());
        let block_hash_b256 = B256::from_str(block_hash).unwrap();
        let block = provider
            .get_block_by_hash(
                B256::from_str(block_hash).unwrap(),
                // for alloy < 0.12
                //alloy::rpc::types::BlockTransactionsKind::Full,
            )
            .await
            .expect("Failed to get Block!")
            .expect("Block not found!");
        let receipts: Vec<TransactionReceipt> = provider
            .get_block_receipts(alloy::eips::BlockId::Hash(block_hash_b256.into()))
            .await
            .unwrap()
            .unwrap();
        let memdb = Arc::new(MemoryDB::new(true));
        let mut trie = EthTrie::new(memdb.clone());
        for (index, receipt) in receipts.clone().into_iter().enumerate() {
            let inner: ReceiptEnvelope<alloy::rpc::types::Log> = receipt.inner;
            let mut out: Vec<u8> = Vec::new();
            let index_encoded = alloy_rlp::encode(index);
            match inner {
                ReceiptEnvelope::Eip2930(r) => {
                    let prefix: u8 = 0x01;
                    insert_receipt(r, &mut trie, index_encoded, Some(prefix));
                }
                ReceiptEnvelope::Eip1559(r) => {
                    let prefix: u8 = 0x02;
                    insert_receipt(r, &mut trie, index_encoded, Some(prefix));
                }
                ReceiptEnvelope::Eip4844(r) => {
                    let prefix: u8 = 0x03;
                    out.put_u8(0x03);
                    insert_receipt(r, &mut trie, index_encoded, Some(prefix));
                }
                ReceiptEnvelope::Eip7702(r) => {
                    let prefix: u8 = 0x04;
                    out.put_u8(0x04);
                    insert_receipt(r, &mut trie, index_encoded, Some(prefix));
                }
                ReceiptEnvelope::Legacy(r) => {
                    insert_receipt(r, &mut trie, index_encoded, None);
                }
                #[allow(unreachable_patterns)]
                _ => {
                    eprintln!("Unknown Receipt Type!")
                }
            }
        }
        trie.root_hash().unwrap();
        let receipt_key: Vec<u8> = alloy_rlp::encode(target_index);
        let proof = trie.get_proof(&receipt_key).unwrap();
        EthereumMerkleProof {
            proof,
            root: block.header.receipts_root.to_vec(),
            key: receipt_key,
            value: serde_json::to_vec(&receipts).unwrap(),
        }
    }
}

impl MerkleVerifiable for EthereumMerkleProof {
    fn verify(&self, expected_root: &[u8]) -> MerkleProofOutput {
        let root_hash: FixedBytes<32> = FixedBytes::from_slice(expected_root);
        let proof_db = Arc::new(MemoryDB::new(true));
        for node_encoded in &self.proof.clone() {
            let hash: B256 = crate::merkle_lib::keccak::digest_keccak(node_encoded).into();
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
            domain: common::merkle::types::Domain::ETHEREUM,
        }
    }
}
