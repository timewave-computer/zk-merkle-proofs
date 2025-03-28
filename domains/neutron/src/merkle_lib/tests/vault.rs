#[cfg(test)]
mod tests {
    use crate::{
        keys::NeutronKey,
        merkle_lib::{
            tests::defaults::{
                read_pion_1_default_account_address, read_pion_1_vault_contract_address,
                read_rpc_url, read_test_vector_height, read_test_vector_merkle_root,
            },
            types::NeutronMerkleProof,
        },
        rpc::NeutronMerkleRpcClient,
    };
    use base64::Engine;
    use common::merkle::types::{MerkleRpcClient, MerkleVerifiable};
    #[tokio::test]
    pub async fn test_get_neutron_pion_vault_contract_balance_merkle_proof() {
        let rpc_url: String = read_rpc_url();
        let prover = NeutronMerkleRpcClient { rpc_url };
        let neutron_key = NeutronKey::new_wasm_account_mapping(
            b"balances",
            &read_pion_1_default_account_address(),
            &read_pion_1_vault_contract_address(),
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

    #[tokio::test]
    pub async fn test_get_neutron_pion_vault_shares_merkle_proof() {
        let rpc_url = read_rpc_url();
        let prover = NeutronMerkleRpcClient { rpc_url };
        let neutron_key =
            NeutronKey::new_wasm_stored_value("shares", &read_pion_1_vault_contract_address());
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
