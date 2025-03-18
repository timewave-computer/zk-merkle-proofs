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
        let proof: NeutronMerkleProof =
            serde_json::from_slice(&TEST_VECTOR_NEUTRON_STORAGE_PROOF).unwrap();
        println!(
            "Value Decoded: {:?}",
            &String::from_utf8_lossy(&proof.value)
        );
        #[allow(deprecated)]
        proof.verify(&base64::decode(TEST_VECTOR_NEUTRON_ROOT).unwrap());
    }

    #[cfg(all(feature = "no-sp1", feature = "tests-online"))]
    #[tokio::test]
    async fn test_get_neutron_contract_value_from_dict() {
        use common::merkle::types::MerkleProver;

        use crate::{
            keys::NeutronKey,
            merkle_lib::{tests::test_vector::read_rpc_url, types::MerkleProverNeutron},
        };

        let contract_address = "neutron1nyuryl5u5z04dx4zsqgvsuw7fe8gl2f77yufynauuhklnnmnjncqcls0tj";
        let height: u64 = 918;
        let initial_address = "neutron1m9l358xunhhwds0568za49mzhvuxx9ux8xafx2";
        let neutron_key: NeutronKey =
            NeutronKey::new_wasm_account_mapping(b"store", initial_address, contract_address);
        let rpc_url = read_rpc_url();
        let prover = MerkleProverNeutron { rpc_url };
        let _proofs = prover
            .get_merkle_proof_from_rpc(&neutron_key.serialize(), "", height)
            .await;
    }

    #[cfg(all(feature = "no-sp1", feature = "tests-online"))]
    // first verifies account state, then a single storage proof
    // currently the variables need to be manually set before running the test
    #[tokio::test]
    pub async fn get_neutron_test_vector_bank_store_supply() {
        use common::merkle::types::MerkleProver;

        use crate::{
            keys::NeutronKey,
            merkle_lib::{
                tests::test_vector::{read_rpc_url, read_test_vector_height},
                types::MerkleProverNeutron,
            },
        };

        let rpc_url = read_rpc_url();
        let prover = MerkleProverNeutron { rpc_url };
        let neutron_key = NeutronKey::new_bank_total_supply("untrn");
        let _proofs = prover
            .get_merkle_proof_from_rpc(&neutron_key.serialize(), "", read_test_vector_height())
            .await;
    }
}
