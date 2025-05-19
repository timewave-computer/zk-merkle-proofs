//! Ethereum Merkle proof types and implementations.
//!
//! This module provides types and implementations for working with Ethereum Merkle proofs,
//! including account proofs, storage proofs, and receipt proofs. It implements the common
//! Merkle proof traits for Ethereum-specific data structures and provides functionality
//! to fetch and verify proofs from Ethereum nodes.
use super::{keccak::digest_keccak, rlp_decode_bytes};
use crate::{
    timewave_rlp::{self, alloy_bytes::Bytes},
    timewave_trie::verify::verify_proof,
};
use anyhow::{Context, Ok, Result};
use common::merkle::types::MerkleVerifiable;
use num_bigint::BigUint;
use nybbles::Nibbles;
use serde::{Deserialize, Serialize};
use tracing::info;

/// Represents an Ethereum Merkle proof with its associated data.
///
/// This struct contains all the necessary components to verify a Merkle proof
/// in the Ethereum state trie, storage trie, or receipt trie. The proof includes
/// the path from the leaf node to the root, the key being proven, and the RLP-encoded
/// value at the leaf node.
///
/// # Fields
/// * `proof` - The list of proof nodes in the Merkle path
/// * `key` - The key being proven (e.g., account address, storage key, or receipt index)
/// * `value` - The RLP-encoded value being proven
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EthereumStorageProof {
    /// The list of proof nodes in the Merkle path
    pub proof: Vec<Vec<u8>>,
    /// The key being proven
    pub key: Vec<u8>,
    /// The RLP-encoded value being proven
    pub value: Vec<u8>,
}

