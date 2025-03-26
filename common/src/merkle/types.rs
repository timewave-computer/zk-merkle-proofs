/// A trait for types that can generate Merkle proofs from RPC calls.
///
/// This trait defines the interface for different proving systems to fetch proofs
/// from their respective blockchain networks. It provides a standardized way to
/// retrieve Merkle proofs for any key at a specific block height.
pub trait MerkleProver {
    #[allow(async_fn_in_trait)]
    /// Retrieves a Merkle proof for a given key at a specific block height.
    ///
    /// # Arguments
    /// * `key` - The key to generate a proof for (e.g., account address, storage key)
    /// * `address` - The RPC endpoint address to fetch the proof from
    /// * `height` - The block height at which to generate the proof
    ///
    /// # Returns
    /// A vector of bytes containing the encoded Merkle proof
    ///
    /// # Note
    /// The exact format of the proof depends on the implementing blockchain network.
    /// The proof should be sufficient to verify the existence and value of the key
    /// in the Merkle tree at the specified block height.
    async fn get_merkle_proof_from_rpc(&self, key: &str, address: &str, height: u64) -> Vec<u8>;
}

/// A trait for types that can verify Merkle proofs against an expected root.
///
/// This trait provides the functionality to verify that a proof is valid
/// for a given Merkle root. It is used to ensure that the proof correctly
/// demonstrates the existence and value of a key in the Merkle tree.
pub trait MerkleVerifiable {
    /// Verifies the proof against the expected Merkle root.
    ///
    /// # Arguments
    /// * `root` - The expected Merkle root to verify against
    ///
    /// # Returns
    /// A boolean indicating whether the proof is valid for the given root
    ///
    /// # Note
    /// The verification process should check that:
    /// 1. The proof nodes form a valid path from the leaf to the root
    /// 2. The leaf node contains the expected key-value pair
    /// 3. The root hash matches the expected root
    fn verify(&self, root: &[u8]) -> bool;
}
