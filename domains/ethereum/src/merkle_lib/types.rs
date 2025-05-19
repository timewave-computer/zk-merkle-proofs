//! Ethereum Merkle proof types and implementations.
//!
//! This module provides types and implementations for working with Ethereum Merkle proofs,
//! including account proofs, storage proofs, and receipt proofs. It implements the common
//! Merkle proof traits for Ethereum-specific data structures and provides functionality
//! to fetch and verify proofs from Ethereum nodes.
use super::{digest_keccak, rlp_decode_bytes};
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

/// Represents different types of Ethereum Merkle proofs.
///
/// This enum encapsulates the various types of Merkle proofs that can be used
/// in Ethereum, including account proofs, storage proofs, combined proofs, and
/// receipt proofs. Each variant contains the specific proof data needed for
/// verification.
///
/// # Variants
///
/// * `Account(EthereumAccountProof)` - A proof for verifying an account's state in the state trie
/// * `Storage(EthereumStorageProof)` - A proof for verifying a storage value in an account's storage trie
/// * `Combined(EthereumCombinedProof)` - A combined proof containing both account and storage proofs
/// * `Receipt(EthereumReceiptProof)` - A proof for verifying a transaction receipt in the receipt trie

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EthereumProofType {
    Account(EthereumAccountProof),
    Storage(EthereumStorageProof),
    Combined(EthereumCombinedProof),
    Receipt(EthereumReceiptProof),
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

/// Represents a combined Ethereum Merkle proof containing both account and storage proofs.
///
/// This struct combines an account proof and a storage proof to allow for verification
/// of both account state and storage state in a single operation. This is commonly used
/// when verifying storage values for a specific account.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EthereumCombinedProof {
    /// The proof for the account's existence and state
    pub account_proof: EthereumAccountProof,
    /// The proof for a specific storage value in the account's storage trie
    pub storage_proof: EthereumStorageProof,
}

impl EthereumCombinedProof {
    /// Creates a new combined proof from an account proof and storage proof.
    ///
    /// # Arguments
    /// * `account_proof` - The proof for the account's existence and state
    /// * `storage_proof` - The proof for a specific storage value
    ///
    /// # Returns
    /// A new `EthereumCombinedProof` instance
    pub fn new(account_proof: EthereumAccountProof, storage_proof: EthereumStorageProof) -> Self {
        Self {
            account_proof,
            storage_proof,
        }
    }
}

/// Represents an Ethereum storage Merkle proof.
///
/// This struct contains the necessary components to verify a Merkle proof for a storage
/// value in an Ethereum account's storage trie. The proof includes the path from the
/// leaf node to the root, the storage key being proven, and the RLP-encoded value
/// at the leaf node.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EthereumStorageProof {
    /// The list of proof nodes in the Merkle path from leaf to root
    pub proof: Vec<Vec<u8>>,
    /// The storage key being proven (keccak256 hash of the original key)
    pub key: Vec<u8>,
    /// The RLP-encoded value being proven
    pub value: Vec<u8>,
}

impl EthereumStorageProof {
    /// Creates a new Ethereum storage Merkle proof.
    ///
    /// # Arguments
    /// * `proof` - The list of proof nodes in the Merkle path
    /// * `key` - The storage key being proven
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

/// Represents an Ethereum account Merkle proof.
///
/// This struct contains the necessary components to verify a Merkle proof for an
/// Ethereum account in the state trie. The proof includes the path from the leaf
/// node to the root, the account address being proven, and the RLP-encoded account
/// data at the leaf node.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EthereumAccountProof {
    /// The list of proof nodes in the Merkle path from leaf to root
    pub proof: Vec<Vec<u8>>,
    /// The account address being proven
    pub address: Vec<u8>,
    /// The RLP-encoded account data being proven
    pub value: Vec<u8>,
}

