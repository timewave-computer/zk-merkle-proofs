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

pub trait RlpDecodable {
    fn rlp_decode(rlp: &[u8]) -> Result<Self>
    where
        Self: Sized;
}

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
/// * `Simple(EthereumSimpleProof)` - A simplified proof format that combines multiple proofs into a single structure
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EthereumProofType {
    /// A proof for verifying an account's state in the state trie
    Account(EthereumAccountProof),
    /// A proof for verifying a storage value in an account's storage trie
    Storage(EthereumStorageProof),
    /// A combined proof containing both account and storage proofs
    Combined(EthereumCombinedProof),
    /// A proof for verifying a transaction receipt in the receipt trie
    Receipt(EthereumReceiptProof),
    /// A simplified proof format that combines multiple proofs into a single structure
    Simple(EthereumSimpleProof),
}

impl MerkleVerifiable for EthereumProofType {
    fn verify(&self, root: &[u8]) -> Result<bool> {
        // Match on the proof type and verify
        match self {
            EthereumProofType::Simple(simple_proof) => Ok(simple_proof.verify(root)?),
            EthereumProofType::Account(account_proof) => Ok(account_proof.verify(root)?),
            _ => {
                panic!("Unsupported EthereumProofType: The MVP only supports SimpleProof and AccountProof");
            }
        }
    }
}

/// Represents a simplified Ethereum Merkle proof that combines multiple proofs into a single structure.
///
/// This struct provides a flattened representation of Ethereum proofs, combining proof nodes,
/// keys, and values into single vectors with length prefixes. This format is useful for
/// serialization and transmission of proofs.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EthereumSimpleProof {
    /// The combined proof nodes with length prefixes
    pub proof: Vec<Vec<u8>>,
    /// The combined keys with length prefixes
    pub key: Vec<u8>,
    /// The combined values with length prefixes
    pub value: Vec<u8>,
}

impl EthereumSimpleProof {
    /// Creates a new simplified Ethereum proof.
    ///
    /// # Arguments
    /// * `proof` - The combined proof nodes with length prefixes
    /// * `key` - The combined keys with length prefixes
    /// * `value` - The combined values with length prefixes
    ///
    /// # Returns
    /// A new `EthereumSimpleProof` instance
    pub fn new(proof: Vec<Vec<u8>>, key: Vec<u8>, value: Vec<u8>) -> Self {
        Self { proof, key, value }
    }

    /// Creates a simplified proof from a combined proof.
    ///
    /// This method takes a combined proof containing both account and storage proofs
    /// and converts it into a simplified format with length-prefixed components.
    ///
    /// # Arguments
    /// * `combined_proof` - The combined proof to convert
    ///
    /// # Returns
    /// A new `EthereumSimpleProof` instance
    pub fn from_combined_proof(combined_proof: EthereumCombinedProof) -> Self {
        let mut combined_nodes: Vec<Vec<u8>> = Vec::new();
        let account_proof_len = combined_proof.account_proof.proof.len();

        // Add length information for account proof
        // We assume proof length will never exceed 65,535 bytes (u16::MAX)
        combined_nodes.push((account_proof_len as u16).to_be_bytes().to_vec());

        // Add the actual proof nodes
        combined_nodes.extend(combined_proof.account_proof.proof.clone());
        combined_nodes.extend(combined_proof.storage_proof.proof.clone());

        let mut combined_key: Vec<u8> = Vec::new();
        // Declare the keys
        let account_key = combined_proof.account_proof.address.clone();
        let storage_key = combined_proof.storage_proof.key.clone();

        // Add key length information (using u16 to be consistent with node lengths)
        // We assume key length will never exceed 65,535 bytes (u16::MAX)
        let account_key_len = account_key.len() as u16;
        combined_key.extend(account_key_len.to_be_bytes().to_vec());

        // combine the address and storage proof nodes into a single, flattened proof
        combined_key.extend(account_key);
        combined_key.extend(storage_key);

        // Create combined values with length information
        let mut combined_values: Vec<u8> = Vec::new();
        let account_value = combined_proof.account_proof.value;
        let storage_value = combined_proof.storage_proof.value;

        // Add length information for account value
        let account_value_len = account_value.len() as u16;
        combined_values.extend(account_value_len.to_be_bytes().to_vec());
        combined_values.extend(account_value);
        combined_values.extend(storage_value);

        Self {
            proof: combined_nodes,
            key: combined_key,
            value: combined_values,
        }
    }
}

