#[cfg(test)]
#[cfg(feature = "web")]
mod tests {
    use common::MerkleVerifiable;

    use crate::merkle_lib::test_vector::get_ethereum_test_vector_storage_proof;
    #[tokio::test]
    // first verifies account state, then a single storage proof
    async fn test_verify_storage_proof_single() {
        let eth_proof = get_ethereum_test_vector_storage_proof().await;
        eth_proof.verify(&eth_proof.root.to_vec());
    }
}
