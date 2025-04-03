#[cfg(test)]
mod tests {
    use crate::{
        keys::Ics23Key,
        merkle_lib::{
            tests::defaults::constants::{
                read_pion_1_default_account_address, read_pion_1_vault_contract_address,
                read_rpc_url, read_test_vector_height, read_test_vector_merkle_root,
            },
            types::Ics23MerkleProof,
        },
        rpc::Ics23MerkleRpcClient,
    };
    use base64::Engine;
    use common::merkle::types::{MerkleClient, MerkleVerifiable};
    #[tokio::test]
    pub async fn test_get_neutron_pion_vault_contract_balance_merkle_proof() {
        let rpc_url: String = read_rpc_url();
        let prover = Ics23MerkleRpcClient { rpc_url };
        let neutron_key = Ics23Key::new_wasm_account_mapping(
            b"balances",
            &read_pion_1_default_account_address(),
            &read_pion_1_vault_contract_address(),
        );
        let proofs = prover
            .get_proof(&neutron_key.serialize(), "", read_test_vector_height())
            .await
            .unwrap();
        let neutron_proof: Ics23MerkleProof = serde_json::from_slice(&proofs).unwrap();
        assert!(neutron_proof
            .verify(
                &base64::engine::general_purpose::STANDARD
                    .decode(read_test_vector_merkle_root())
                    .unwrap(),
            )
            .unwrap());
    }

    #[tokio::test]
    pub async fn test_get_neutron_pion_vault_shares_merkle_proof() {
        let rpc_url = read_rpc_url();
        let prover = Ics23MerkleRpcClient { rpc_url };
        let neutron_key =
            Ics23Key::new_wasm_stored_value("shares", &read_pion_1_vault_contract_address());
        let proofs = prover
            .get_proof(&neutron_key.serialize(), "", read_test_vector_height())
            .await
            .unwrap();
        let neutron_proof: Ics23MerkleProof = serde_json::from_slice(&proofs).unwrap();
        assert!(neutron_proof
            .verify(
                &base64::engine::general_purpose::STANDARD
                    .decode(read_test_vector_merkle_root())
                    .unwrap(),
            )
            .unwrap());
    }
}
