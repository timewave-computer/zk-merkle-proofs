//! Ethereum Merkle proof types and implementations.
//!
//! This module provides types and implementations for working with Ethereum Merkle proofs,
//! including account proofs, storage proofs, and receipt proofs. It implements the common
//! Merkle proof traits for Ethereum-specific data structures and provides functionality
//! to fetch and verify proofs from Ethereum nodes.
use super::keccak::digest_keccak;
use crate::{
    timewave_rlp::{self, alloy_bytes::Bytes},
    timewave_trie::verify::{verify_proof, ProofVerificationError},
};
use anyhow::{anyhow, Context, Result};
use common::merkle::types::MerkleVerifiable;
use nybbles::Nibbles;
use std::result::Result as StdResult;
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
/// * `value` - The rlp-decoded value being proven
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct EthereumMerkleProof {
    /// The list of proof nodes in the Merkle path
    pub proof: Vec<Vec<u8>>,
    /// The key being proven
    pub key: Vec<u8>,
    /// The rlp-decoded value being proven
    pub value: Vec<u8>,
}

impl EthereumMerkleProof {
    /// Creates a new Ethereum Merkle proof.
    ///
    /// # Arguments
    /// * `proof` - The list of proof nodes in the Merkle path
    /// * `key` - The key being proven
    /// * `value` - The rlp-decoded value being proven
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
/// * `value` - The rlp-decoded value being proven
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct EthereumRawMerkleProof {
    /// The list of proof nodes in the Merkle path
    pub proof: Vec<Vec<u8>>,
    /// The original key before hashing
    pub key: Vec<u8>,
    /// The RLP-decoded value being proven
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
    /// 1. Constructs the trie from the proof nodes and computes the root hash
    /// 2. Verifies that the leaf node contains the expected key-value pair
    /// 3. Checks that the root hash matches the expected root
    fn verify(&self, root: &[u8]) -> Result<bool> {
        let proof_nodes: Vec<Bytes> = self
            .proof
            .iter()
            .map(|node| Bytes::copy_from_slice(node))
            .collect();
        let leaf_node_decoded: Vec<timewave_rlp::Bytes> = decode_rlp_bytes(
            proof_nodes
                .to_vec()
                .last()
                .ok_or_else(|| ProofVerificationError::ValueMismatch {
                    path: Nibbles::unpack(&self.key),
                    got: None,
                    expected: Some(Bytes::copy_from_slice(&self.value)),
                })
                .map_err(|e| anyhow::anyhow!("Failed to extract leaf node from proof: {:?}", e))?,
        )?;
        let stored_value = leaf_node_decoded
            .last()
            .ok_or_else(|| ProofVerificationError::ValueMismatch {
                path: Nibbles::unpack(&self.key),
                got: None,
                expected: Some(Bytes::copy_from_slice(&self.value)),
            })
            .map_err(|e| anyhow::anyhow!("Failed to get stored value from leaf: {:?}", e))?
            .to_vec();
        if stored_value != self.value {
            info!("Value mismatch!");
            info!("Expected value: {:?}", self.value);
            info!("Stored value: {:?}", stored_value);
            return Ok(false);
        }
        let key = Nibbles::unpack(&self.key);
        let result = verify_proof(
            &root
                .try_into()
                .map_err(|_| ProofVerificationError::RootMismatch {
                    got: [0; 32],
                    expected: root.try_into().unwrap_or([0; 32]),
                })?,
            key,
            Some(self.value.to_vec()),
            proof_nodes.iter(),
        );
        match result {
            Ok(_) => Ok(true),
            Err(e) => Err(anyhow::anyhow!("Proof verification failed: {:?}", e)),
        }
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
pub fn decode_rlp_bytes(bytes: &[u8]) -> Result<Vec<timewave_rlp::Bytes>> {
    let decoded = timewave_rlp::decode_exact(bytes)
        .map_err(|e| anyhow::anyhow!("Failed to decode RLP bytes: {:?}", e))?;
    Ok(decoded)
}
