#[cfg(test)]
#[cfg(feature = "web")]
mod tests {
    use common::MerkleVerifiable;
    #[tokio::test]
    async fn test_verify_storage_proof_single() {
        use crate::test_vector::{
            get_neutron_test_vector_bank_store_supply, read_test_vector_merkle_root,
        };
        let proof = get_neutron_test_vector_bank_store_supply().await;
        proof.verify(&base64::decode(read_test_vector_merkle_root()).unwrap());
    }
}
