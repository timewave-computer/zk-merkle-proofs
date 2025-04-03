use alloy::{
    hex::{self, FromHex},
    providers::{Provider, ProviderBuilder},
    rpc::types::{EIP1186AccountProofResponse, TransactionReceipt},
    serde::JsonStorageKey,
};
use alloy_primitives::{Address, FixedBytes};
use alloy_trie::{proof::ProofRetainer, root::adjust_index_for_rlp, HashBuilder, Nibbles};
use anyhow::{Context, Result};
use common::merkle::types::MerkleClient;
use std::{io::Read, str::FromStr};
use url::Url;

use crate::{
    ethereum_rpc::rlp::encode_receipt,
    merkle_lib::types::{decode_rlp_bytes, EthereumMerkleProof, EthereumRawMerkleProof},
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

impl MerkleClient for EvmMerkleRpcClient {
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
    async fn get_proof(&self, key: &str, address: &str, height: u64) -> Result<Vec<u8>> {
        let address_object = Address::from_hex(address)?;
        let provider = ProviderBuilder::new().on_http(Url::from_str(&self.rpc_url)?);
        let proof: EIP1186AccountProofResponse = provider
            .get_proof(address_object, vec![FixedBytes::from_hex(key)?])
            .block_id(height.into())
            .await?;
        Ok(serde_json::to_vec(&proof)?)
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
    ) -> Result<(EthereumMerkleProof, EthereumMerkleProof)> {
        let proof = self.get_proof(key, address, height).await?;
        let proof_deserialized: EIP1186AccountProofResponse = serde_json::from_slice(&proof)?;
        let account_proof: Vec<Vec<u8>> = proof_deserialized
            .account_proof
            .iter()
            .map(|b| b.to_vec())
            .collect();
        let leaf_node_decoded: Vec<crate::timewave_rlp::Bytes> = decode_rlp_bytes(
            proof_deserialized
                .account_proof
                .last()
                .context("Failed to get last account proof")?,
        )?;
        let stored_account = leaf_node_decoded
            .last()
            .context("Failed to extract leaf from account proof")?
            .to_vec();
        let account_proof =
            EthereumMerkleProof::new(account_proof.clone(), hex::decode(address)?, stored_account);
        let raw_storage_proofs: Vec<(Vec<Vec<u8>>, JsonStorageKey)> = proof_deserialized
            .storage_proof
            .iter()
            .cloned()
            .map(|p| (p.proof.into_iter().map(|b| b.to_vec()).collect(), p.key))
            .collect();
        let first_storage_proof = raw_storage_proofs
            .first()
            .context("Failed to get first storage proof");
        let first_storage_proof = first_storage_proof?;
        let leaf_node_decoded: Vec<crate::timewave_rlp::Bytes> = decode_rlp_bytes(
            &first_storage_proof
                .0
                .to_vec()
                .last()
                .context("Failed to get last storage proof")?
                .to_vec(),
        )?;
        let stored_value = leaf_node_decoded
            .last()
            .context("Failed to extract leaf from storage proof");
        let storage_proof = EthereumMerkleProof::new(
            first_storage_proof.0.clone(),
            first_storage_proof
                .1
                .as_b256()
                .bytes()
                .collect::<Result<Vec<u8>, _>>()?
                .to_vec(),
            stored_value
                .context("Failed to extract leaf from storage proof")?
                .to_vec(),
        );
        Ok((account_proof, storage_proof))
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
    ) -> Result<EthereumMerkleProof> {
        let proof = self.get_proof(key, address, height).await?;
        let proof_deserialized: EIP1186AccountProofResponse = serde_json::from_slice(&proof)?;
        let account_proof: Vec<Vec<u8>> = proof_deserialized
            .account_proof
            .iter()
            .map(|b| b.to_vec())
            .collect();
        let leaf_node_decoded = decode_rlp_bytes(
            proof_deserialized
                .account_proof
                .last()
                .context("Failed to get leaf from account proof")?,
        )?;
        let stored_account = leaf_node_decoded
            .last()
            .context("Failed to extract account root from leaf")?
            .to_vec();
        Ok(EthereumMerkleProof::new(
            account_proof.clone(),
            hex::decode(address)?,
            stored_account,
        ))
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
    ) -> Result<EthereumMerkleProof> {
        let proof = self.get_proof(key, address, height).await?;
        let proof_deserialized: EIP1186AccountProofResponse = serde_json::from_slice(&proof)?;
        let raw_storage_proofs: Vec<(Vec<Vec<u8>>, JsonStorageKey)> = proof_deserialized
            .storage_proof
            .iter()
            .cloned()
            .map(|p| (p.proof.into_iter().map(|b| b.to_vec()).collect(), p.key))
            .collect();
        let first_storage_proof = raw_storage_proofs
            .first()
            .context("Failed to get first storage proof")?;
        let leaf_node_decoded: Vec<crate::timewave_rlp::Bytes> =
            match crate::timewave_rlp::decode_exact(
                first_storage_proof
                    .0
                    .to_vec()
                    .last()
                    .context("Failed to extract leaf from storage proof")?,
            ) {
                Ok(decoded) => decoded,
                Err(e) => {
                    return Err(anyhow::anyhow!("Failed to decode RLP bytes: {:?}", e));
                }
            };
        let stored_value = leaf_node_decoded
            .last()
            .context("Failed to extract value from leaf")?
            .to_vec();
        Ok(EthereumMerkleProof::new(
            first_storage_proof.0.clone(),
            first_storage_proof
                .1
                .as_b256()
                .bytes()
                .collect::<Result<Vec<u8>, _>>()?
                .to_vec(),
            stored_value,
        ))
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
    ) -> Result<EthereumMerkleProof> {
        let provider = ProviderBuilder::new().on_http(Url::from_str(&self.rpc_url)?);
        let receipts: Vec<TransactionReceipt> = provider
            .get_block_receipts(alloy::eips::BlockId::Number(
                alloy::eips::BlockNumberOrTag::Number(block_height),
            ))
            .await?
            .context("Failed to get block receipts")?;
        let retainer = ProofRetainer::new(vec![Nibbles::unpack(
            crate::timewave_rlp::encode_fixed_size(&target_index),
        )]);
        let mut hb: HashBuilder = HashBuilder::default().with_proof_retainer(retainer);
        for i in 0..receipts.len() {
            let index = adjust_index_for_rlp(i, receipts.len());
            let index_buffer = crate::timewave_rlp::encode_fixed_size(&index);
            hb.add_leaf(
                Nibbles::unpack(&index_buffer),
                encode_receipt(&receipts[index])?.as_slice(),
            );
        }
        let receipt_key: Vec<u8> = crate::timewave_rlp::encode(target_index);
        hb.root();
        let proof = hb
            .take_proof_nodes()
            .into_nodes_sorted()
            .into_iter()
            .map(|n| n.1)
            .collect::<Vec<_>>()
            .iter()
            .map(|n| n.to_vec())
            .collect::<Vec<_>>();
        let leaf_node_decoded: Vec<crate::timewave_rlp::Bytes> = decode_rlp_bytes(
            proof
                .to_vec()
                .last()
                .context("Failed to extract leaf from receipt proof")?,
        )?;
        let receipt_rlp = leaf_node_decoded
            .last()
            .context("Failed to extract value from leaf")?
            .to_vec();
        Ok(EthereumRawMerkleProof::new(proof, receipt_key, receipt_rlp).into())
    }
}
