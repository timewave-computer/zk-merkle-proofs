#[cfg(feature = "no-sp1")]
#[cfg(test)]
mod tests {
    use crate::{
        keys::NeutronKey,
        merkle_lib::{
            tests::defaults::{
                read_pion_1_default_account_address, read_pion_1_vault_contract_address,
                read_rpc_url, read_test_vector_height,
            },
            types::MerkleProverNeutron,
        },
    };
    use common::merkle::types::MerkleProver;

    #[tokio::test]
    pub async fn test_get_neutron_pion_vault_contract_balance_merkle_proof() {
        let rpc_url = read_rpc_url();
        let prover = MerkleProverNeutron { rpc_url };
        let neutron_key = NeutronKey::new_wasm_account_mapping(
            b"balances",
            &read_pion_1_default_account_address(),
            &read_pion_1_vault_contract_address(),
        );
        let _proofs = prover
            .get_merkle_proof_from_rpc(&neutron_key.serialize(), "", read_test_vector_height())
            .await;
    }

    #[tokio::test]
    pub async fn test_get_neutron_pion_vault_shares_merkle_proof() {
        let rpc_url = read_rpc_url();
        let prover = MerkleProverNeutron { rpc_url };
        let neutron_key =
            NeutronKey::new_wasm_stored_value("shares", &read_pion_1_vault_contract_address());
        let _proofs = prover
            .get_merkle_proof_from_rpc(&neutron_key.serialize(), "", read_test_vector_height())
            .await;
    }
}
