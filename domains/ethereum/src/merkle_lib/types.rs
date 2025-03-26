//! Ethereum Merkle proof types and implementations.
//!
//! This module provides types and implementations for working with Ethereum Merkle proofs,
//! including account proofs, storage proofs, and receipt proofs.

use super::keccak::digest_keccak;
use alloy_primitives::{FixedBytes, B256};
use common::{merkle::types::MerkleProofOutput, merkle::types::MerkleVerifiable};
use eth_trie::{EthTrie, MemoryDB, Trie, DB};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[cfg(feature = "no-zkvm")]
use {
    crate::merkle_lib::logs::insert_receipt,
    alloy::hex::{self, FromHex},
    alloy::providers::{Provider, ProviderBuilder},
    alloy::rpc::types::EIP1186AccountProofResponse,
    alloy::{consensus::ReceiptEnvelope, rpc::types::TransactionReceipt, serde::JsonStorageKey},
    alloy_primitives::Address,
    alloy_rlp::BufMut,
    common::merkle::types::MerkleProver,
    std::{io::Read, str::FromStr},
    url::Url,
};

/// Represents an Ethereum Merkle proof with its associated data.
///
/// This struct contains all the necessary components to verify a Merkle proof
/// in the Ethereum state trie, storage trie, or receipt trie.
///
/// # Fields
/// * `proof` - The list of proof nodes in the Merkle path
/// * `key` - The key being proven (e.g., account address, storage key, or receipt index)
/// * `root` - The root hash of the trie being proven
/// * `value` - The RLP-encoded value being proven
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EthereumMerkleProof {
    /// The list of proof nodes in the Merkle path
    pub proof: Vec<Vec<u8>>,
    /// The key being proven
    pub key: Vec<u8>,
    /// The RLP-encoded value being proven
    pub value: Vec<u8>,
}
impl EthereumMerkleProof {
    pub fn new(proof: Vec<Vec<u8>>, key: Vec<u8>, value: Vec<u8>) -> Self {
        Self {
            proof,
            key: digest_keccak(&key).to_vec(),
            value,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EthereumRawMerkleProof {
    pub proof: Vec<Vec<u8>>,
    pub key: Vec<u8>,
    pub value: Vec<u8>,
}
impl EthereumRawMerkleProof {
    pub fn new(proof: Vec<Vec<u8>>, key: Vec<u8>, value: Vec<u8>) -> Self {
        Self { proof, key, value }
    }
}

impl From<EthereumRawMerkleProof> for EthereumMerkleProof {
    fn from(proof: EthereumRawMerkleProof) -> Self {
        Self {
            proof: proof.proof,
            key: digest_keccak(&proof.key).to_vec(),
            value: proof.value,
        }
    }
}

impl EthereumRawMerkleProof {
    pub fn as_raw_merkle_proof(&self) -> EthereumMerkleProof {
        EthereumMerkleProof {
            proof: self.proof.clone(),
            key: self.key.clone(),
            value: self.value.clone(),
        }
    }
}

/// A Merkle prover implementation for Ethereum.
///
/// This struct provides functionality to fetch and verify Merkle proofs
/// from an Ethereum node via RPC.
#[cfg(feature = "no-zkvm")]
pub struct MerkleProverEvm {
    /// The RPC endpoint URL
    pub rpc_url: String,
}

#[cfg(feature = "no-zkvm")]
impl MerkleProver for MerkleProverEvm {
    /// Retrieves an account proof from an Ethereum node.
    ///
    /// # Arguments
    /// * `key` - The storage key to prove
    /// * `address` - The account address to prove
    /// * `height` - The block height to prove at
    ///
    /// # Returns
    /// A vector of bytes containing the serialized proof
    ///
    /// # Panics
    /// Panics if the RPC call fails or if the proof cannot be serialized
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

#[cfg(feature = "no-zkvm")]
impl MerkleProverEvm {
    /// Retrieves both account and storage proofs for a given account and storage key.
    ///
    /// # Arguments
    /// * `key` - The storage key to prove
    /// * `address` - The account address to prove
    /// * `height` - The block height to prove at
    /// * `block_state_root` - The state root of the block
    /// * `storage_hash` - The storage root hash of the account
    ///
    /// # Returns
    /// A tuple containing the account proof and storage proof
    ///
    /// # Panics
    /// Panics if the proofs cannot be retrieved or deserialized
    pub async fn get_account_and_storage_proof(
        &self,
        key: &str,
        address: &str,
        height: u64,
    ) -> (EthereumMerkleProof, EthereumMerkleProof) {
        let proof = self.get_merkle_proof_from_rpc(key, address, height).await;
        let proof_deserialized: EIP1186AccountProofResponse =
            serde_json::from_slice(&proof).unwrap();
        let account_proof: Vec<Vec<u8>> = proof_deserialized
            .account_proof
            .iter()
            .map(|b| b.to_vec())
            .collect();
        let account_proof =
            EthereumMerkleProof::new(account_proof.clone(), hex::decode(address).unwrap(), vec![]);
        let raw_storage_proofs: Vec<(Vec<Vec<u8>>, JsonStorageKey)> = proof_deserialized
            .storage_proof
            .iter()
            .cloned()
            .map(|p| (p.proof.into_iter().map(|b| b.to_vec()).collect(), p.key))
            .collect();
        let first_storage_proof = raw_storage_proofs.first().unwrap();
        let storage_proof = EthereumMerkleProof::new(
            first_storage_proof.0.clone(),
            first_storage_proof
                .1
                .as_b256()
                .bytes()
                .collect::<Result<Vec<u8>, _>>()
                .unwrap()
                .to_vec(),
            vec![],
        );
        (account_proof, storage_proof)
    }

    pub async fn get_account_proof(
        &self,
        key: &str,
        address: &str,
        height: u64,
    ) -> EthereumMerkleProof {
        let proof = self.get_merkle_proof_from_rpc(key, address, height).await;
        let proof_deserialized: EIP1186AccountProofResponse =
            serde_json::from_slice(&proof).unwrap();
        let account_proof: Vec<Vec<u8>> = proof_deserialized
            .account_proof
            .iter()
            .map(|b| b.to_vec())
            .collect();
        EthereumMerkleProof::new(account_proof.clone(), hex::decode(address).unwrap(), vec![])
    }

    pub async fn get_storage_proof(
        &self,
        key: &str,
        address: &str,
        height: u64,
    ) -> EthereumMerkleProof {
        let proof = self.get_merkle_proof_from_rpc(key, address, height).await;
        let proof_deserialized: EIP1186AccountProofResponse =
            serde_json::from_slice(&proof).unwrap();
        let raw_storage_proofs: Vec<(Vec<Vec<u8>>, JsonStorageKey)> = proof_deserialized
            .storage_proof
            .iter()
            .cloned()
            .map(|p| (p.proof.into_iter().map(|b| b.to_vec()).collect(), p.key))
            .collect();
        let first_storage_proof = raw_storage_proofs.first().unwrap();
        EthereumMerkleProof::new(
            first_storage_proof.0.clone(),
            first_storage_proof
                .1
                .as_b256()
                .bytes()
                .collect::<Result<Vec<u8>, _>>()
                .unwrap()
                .to_vec(),
            vec![],
        )
    }

    /// Retrieves a receipt proof for a specific transaction in a block.
    ///
    /// # Arguments
    /// * `block_hash` - The hash of the block containing the receipt
    /// * `target_index` - The index of the receipt in the block
    ///
    /// # Returns
    /// A Merkle proof for the receipt
    ///
    /// # Panics
    /// Panics if the block or receipts cannot be retrieved, or if the proof cannot be constructed
    pub async fn get_receipt_proof(
        &self,
        block_height: u64,
        target_index: u32,
    ) -> EthereumMerkleProof {
        let provider = ProviderBuilder::new().on_http(Url::from_str(&self.rpc_url).unwrap());
        let receipts: Vec<TransactionReceipt> = provider
            .get_block_receipts(alloy::eips::BlockId::Number(
                alloy::eips::BlockNumberOrTag::Number(block_height),
            ))
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
                    insert_receipt(r, &mut trie, &index_encoded, Some(prefix));
                }
                ReceiptEnvelope::Eip1559(r) => {
                    let prefix: u8 = 0x02;
                    insert_receipt(r, &mut trie, &index_encoded, Some(prefix));
                }
                ReceiptEnvelope::Eip4844(r) => {
                    let prefix: u8 = 0x03;
                    out.put_u8(0x03);
                    insert_receipt(r, &mut trie, &index_encoded, Some(prefix));
                }
                ReceiptEnvelope::Eip7702(r) => {
                    let prefix: u8 = 0x04;
                    out.put_u8(0x04);
                    insert_receipt(r, &mut trie, &index_encoded, Some(prefix));
                }
                ReceiptEnvelope::Legacy(r) => {
                    insert_receipt(r, &mut trie, &index_encoded, None);
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
        // must preserve the raw proof for the receipt
        EthereumRawMerkleProof::new(proof, receipt_key, serde_json::to_vec(&receipts).unwrap())
            .as_raw_merkle_proof()
    }
}

/// Implementation of Merkle proof verification for Ethereum proofs.
///
/// This implementation verifies proofs against the Ethereum state trie,
/// storage trie, or receipt trie.
impl MerkleVerifiable for EthereumMerkleProof {
    /// Verifies a Merkle proof against an expected root hash.
    ///
    /// # Arguments
    /// * `expected_root` - The expected root hash to verify against
    ///
    /// # Returns
    /// A `MerkleProofOutput` containing the verification result
    ///
    /// # Panics
    /// Panics if the proof is invalid or if the key does not exist
    fn verify(&self, expected_root: &[u8]) -> MerkleProofOutput {
        let root_hash: FixedBytes<32> = FixedBytes::from_slice(expected_root);
        let proof_db = Arc::new(MemoryDB::new(true));
        for node_encoded in &self.proof {
            let hash: B256 = digest_keccak(node_encoded).into();
            proof_db
                .insert(hash.as_slice(), node_encoded.to_vec())
                .unwrap();
        }
        let mut trie = EthTrie::from(proof_db, root_hash).expect("Invalid merkle proof");
        assert_eq!(root_hash, trie.root_hash().unwrap());
        let value = trie
            .verify_proof(root_hash, &self.key.clone(), self.proof.clone())
            .expect("Failed to verify Merkle Proof")
            .expect("Key does not exist!");
        MerkleProofOutput {
            root: expected_root.to_vec(),
            key: self.key.clone(),
            value,
        }
    }
}
