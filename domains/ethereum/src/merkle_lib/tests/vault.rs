#[cfg(test)]
mod tests {
    use crate::merkle_lib::tests::defaults::read_sepolia_url;
    use crate::merkle_lib::types::decode_rlp_bytes;
    use crate::merkle_lib::{
        keccak::digest_keccak,
        tests::defaults::{
            read_ethereum_vault_contract_address, read_sepolia_default_account_address,
            read_sepolia_height,
        },
    };
    use crate::rpc::EvmMerkleRpcClient;
    use alloy::hex;
    use alloy::{
        hex::FromHex,
        providers::{Provider, ProviderBuilder},
    };
    use alloy_primitives::U256;
    use alloy_sol_types::SolValue;
    use common::merkle::types::MerkleVerifiable;
    use std::str::FromStr;
    use url::Url;

    #[tokio::test]
    async fn test_vault_contract_balance_on_sepolia() {
        let sepolia_height = read_sepolia_height().await;
        let address =
            alloy_primitives::Address::from_hex(read_sepolia_default_account_address()).unwrap();
        let slot: U256 = alloy_primitives::U256::from(0);
        let encoded_key = (address, slot).abi_encode();
        let keccak_key = digest_keccak(&encoded_key).to_vec();
        let provider = ProviderBuilder::new().on_http(Url::from_str(&read_sepolia_url()).unwrap());
        let merkle_prover = EvmMerkleRpcClient {
            rpc_url: read_sepolia_url().to_string(),
        };
        let (account_proof, storage_proof) = merkle_prover
            .get_account_and_storage_proof(
                &alloy::hex::encode(&keccak_key),
                &read_ethereum_vault_contract_address(),
                sepolia_height,
            )
            .await;
        let block = provider
            .get_block_by_number(alloy::eips::BlockNumberOrTag::Number(sepolia_height))
            .await
            .expect("Failed to get Block!")
            .expect("Block not found!");
        assert!(account_proof.verify(block.header.state_root.as_slice()));
        let account_decoded = decode_rlp_bytes(&account_proof.value);
        assert!(storage_proof.verify(account_decoded.get(2).unwrap()));
    }

    #[tokio::test]
    async fn test_vault_contract_shares_on_sepolia() {
        let sepolia_height = read_sepolia_height().await;
        let storage_slot_key =
            hex::decode("0x0000000000000000000000000000000000000000000000000000000000000001")
                .unwrap();

        let provider = ProviderBuilder::new().on_http(Url::from_str(&read_sepolia_url()).unwrap());
        let merkle_prover = EvmMerkleRpcClient {
            rpc_url: read_sepolia_url().to_string(),
        };
        let (account_proof, storage_proof) = merkle_prover
            .get_account_and_storage_proof(
                &alloy::hex::encode(&storage_slot_key),
                &read_ethereum_vault_contract_address(),
                sepolia_height,
            )
            .await;
        let block = provider
            .get_block_by_number(alloy::eips::BlockNumberOrTag::Number(sepolia_height))
            .await
            .expect("Failed to get Block!")
            .expect("Block not found!");
        assert!(account_proof.verify(block.header.state_root.as_slice()));
        let account_decoded = decode_rlp_bytes(&account_proof.value);
        assert!(storage_proof.verify(account_decoded.get(2).unwrap()));
    }

    #[tokio::test]
    async fn test_account_and_storage_proof_from_rpc() {
        let sepolia_height = read_sepolia_height().await;
        let storage_slot_key =
            hex::decode("0x0000000000000000000000000000000000000000000000000000000000000001")
                .unwrap();
        let provider = ProviderBuilder::new().on_http(Url::from_str(&read_sepolia_url()).unwrap());
        let prover = EvmMerkleRpcClient {
            rpc_url: read_sepolia_url().to_string(),
        };
        let block = provider
            .get_block_by_number(alloy::eips::BlockNumberOrTag::Number(sepolia_height))
            .await
            .expect("Failed to get Block!")
            .expect("Block not found!");
        let account_proof = prover
            .get_account_proof(
                &alloy::hex::encode(&storage_slot_key),
                &read_ethereum_vault_contract_address(),
                sepolia_height,
            )
            .await;
        assert!(account_proof.verify(block.header.state_root.as_slice()));
        let storage_proof = prover
            .get_storage_proof(
                &alloy::hex::encode(&storage_slot_key),
                &read_ethereum_vault_contract_address(),
                sepolia_height,
            )
            .await;

        let account_decoded = decode_rlp_bytes(&account_proof.value);
        assert!(storage_proof.verify(account_decoded.get(2).unwrap()));
    }
}
