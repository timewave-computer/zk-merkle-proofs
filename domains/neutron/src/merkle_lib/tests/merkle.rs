#[cfg(test)]
#[cfg(feature = "no-sp1")]
mod tests {
    use common::merkle::types::MerkleVerifiable;

    use crate::merkle_lib::{
        tests::test_vector::TEST_VECTOR_NEUTRON_STORAGE_PROOF, types::NeutronMerkleProof,
    };
    #[tokio::test]
    async fn test_verify_storage_proof_single() {
        use crate::merkle_lib::tests::test_vector::read_test_vector_merkle_root;
        let proof: NeutronMerkleProof =
            serde_json::from_slice(&TEST_VECTOR_NEUTRON_STORAGE_PROOF).unwrap();
        #[allow(deprecated)]
        proof.verify(&base64::decode(read_test_vector_merkle_root()).unwrap());
    }
}
