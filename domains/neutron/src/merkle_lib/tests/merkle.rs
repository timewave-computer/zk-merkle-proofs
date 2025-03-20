#[cfg(test)]
#[cfg(feature = "no-sp1")]
mod tests {
    use crate::merkle_lib::{
        tests::defaults::{
            read_pion_1_default_account_address, TEST_VECTOR_NEUTRON_ROOT,
            TEST_VECTOR_NEUTRON_STORAGE_PROOF,
        },
        types::NeutronMerkleProof,
    };
    use crate::{
        keys::NeutronKey,
        merkle_lib::{
            tests::defaults::{read_rpc_url, read_test_vector_height},
            types::MerkleProverNeutron,
        },
    };
    use common::merkle::types::{MerkleProver, MerkleVerifiable};
    #[tokio::test]
    async fn test_verify_storage_proof_single() {
        let proof: NeutronMerkleProof =
            serde_json::from_slice(&TEST_VECTOR_NEUTRON_STORAGE_PROOF).unwrap();
        println!(
            "Value Decoded: {:?}",
            &String::from_utf8_lossy(&proof.value)
        );
        #[allow(deprecated)]
        proof.verify(&base64::decode(TEST_VECTOR_NEUTRON_ROOT).unwrap());
    }

    #[cfg(feature = "no-sp1")]
    #[tokio::test]
    async fn test_get_neutron_wasm_store_dictionary_merkle_proof() {
        use crate::{
            keys::NeutronKey,
            merkle_lib::{
                tests::defaults::{read_rpc_url, read_test_vector_height},
                types::MerkleProverNeutron,
            },
        };
        let contract_address = "neutron1xlklun3vpf7ts08mm79tyyllyezles7mpp3np5a4ueadgfz9ndns350qw2";
        let initial_address = &read_pion_1_default_account_address();
        let neutron_key: NeutronKey =
            NeutronKey::new_wasm_account_mapping(b"store", initial_address, contract_address);
        let rpc_url = read_rpc_url();
        let prover = MerkleProverNeutron { rpc_url };
        let _proofs = prover
            .get_merkle_proof_from_rpc(&neutron_key.serialize(), "", read_test_vector_height())
            .await;
    }

    #[cfg(feature = "no-sp1")]
    // first verifies account state, then a single storage proof
    // currently the variables need to be manually set before running the test
    #[tokio::test]
    pub async fn test_get_neutron_bank_store_supply_merkle_proof() {
        use crate::{
            keys::NeutronKey,
            merkle_lib::{
                tests::defaults::{
                    read_rpc_url, read_test_vector_height, read_test_vector_merkle_root,
                },
                types::MerkleProverNeutron,
            },
        };
        let rpc_url = read_rpc_url();
        let prover = MerkleProverNeutron { rpc_url };
        let neutron_key = NeutronKey::new_bank_total_supply("untrn");
        let proofs = prover
            .get_merkle_proof_from_rpc(&neutron_key.serialize(), "", read_test_vector_height())
            .await;
        let neutron_proof: NeutronMerkleProof = serde_json::from_slice(&proofs).unwrap();
        neutron_proof.verify(&base64::decode(read_test_vector_merkle_root()).unwrap());
    }

    #[cfg(feature = "no-sp1")]
    #[tokio::test]
    pub async fn test_get_neutron_bank_store_balance_merkle_proof() {
        use crate::merkle_lib::tests::defaults::read_test_vector_merkle_root;

        let rpc_url = read_rpc_url();
        let prover = MerkleProverNeutron { rpc_url };
        let neutron_key = NeutronKey::new_bank_account_balance(
            "untrn",
            "neutron1m9l358xunhhwds0568za49mzhvuxx9ux8xafx2",
        );
        let proofs = prover
            .get_merkle_proof_from_rpc(&neutron_key.serialize(), "", read_test_vector_height())
            .await;
        let neutron_proof: NeutronMerkleProof = serde_json::from_slice(&proofs).unwrap();
        neutron_proof.verify(&base64::decode(read_test_vector_merkle_root()).unwrap());
    }
}
