//! Ethereum Merkle proof types and implementations.
//!
//! This module provides types and implementations for working with Ethereum Merkle proofs,
//! including account proofs, storage proofs, and receipt proofs. It implements the common
//! Merkle proof traits for Ethereum-specific data structures and provides functionality
//! to fetch and verify proofs from Ethereum nodes.

use super::keccak::digest_keccak;
use alloy_primitives::{FixedBytes, B256};
use common::merkle::types::MerkleVerifiable;
use eth_trie::{EthTrie, MemoryDB, Trie, DB};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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
pub struct EthereumMerkleProof {
    /// The list of proof nodes in the Merkle path
    pub proof: Vec<Vec<u8>>,
    /// The key being proven
    pub key: Vec<u8>,
    /// The RLP-encoded value being proven
    pub value: Vec<u8>,
}

impl EthereumMerkleProof {
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
pub struct EthereumRawMerkleProof {
    /// The list of proof nodes in the Merkle path
    pub proof: Vec<Vec<u8>>,
    /// The original key before hashing
    pub key: Vec<u8>,
    /// The RLP-encoded value being proven
    pub value: Vec<u8>,
}

impl EthereumRawMerkleProof {
    pub fn new(proof: Vec<Vec<u8>>, key: Vec<u8>, value: Vec<u8>) -> Self {
        Self { proof, key, value }
    }
}

impl From<EthereumRawMerkleProof> for EthereumMerkleProof {
    /// Converts a raw proof into a regular Ethereum Merkle proof.
    ///
    /// # Arguments
    /// * `proof` - The raw proof to convert
    ///
    /// # Returns
    /// A new `EthereumMerkleProof` with the key hashed using keccak256
    fn from(proof: EthereumRawMerkleProof) -> Self {
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
impl MerkleVerifiable for EthereumMerkleProof {
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
    /// 1. Reconstructs the Merkle path using the proof nodes
    /// 2. Verifies that the leaf node contains the expected key-value pair
    /// 3. Checks that the root hash matches the expected root
    fn verify(&self, root: &[u8]) -> bool {
        let root_hash: FixedBytes<32> = FixedBytes::from_slice(root);
        let proof_db = Arc::new(MemoryDB::new(true));

        for node_encoded in &self.proof {
            let hash: B256 = digest_keccak(node_encoded).into();
            proof_db
                .insert(hash.as_slice(), node_encoded.to_vec())
                .expect("Failed to insert proof node!");
        }

        let mut trie = EthTrie::from(proof_db, root_hash).expect("Invalid merkle proof");

        if root_hash != trie.root_hash().unwrap() {
            println!("Root hash mismatch!");
            return false;
        }

        let stored_value = trie
            .verify_proof(root_hash, &self.key.clone(), self.proof.clone())
            .expect("Failed to verify Merkle Proof")
            .expect("Key does not exist!");

        if stored_value != self.value {
            println!("Value mismatch!");
            println!("Expected value: {:?}", self.value);
            println!("Stored value: {:?}", stored_value);
            return false;
        }

        true
    }
}

/// Decodes RLP-encoded bytes into a vector of bytes.
///
/// # Arguments
/// * `bytes` - The RLP-encoded bytes to decode
///
/// # Returns
/// A vector of decoded bytes
///
/// # Panics
/// Panics if the bytes cannot be decoded
#[cfg(feature = "no-zkvm")]
pub fn decode_rlp_bytes(bytes: &[u8]) -> Vec<alloy_primitives::Bytes> {
    let decoded: Vec<alloy_primitives::Bytes> = alloy_rlp::decode_exact(bytes).unwrap();
    decoded
}
