#[cfg(test)]
mod tests {
    use crate::merkle_lib::{
        tests::defaults::constants::{
            get_latest_root_and_height, get_test_vector_neutron_storage_proof,
            read_pion_1_default_account_address, TEST_VECTOR_NEUTRON_ROOT,
        },
        types::Ics23MerkleProof,
    };
    use base64::Engine;
    use common::merkle::types::{MerkleClient, MerkleVerifiable};
    #[tokio::test]
    async fn test_verify_storage_proof_single() {
        let proof: Ics23MerkleProof =
            serde_json::from_slice(&get_test_vector_neutron_storage_proof()).unwrap();
        assert!(proof
            .verify(
                &base64::engine::general_purpose::STANDARD
                    .decode(TEST_VECTOR_NEUTRON_ROOT)
                    .unwrap(),
            )
            .unwrap());
    }

    #[tokio::test]
    async fn test_get_neutron_wasm_store_dictionary_merkle_proof() {
        use crate::{
            keys::Ics23Key, merkle_lib::tests::defaults::constants::read_rpc_url,
            rpc::Ics23MerkleRpcClient,
        };
        let contract_address = "neutron1xlklun3vpf7ts08mm79tyyllyezles7mpp3np5a4ueadgfz9ndns350qw2";
        let initial_address = &read_pion_1_default_account_address();
        let neutron_key: Ics23Key =
            Ics23Key::new_wasm_account_mapping(b"store", initial_address, contract_address);
        let rpc_url = read_rpc_url();
        let prover = Ics23MerkleRpcClient { rpc_url };
        let (root, height) = get_latest_root_and_height().await;
        let proofs = prover
            .get_proof(&neutron_key.to_string(), "", height)
            .await
            .unwrap();
        let neutron_proof: Ics23MerkleProof = serde_json::from_slice(&proofs).unwrap();
        assert!(neutron_proof.verify(&root).unwrap());
    }

    // first verifies account state, then a single storage proof
    // currently the variables need to be manually set before running the test
    #[tokio::test]
    pub async fn test_get_neutron_bank_store_supply_merkle_proof() {
        use crate::{
            keys::Ics23Key, merkle_lib::tests::defaults::constants::read_rpc_url,
            rpc::Ics23MerkleRpcClient,
        };
        let rpc_url = read_rpc_url();
        let prover = Ics23MerkleRpcClient { rpc_url };
        let neutron_key = Ics23Key::new_bank_total_supply("untrn");
        let (root, height) = get_latest_root_and_height().await;
        let proofs = prover
            .get_proof(&neutron_key.to_string(), "", height)
            .await
            .unwrap();
        let neutron_proof: Ics23MerkleProof = serde_json::from_slice(&proofs).unwrap();
        assert!(neutron_proof.verify(&root).unwrap());
    }

    #[tokio::test]
    pub async fn test_get_neutron_bank_store_balance_merkle_proof() {
        use crate::{
            keys::Ics23Key,
            merkle_lib::tests::defaults::constants::{get_latest_root_and_height, read_rpc_url},
            rpc::Ics23MerkleRpcClient,
        };

        let rpc_url = read_rpc_url();
        let prover = Ics23MerkleRpcClient { rpc_url };
        let neutron_key = Ics23Key::new_bank_account_balance(
            "untrn",
            "neutron1m9l358xunhhwds0568za49mzhvuxx9ux8xafx2",
        );
        let (root, height) = get_latest_root_and_height().await;
        let proofs = prover
            .get_proof(&neutron_key.to_string(), "", height)
            .await
            .unwrap();
        let neutron_proof: Ics23MerkleProof = serde_json::from_slice(&proofs).unwrap();
        assert!(neutron_proof.verify(&root).unwrap());
    }
}
