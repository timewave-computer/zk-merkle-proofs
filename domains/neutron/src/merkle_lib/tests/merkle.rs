#[cfg(test)]
mod tests {
    use crate::merkle_lib::{
        tests::defaults::{
            get_test_vector_neutron_storage_proof, read_pion_1_default_account_address,
            TEST_VECTOR_NEUTRON_ROOT,
        },
        types::NeutronMerkleProof,
    };
    use base64::Engine;
    use common::merkle::types::{MerkleRpcClient, MerkleVerifiable};
    #[tokio::test]
    async fn test_verify_storage_proof_single() {
        let proof: NeutronMerkleProof =
            serde_json::from_slice(&get_test_vector_neutron_storage_proof()).unwrap();
        println!(
            "Value Decoded: {:?}",
            &String::from_utf8_lossy(&proof.value)
        );
        assert!(proof.verify(
            &base64::engine::general_purpose::STANDARD
                .decode(TEST_VECTOR_NEUTRON_ROOT)
                .unwrap(),
        ));
    }

    #[tokio::test]
    async fn test_get_neutron_wasm_store_dictionary_merkle_proof() {
        use crate::{
            keys::NeutronKey,
            merkle_lib::tests::defaults::{
                read_rpc_url, read_test_vector_height, read_test_vector_merkle_root,
            },
            rpc::NeutronMerkleRpcClient,
        };
        let contract_address = "neutron1xlklun3vpf7ts08mm79tyyllyezles7mpp3np5a4ueadgfz9ndns350qw2";
        let initial_address = &read_pion_1_default_account_address();
        let neutron_key: NeutronKey =
            NeutronKey::new_wasm_account_mapping(b"store", initial_address, contract_address);
        let rpc_url = read_rpc_url();
        let prover = NeutronMerkleRpcClient { rpc_url };
        let proofs = prover
            .get_proof(&neutron_key.serialize(), "", read_test_vector_height())
            .await;
        let neutron_proof: NeutronMerkleProof = serde_json::from_slice(&proofs).unwrap();
        assert!(neutron_proof.verify(
            &base64::engine::general_purpose::STANDARD
                .decode(read_test_vector_merkle_root())
                .unwrap(),
        ));
    }

    // first verifies account state, then a single storage proof
    // currently the variables need to be manually set before running the test
    #[tokio::test]
    pub async fn test_get_neutron_bank_store_supply_merkle_proof() {
        use crate::{
            keys::NeutronKey,
            merkle_lib::tests::defaults::{
                read_rpc_url, read_test_vector_height, read_test_vector_merkle_root,
            },
            rpc::NeutronMerkleRpcClient,
        };
        let rpc_url = read_rpc_url();
        let prover = NeutronMerkleRpcClient { rpc_url };
        let neutron_key = NeutronKey::new_bank_total_supply("untrn");
        let proofs = prover
            .get_proof(&neutron_key.serialize(), "", read_test_vector_height())
            .await;
        let neutron_proof: NeutronMerkleProof = serde_json::from_slice(&proofs).unwrap();
        assert!(neutron_proof.verify(
            &base64::engine::general_purpose::STANDARD
                .decode(read_test_vector_merkle_root())
                .unwrap(),
        ));
    }

    #[tokio::test]
    pub async fn test_get_neutron_bank_store_balance_merkle_proof() {
        use crate::{
            keys::NeutronKey,
            merkle_lib::tests::defaults::{
                read_rpc_url, read_test_vector_height, read_test_vector_merkle_root,
            },
            rpc::NeutronMerkleRpcClient,
        };

        let rpc_url = read_rpc_url();
        let prover = NeutronMerkleRpcClient { rpc_url };
        let neutron_key = NeutronKey::new_bank_account_balance(
            "untrn",
            "neutron1m9l358xunhhwds0568za49mzhvuxx9ux8xafx2",
        );
        let proofs = prover
            .get_proof(&neutron_key.serialize(), "", read_test_vector_height())
            .await;
        let neutron_proof: NeutronMerkleProof = serde_json::from_slice(&proofs).unwrap();
        assert!(neutron_proof.verify(
            &base64::engine::general_purpose::STANDARD
                .decode(read_test_vector_merkle_root())
                .unwrap(),
        ));
    }
}
