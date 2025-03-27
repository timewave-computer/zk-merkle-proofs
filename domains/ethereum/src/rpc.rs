use std::{io::Read, str::FromStr, sync::Arc};

use alloy::{
    consensus::ReceiptEnvelope,
    hex::{self, FromHex},
    providers::{Provider, ProviderBuilder},
    rpc::types::{EIP1186AccountProofResponse, TransactionReceipt},
    serde::JsonStorageKey,
};
use alloy_primitives::{Address, FixedBytes};
use alloy_rlp::BufMut;
use common::merkle::types::MerkleRpcClient;
use eth_trie::{EthTrie, MemoryDB, Trie};
use url::Url;

use crate::merkle_lib::{
    logs::insert_receipt,
    types::{decode_rlp_bytes, EthereumMerkleProof, EthereumRawMerkleProof},
};

/// A Merkle prover implementation for Ethereum.
///
/// This struct provides functionality to fetch and verify Merkle proofs
/// from an Ethereum node via RPC.
///
/// # Fields
/// * `rpc_url` - The RPC endpoint URL of the Ethereum node
pub struct EvmMerkleRpcClient {
    /// The RPC endpoint URL
    pub rpc_url: String,
}

impl MerkleRpcClient for EvmMerkleRpcClient {
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
    async fn get_proof(&self, key: &str, address: &str, height: u64) -> Vec<u8> {
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

impl EvmMerkleRpcClient {
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
        let proof = self.get_proof(key, address, height).await;
        let proof_deserialized: EIP1186AccountProofResponse =
            serde_json::from_slice(&proof).unwrap();
        let account_proof: Vec<Vec<u8>> = proof_deserialized
            .account_proof
            .iter()
            .map(|b| b.to_vec())
            .collect();
        let leaf_node_decoded: Vec<alloy_primitives::Bytes> =
            decode_rlp_bytes(proof_deserialized.account_proof.last().unwrap());
        let stored_account = leaf_node_decoded.last().unwrap().to_vec();
        let account_proof = EthereumMerkleProof::new(
            account_proof.clone(),
            hex::decode(address).unwrap(),
            stored_account,
        );
        let raw_storage_proofs: Vec<(Vec<Vec<u8>>, JsonStorageKey)> = proof_deserialized
            .storage_proof
            .iter()
            .cloned()
            .map(|p| (p.proof.into_iter().map(|b| b.to_vec()).collect(), p.key))
            .collect();
        let first_storage_proof = raw_storage_proofs.first().unwrap();
        let leaf_node_decoded: Vec<alloy_primitives::Bytes> =
            decode_rlp_bytes(&first_storage_proof.0.to_vec().last().unwrap().to_vec());
        let stored_value = leaf_node_decoded.last().unwrap().to_vec();
        let storage_proof = EthereumMerkleProof::new(
            first_storage_proof.0.clone(),
            first_storage_proof
                .1
                .as_b256()
                .bytes()
                .collect::<Result<Vec<u8>, _>>()
                .unwrap()
                .to_vec(),
            stored_value,
        );
        (account_proof, storage_proof)
    }

    /// Retrieves an account proof for a given address.
    ///
    /// # Arguments
    /// * `key` - The storage key to prove
    /// * `address` - The account address to prove
    /// * `height` - The block height to prove at
    ///
    /// # Returns
    /// An account proof for the given address
    ///
    /// # Panics
    /// Panics if the proof cannot be retrieved or deserialized
    pub async fn get_account_proof(
        &self,
        key: &str,
        address: &str,
        height: u64,
    ) -> EthereumMerkleProof {
        let proof = self.get_proof(key, address, height).await;
        let proof_deserialized: EIP1186AccountProofResponse =
            serde_json::from_slice(&proof).unwrap();
        let account_proof: Vec<Vec<u8>> = proof_deserialized
            .account_proof
            .iter()
            .map(|b| b.to_vec())
            .collect();
        let leaf_node_decoded = decode_rlp_bytes(proof_deserialized.account_proof.last().unwrap());
        let stored_account = leaf_node_decoded.last().unwrap().to_vec();
        EthereumMerkleProof::new(
            account_proof.clone(),
            hex::decode(address).unwrap(),
            stored_account,
        )
    }

    /// Retrieves a storage proof for a given account and storage key.
    ///
    /// # Arguments
    /// * `key` - The storage key to prove
    /// * `address` - The account address to prove
    /// * `height` - The block height to prove at
    ///
    /// # Returns
    /// A storage proof for the given account and storage key
    ///
    /// # Panics
    /// Panics if the proof cannot be retrieved or deserialized
    pub async fn get_storage_proof(
        &self,
        key: &str,
        address: &str,
        height: u64,
    ) -> EthereumMerkleProof {
        let proof = self.get_proof(key, address, height).await;
        let proof_deserialized: EIP1186AccountProofResponse =
            serde_json::from_slice(&proof).unwrap();
        let raw_storage_proofs: Vec<(Vec<Vec<u8>>, JsonStorageKey)> = proof_deserialized
            .storage_proof
            .iter()
            .cloned()
            .map(|p| (p.proof.into_iter().map(|b| b.to_vec()).collect(), p.key))
            .collect();
        let first_storage_proof = raw_storage_proofs.first().unwrap();
        let leaf_node_decoded: Vec<alloy_primitives::Bytes> =
            alloy_rlp::decode_exact(first_storage_proof.0.to_vec().last().unwrap()).unwrap();
        let stored_value = leaf_node_decoded.last().unwrap().to_vec();
        EthereumMerkleProof::new(
            first_storage_proof.0.clone(),
            first_storage_proof
                .1
                .as_b256()
                .bytes()
                .collect::<Result<Vec<u8>, _>>()
                .unwrap()
                .to_vec(),
            stored_value,
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
        let leaf_node_decoded: Vec<alloy_primitives::Bytes> =
            alloy_rlp::decode_exact(proof.last().unwrap()).unwrap();
        // a single, rlp-encoded receipt
        let receipt_rlp = leaf_node_decoded.clone().last().unwrap().to_vec();
        EthereumRawMerkleProof::new(proof, receipt_key, receipt_rlp).into()
    }
}