impl EthereumAccountProof {
    /// Creates a new Ethereum account Merkle proof.
    ///
    /// # Arguments
    /// * `proof` - The list of proof nodes in the Merkle path
    /// * `address` - The account address being proven
    /// * `value` - The RLP-encoded account data being proven
    ///
    /// # Returns
    /// A new `EthereumAccountProof` instance
    pub fn new(proof: Vec<Vec<u8>>, address: Vec<u8>, value: Vec<u8>) -> Self {
        Self {
            proof,
            address,
            value,
        }
    }
}

/// Represents a raw Ethereum receipt Merkle proof before key hashing.
///
/// This struct is used as an intermediate representation when constructing
/// Ethereum receipt Merkle proofs, before the key is hashed using keccak256.
/// It contains the proof path, the original key, and the RLP-encoded receipt data.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EthereumReceiptProof {
    /// The list of proof nodes in the Merkle path from leaf to root
    pub proof: Vec<Vec<u8>>,
    /// The original key before hashing (typically the transaction index)
    pub key: Vec<u8>,
    /// The RLP-encoded receipt data being proven
    pub value: Vec<u8>,
}

impl EthereumReceiptProof {
    /// Creates a new raw Ethereum receipt Merkle proof.
    ///
    /// # Arguments
    /// * `proof` - The list of proof nodes in the Merkle path
    /// * `key` - The original key before hashing
    /// * `value` - The RLP-encoded receipt data being proven
    ///
    /// # Returns
    /// A new `EthereumReceiptProof` instance
    pub fn new(proof: Vec<Vec<u8>>, key: Vec<u8>, value: Vec<u8>) -> Self {
        Self { proof, key, value }
    }
}

impl From<EthereumReceiptProof> for EthereumStorageProof {
    /// Converts a raw receipt proof into a regular Ethereum storage proof.
    ///
    /// This implementation preserves the proof nodes and value as-is, while
    /// using the original key directly. This is used when converting receipt
    /// proofs to storage proofs for verification purposes.
    ///
    /// # Arguments
    /// * `proof` - The raw receipt proof to convert
    ///
    /// # Returns
    /// A new `EthereumStorageProof` with the same proof nodes and value
    fn from(proof: EthereumReceiptProof) -> Self {
        Self {
            proof: proof.proof,
            key: proof.key,
            value: proof.value,
        }
    }
}

impl From<&EthereumReceiptProof> for EthereumStorageProof {
    fn from(proof: &EthereumReceiptProof) -> Self {
        Self {
            proof: proof.proof.clone(),
            key: proof.key.clone(),
            value: proof.value.clone(),
        }
    }
}

/// Implementation of Merkle proof verification for Ethereum storage proofs.
///
/// This implementation verifies proofs against the Ethereum storage trie by:
/// 1. Decoding the proof nodes and checking the leaf node value
/// 2. Verifying the proof path using the keccak256-hashed storage key
/// 3. Ensuring the computed root matches the expected root
impl MerkleVerifiable for EthereumStorageProof {
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

/// Implementation of Merkle proof verification for Ethereum account proofs.
///
/// This implementation verifies proofs against the Ethereum state trie by:
/// 1. Decoding the proof nodes and checking the leaf node value
/// 2. Verifying the proof path using the keccak256-hashed account address
/// 3. Ensuring the computed root matches the expected root
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

impl MerkleVerifiable for EthereumReceiptProof {
    fn verify(&self, root: &[u8]) -> Result<bool> {
        let storage_proof: EthereumStorageProof = self.into();
        storage_proof.verify(root)
    }
}

/// Implementation of Merkle proof verification for combined Ethereum proofs.
///
/// This implementation verifies both account and storage proofs in sequence:
/// 1. First verifies the account proof against the state root
/// 2. Then verifies the storage proof against the account's storage root
/// 3. Returns true only if both verifications succeed
impl MerkleVerifiable for EthereumCombinedProof {
    fn verify(&self, root: &[u8]) -> Result<bool> {
        let storage_proof = self.storage_proof.verify(&self.account_proof.value)?;
        let account_proof = self.account_proof.verify(root)?;
        Ok(storage_proof && account_proof)
    }
}
