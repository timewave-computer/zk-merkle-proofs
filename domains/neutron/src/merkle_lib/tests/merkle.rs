#[cfg(test)]
#[cfg(feature = "no-sp1")]
mod tests {
    use crate::merkle_lib::{
        tests::test_vector::{TEST_VECTOR_NEUTRON_ROOT, TEST_VECTOR_NEUTRON_STORAGE_PROOF},
        types::NeutronMerkleProof,
    };
    use common::merkle::types::MerkleVerifiable;
    #[tokio::test]
    async fn test_verify_storage_proof_single() {
        use crate::merkle_lib::tests::test_vector::read_test_vector_merkle_root;
        let proof: NeutronMerkleProof =
            serde_json::from_slice(&TEST_VECTOR_NEUTRON_STORAGE_PROOF).unwrap();
        println!(
            "Value Decoded: {:?}",
            &String::from_utf8_lossy(&proof.value)
        );
        #[allow(deprecated)]
        proof.verify(&base64::decode(TEST_VECTOR_NEUTRON_ROOT).unwrap());
    }
}
