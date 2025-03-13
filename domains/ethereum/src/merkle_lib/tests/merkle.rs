#[cfg(test)]
#[cfg(feature = "web")]
mod tests {
    use crate::merkle_lib::test_vector::get_ethereum_test_vector_storage_proof;
    use common::merkle::types::MerkleVerifiable;

    #[tokio::test]
    async fn test_verify_storage_proof_single() {
        let eth_proof = get_ethereum_test_vector_storage_proof().await;
        let leaf = eth_proof.proof.last().unwrap().to_owned();

        let leaf_decoded: Vec<alloy_primitives::Bytes> = alloy_rlp::decode_exact(&leaf).unwrap();
        let value_encoded = leaf_decoded.get(1).unwrap();
        assert_eq!(value_encoded.to_vec(), eth_proof.value);
        eth_proof.verify(&eth_proof.root.to_vec());
    }
}
