use serde::{Deserialize, Serialize};

/// Represents the output of a Merkle proof verification.
/// Contains the necessary components to verify a Merkle proof including the root,
/// key, value, and domain information.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MerkleProofOutput {
    /// The Merkle root hash that commits to the encoded roots
    pub root: Vec<u8>,
    /// The key that was queried in the key-value store
    pub key: Vec<u8>,
    /// The value hash that commits to the encoded values
    pub value: Vec<u8>,
    /// The domain identifier indicating which proving system to use
    pub domain: Domain,
}

/// A trait for types that can generate Merkle proofs from RPC calls.
/// This trait is implemented by different proving systems to fetch proofs
/// from their respective blockchain networks.
pub trait MerkleProver {
    #[allow(async_fn_in_trait)]
    /// Retrieves a Merkle proof for a given key at a specific block height from an RPC endpoint.
    ///
    /// # Arguments
    /// * `key` - The key to generate a proof for
    /// * `address` - The RPC endpoint address
    /// * `height` - The block height at which to generate the proof
    ///
    /// # Returns
    /// A vector of bytes containing the encoded Merkle proof
    async fn get_merkle_proof_from_rpc(&self, key: &str, address: &str, height: u64) -> Vec<u8>;
}

/// A trait for types that can verify Merkle proofs against an expected root.
/// This trait provides the functionality to verify that a proof is valid
/// for a given Merkle root.
pub trait MerkleVerifiable {
    /// Verifies the proof against the expected Merkle root.
    ///
    /// # Arguments
    /// * `expected_root` - The expected Merkle root to verify against
    ///
    /// # Returns
    /// A `MerkleProofOutput` containing the verification result
    fn verify(&self, expected_root: &[u8]) -> MerkleProofOutput;
}

/// Represents the target domain for Merkle proof generation and verification.
/// This enum is used to determine which proving system should be used
/// for a particular blockchain network.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Domain {
    /// Ethereum network domain
    ETHEREUM,
    /// Neutron network domain (currently unsupported)
    NEUTRON,
}