/// Implementation of Merkle proof verification for simplified Ethereum proofs.
///
/// This implementation verifies proofs by:
/// 1. Extracting length-prefixed components from the combined structures
/// 2. Verifying the account proof against the state root
/// 3. Verifying the storage proof against the account's storage root
/// 4. Returns true only if both verifications succeed
impl MerkleVerifiable for EthereumSimpleProof {
    fn verify(&self, root: &[u8]) -> Result<bool> {
        let combined_nodes = &self.proof;
        let combined_key = &self.key;
        let combined_values = &self.value;

        // Extract lengths from the combined structures
        let account_proof_len =
            u16::from_be_bytes([combined_nodes[0][0], combined_nodes[0][1]]) as usize;
        let account_key_len = u16::from_be_bytes([combined_key[0], combined_key[1]]) as usize;
        let account_value_len =
            u16::from_be_bytes([combined_values[0], combined_values[1]]) as usize;

        // Skip the length nodes when getting the actual proof nodes
        let account_proof_nodes = combined_nodes[1..1 + account_proof_len].to_vec();
        let storage_proof_nodes = combined_nodes[1 + account_proof_len..].to_vec();

        // Skip the length bytes when getting the actual key parts
        let account_key_part = combined_key[2..2 + account_key_len].to_vec();
        let storage_key_part = combined_key[2 + account_key_len..].to_vec();

        // Skip the length bytes when getting the actual value parts
        let account_value_part = combined_values[2..2 + account_value_len].to_vec();
        let storage_value_part = combined_values[2 + account_value_len..].to_vec();

        // Assert that the storage proof is under the storage root used in the account proof
        let account_decoded = EthereumAccount::rlp_decode(&account_value_part).unwrap();

        let account_proof = EthereumAccountProof::new(
            account_proof_nodes.clone(),
            account_key_part,
            account_value_part,
        );

        let account_result = account_proof.verify(root).unwrap();
        if !account_result {
            return Ok(false);
        }

        let storage_proof = EthereumStorageProof::new(
            storage_proof_nodes.clone(),
            storage_key_part,
            storage_value_part,
        );

        let storage_result = storage_proof.verify(&account_decoded.storage_root).unwrap();

        if !storage_result {
            return Ok(false);
        }

        Ok(true)
    }
}
/// Represents an Ethereum account in the state trie.
///
/// This struct contains the essential data for an Ethereum account, including
/// its nonce, balance, storage root, and code hash. These fields are used to
/// verify the account's state in the Ethereum state trie.
#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl RlpDecodable for EthereumAccount {
    fn rlp_decode(rlp: &[u8]) -> Result<Self> {
        let account_rlp_bytes = rlp_decode_bytes(rlp)?;
        let nonce = if let Some(nonce_bytes) = account_rlp_bytes.first() {
            if nonce_bytes.is_empty() {
                0u64
            } else {
                u64::from_be_bytes({
                    let mut padded = [0u8; 8];
                    let nonce_slice = nonce_bytes.as_ref();
                    let start = 8 - nonce_slice.len();
                    padded[start..].copy_from_slice(nonce_slice);
                    padded
                })
            }
        } else {
            0u64
        };
        let balance = BigUint::from_bytes_be(
            account_rlp_bytes
                .get(1)
                .context("Failed to get balance")?
                .as_ref(),
        );

        let storage_root = account_rlp_bytes
            .get(2)
            .context("Failed to get storage root")?
            .to_vec();

        let code_hash = account_rlp_bytes
            .get(3)
            .context("Failed to get code hash")?
            .to_vec();

        Ok(Self::new(nonce, balance, storage_root, code_hash))
    }
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
    pub account_proof: EthereumAccountProof,
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
        Self { proof, key, value }
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
        let key = Nibbles::unpack(&digest_keccak(&self.key));

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

impl MerkleVerifiable for EthereumReceiptProof {
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

/// Implementation of From trait to convert EthereumReceiptProof to EthereumStorageProof.
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
impl From<EthereumReceiptProof> for EthereumStorageProof {
    fn from(proof: EthereumReceiptProof) -> Self {
        Self {
            proof: proof.proof,
            key: proof.key,
            value: proof.value,
        }
    }
}
