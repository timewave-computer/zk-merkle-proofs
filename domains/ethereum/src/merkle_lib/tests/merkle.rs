#[cfg(test)]
#[cfg(feature = "no-sp1")]
mod tests {
    use crate::merkle_lib::{
        tests::test_vector::{TEST_VECTOR_ETH_ACCOUNT_PROOF, TEST_VECTOR_ETH_STORAGE_PROOF},
        types::EthereumMerkleProof,
    };
    use common::merkle::types::MerkleVerifiable;

    #[tokio::test]
    async fn test_verify_storage_proof_single() {
        let eth_proof: EthereumMerkleProof =
            serde_json::from_slice(&TEST_VECTOR_ETH_STORAGE_PROOF).unwrap();
        let leaf = eth_proof.proof.last().unwrap().to_owned();
        let leaf_decoded: Vec<alloy_primitives::Bytes> = alloy_rlp::decode_exact(&leaf).unwrap();
        let value_encoded = leaf_decoded.get(1).unwrap();
        assert_eq!(value_encoded.to_vec(), eth_proof.value);
        eth_proof.verify(&eth_proof.root.to_vec());
    }
    #[tokio::test]
    async fn test_verify_account_proof_single() {
        let eth_proof: EthereumMerkleProof =
            serde_json::from_slice(&TEST_VECTOR_ETH_ACCOUNT_PROOF).unwrap();
        eth_proof.verify(&eth_proof.root.to_vec());
    }
}