impl EthereumStorageProof {
    /// Creates a new Ethereum Merkle proof.
    ///
    /// # Arguments
    /// * `proof` - The list of proof nodes in the Merkle path
    /// * `key` - The key being proven
    /// * `value` - The RLP-encoded value being proven
    ///
    /// # Note
    /// The key is automatically hashed using keccak256 before being stored
    pub fn new(proof: Vec<Vec<u8>>, key: Vec<u8>, value: Vec<u8>) -> Self {
        Self {
            proof,
            key: digest_keccak(&key).to_vec(),
            value,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EthereumAccountProof {
    pub proof: Vec<Vec<u8>>,
    pub address: Vec<u8>,
    pub value: Vec<u8>,
}

impl EthereumAccountProof {
    pub fn new(proof: Vec<Vec<u8>>, address: Vec<u8>, value: Vec<u8>) -> Self {
        Self {
            proof,
            address,
            value,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EthereumCombinedProof {
    pub storage_proof: EthereumStorageProof,
    pub account_proof: EthereumAccountProof,
}

impl EthereumCombinedProof {
    pub fn new(storage_proof: EthereumStorageProof, account_proof: EthereumAccountProof) -> Self {
        Self {
            storage_proof,
            account_proof,
        }
    }
}

/// Represents a raw Ethereum Merkle proof before key hashing.
///
/// This struct is used as an intermediate representation when constructing
/// Ethereum Merkle proofs, before the key is hashed using keccak256.
///
/// # Fields
/// * `proof` - The list of proof nodes in the Merkle path
/// * `key` - The original key before hashing
/// * `value` - The RLP-encoded value being proven
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EthereumReceiptProof {
    /// The list of proof nodes in the Merkle path
    pub proof: Vec<Vec<u8>>,
    /// The original key before hashing
    pub key: Vec<u8>,
    /// The RLP-encoded value being proven
    pub value: Vec<u8>,
}

impl EthereumReceiptProof {
    pub fn new(proof: Vec<Vec<u8>>, key: Vec<u8>, value: Vec<u8>) -> Self {
        Self { proof, key, value }
    }
}

impl From<EthereumReceiptProof> for EthereumStorageProof {
    /// Converts a raw proof into a regular Ethereum Merkle proof.
    ///
    /// # Arguments
    /// * `proof` - The raw proof to convert
    ///
    /// # Returns
    /// A new `EthereumMerkleProof` with the key hashed using keccak256
    fn from(proof: EthereumReceiptProof) -> Self {
        Self {
            proof: proof.proof,
            key: proof.key,
            value: proof.value,
        }
    }
}

/// Implementation of Merkle proof verification for Ethereum proofs.
///
/// This implementation verifies proofs against the Ethereum state trie,
/// storage trie, or receipt trie.
impl MerkleVerifiable for EthereumStorageProof {
    /// Verifies the proof against the expected Merkle root.
    ///
    /// # Arguments
    /// * `root` - The expected Merkle root to verify against
    ///
    /// # Returns
    /// A boolean indicating whether the proof is valid for the given root
    ///
    /// # Note
    /// The verification process:
    /// 1. Constructs the trie from the proof nodes and computes the root hash
    /// 2. Verifies that the leaf node contains the expected key-value pair
    /// 3. Checks that the root hash matches the expected root
    fn verify(&self, root: &[u8]) -> Result<bool> {
        let proof_nodes: Vec<Bytes> = self
            .proof
            .iter()
            .map(|node| Bytes::copy_from_slice(node))
            .collect();

        let leaf_node_decoded: Vec<timewave_rlp::Bytes> = rlp_decode_bytes(
            proof_nodes
                .to_vec()
                .last()
                .context("Failed to extract leaf node from proof")?,
        )?;

        let stored_value = leaf_node_decoded
            .last()
            .context("Failed to get stored value from leaf")?
            .to_vec();

        if stored_value != self.value {
            info!("Value mismatch!");
            info!("Expected value: {:?}", self.value);
            info!("Stored value: {:?}", stored_value);
            return Ok(false);
        }
        let key = Nibbles::unpack(&self.key);

        let result = verify_proof(
            &root.try_into()?,
            key,
            Some(self.value.to_vec()),
            proof_nodes.iter(),
        );

        match result {
            std::result::Result::Ok(_) => Ok(true),
            Err(e) => {
                anyhow::bail!("Proof verification failed: {:?}", e);
            }
        }
    }
}

impl MerkleVerifiable for EthereumAccountProof {
    fn verify(&self, root: &[u8]) -> Result<bool> {
        let proof_nodes: Vec<Bytes> = self
            .proof
            .iter()
            .map(|node| Bytes::copy_from_slice(node))
            .collect();

        let leaf_node_decoded: Vec<timewave_rlp::Bytes> = rlp_decode_bytes(
            proof_nodes
                .to_vec()
                .last()
                .context("Failed to extract leaf node from proof")?,
        )?;

        let stored_value = leaf_node_decoded
            .last()
            .context("Failed to get stored value from leaf")?
            .to_vec();

        if stored_value != self.value {
            return Ok(false);
        }

        let key = Nibbles::unpack(&digest_keccak(&self.address));

        let result = verify_proof(
            &root.try_into()?,
            key,
            Some(self.value.to_vec()),
            proof_nodes.iter(),
        );

        match result {
            std::result::Result::Ok(_) => Ok(true),
            Err(e) => {
                anyhow::bail!("Proof verification failed: {:?}", e);
            }
        }
    }
}

impl MerkleVerifiable for EthereumCombinedProof {
    fn verify(&self, root: &[u8]) -> Result<bool> {
        let storage_proof = self.storage_proof.verify(&self.account_proof.value)?;
        let account_proof = self.account_proof.verify(root)?;
        Ok(storage_proof && account_proof)
    }
}

/// Represents an Ethereum account in the state trie.
///
/// This struct contains the essential data for an Ethereum account, including
/// its nonce, balance, storage root, and code hash. These fields are used to
/// verify the account's state in the Ethereum state trie.
#[derive(Debug, Clone)]
pub struct EthereumAccount {
    /// The number of transactions sent from this account
    pub nonce: u64,
    /// The account's balance in wei
    pub balance: BigUint,
    /// The root hash of the account's storage trie
    pub storage_root: Vec<u8>,
    /// The hash of the account's contract code (or empty if not a contract)
    pub code_hash: Vec<u8>,
}

impl EthereumAccount {
    /// Creates a new Ethereum account with the specified fields.
    ///
    /// # Arguments
    /// * `nonce` - The account's nonce (number of transactions sent)
    /// * `balance` - The account's balance in wei
    /// * `storage_root` - The root hash of the account's storage trie
    /// * `code_hash` - The hash of the account's contract code
    ///
    /// # Returns
    /// A new `EthereumAccount` instance
    pub fn new(nonce: u64, balance: BigUint, storage_root: Vec<u8>, code_hash: Vec<u8>) -> Self {
        Self {
            nonce,
            balance,
            storage_root,
            code_hash,
        }
    }
}
