#[cfg(test)]
mod tests {
    use crate::ethereum_rpc::rpc::EvmMerkleRpcClient;
    use crate::merkle_lib::tests::defaults::constants::{
        read_ethereum_vault_balances_storage_key, read_sepolia_url,
    };
    use crate::merkle_lib::tests::defaults::constants::{
        read_ethereum_vault_contract_address, read_sepolia_default_account_address,
        read_sepolia_height,
    };
    use crate::merkle_lib::types::{
        EthereumAccountProof, EthereumSimpleProof, EthereumStorageProof,
    };
    use crate::merkle_lib::{digest_keccak, rlp_decode_account, rlp_decode_bytes};
    use crate::timewave_rlp;
    use crate::timewave_trie::constants::{CHILD_INDEX_RANGE, EVEN_FLAG, ODD_FLAG};
    use alloy::hex;
    use alloy::rlp::Decodable;
    use alloy::{
        hex::FromHex,
        providers::{Provider, ProviderBuilder},
    };
    use alloy_primitives::U256;
    use alloy_sol_types::SolValue;
    use alloy_trie::nodes::TrieNode;
    use common::merkle::types::MerkleVerifiable;
    use std::str::FromStr;
    use url::Url;

    #[tokio::test]
    async fn test_vault_contract_balance_on_sepolia() {
        let sepolia_height = read_sepolia_height().await.unwrap();
        let address =
            alloy_primitives::Address::from_hex(read_sepolia_default_account_address()).unwrap();

        let slot: U256 = alloy_primitives::U256::from(0);
        let encoded_key = (address, slot).abi_encode();
        let keccak_key = digest_keccak(&encoded_key).to_vec();
        let provider = ProviderBuilder::new().on_http(Url::from_str(&read_sepolia_url()).unwrap());

        let merkle_prover = EvmMerkleRpcClient {
            rpc_url: read_sepolia_url().to_string(),
        };

        let combined_proof = merkle_prover
            .get_account_and_storage_proof(
                &alloy::hex::encode(&keccak_key),
                &read_ethereum_vault_contract_address(),
                sepolia_height,
            )
            .await
            .unwrap();

        let block = provider
            .get_block_by_number(alloy::eips::BlockNumberOrTag::Number(sepolia_height))
            .await
            .unwrap()
            .unwrap();

        assert!(combined_proof
            .account_proof
            .verify(block.header.state_root.as_slice())
            .unwrap());

        let account_decoded = rlp_decode_account(&combined_proof.account_proof.value).unwrap();
        assert!(combined_proof
            .storage_proof
            .verify(account_decoded.storage_root.as_slice())
            .unwrap());
    }

    #[tokio::test]
    async fn test_vault_contract_shares_on_sepolia() {
        let sepolia_height = read_sepolia_height().await.unwrap();
        let storage_slot_key = hex::decode(read_ethereum_vault_balances_storage_key()).unwrap();

        let provider = ProviderBuilder::new().on_http(Url::from_str(&read_sepolia_url()).unwrap());
        let merkle_prover = EvmMerkleRpcClient {
            rpc_url: read_sepolia_url().to_string(),
        };

        let combined_proof = merkle_prover
            .get_account_and_storage_proof(
                &alloy::hex::encode(&storage_slot_key),
                &read_ethereum_vault_contract_address(),
                sepolia_height,
            )
            .await
            .unwrap();

        let block = provider
            .get_block_by_number(alloy::eips::BlockNumberOrTag::Number(sepolia_height))
            .await
            .unwrap()
            .unwrap();

        assert!(combined_proof
            .account_proof
            .verify(block.header.state_root.as_slice())
            .unwrap());

        let account_decoded = rlp_decode_bytes(&combined_proof.account_proof.value).unwrap();
        assert!(combined_proof
            .storage_proof
            .verify(account_decoded.get(2).unwrap())
            .unwrap());
    }

    #[tokio::test]
    async fn test_account_and_storage_proof_from_rpc() {
        let sepolia_height = read_sepolia_height().await.unwrap();
        let storage_slot_key = hex::decode(read_ethereum_vault_balances_storage_key()).unwrap();
        let provider = ProviderBuilder::new().on_http(Url::from_str(&read_sepolia_url()).unwrap());
        let prover = EvmMerkleRpcClient {
            rpc_url: read_sepolia_url().to_string(),
        };
        let block = provider
            .get_block_by_number(alloy::eips::BlockNumberOrTag::Number(sepolia_height))
            .await
            .unwrap()
            .unwrap();
        let account_proof = prover
            .get_account_proof(&read_ethereum_vault_contract_address(), sepolia_height)
            .await
            .unwrap();
        assert!(account_proof
            .verify(block.header.state_root.as_slice())
            .unwrap());
        let storage_proof = prover
            .get_storage_proof(
                &alloy::hex::encode(&storage_slot_key),
                &read_ethereum_vault_contract_address(),
                sepolia_height,
            )
            .await
            .unwrap();

        let account_decoded = rlp_decode_bytes(&account_proof.value).unwrap();
        assert!(storage_proof
            .verify(account_decoded.get(2).unwrap())
            .unwrap());
    }

    #[tokio::test]
    async fn test_simple_state_proof() {
        // try to combine account and storage proof
        let sepolia_height = read_sepolia_height().await.unwrap();
        let storage_slot_key = hex::decode(read_ethereum_vault_balances_storage_key()).unwrap();

        let provider = ProviderBuilder::new().on_http(Url::from_str(&read_sepolia_url()).unwrap());
        let merkle_prover = EvmMerkleRpcClient {
            rpc_url: read_sepolia_url().to_string(),
        };
        let block = provider
            .get_block_by_number(alloy::eips::BlockNumberOrTag::Number(sepolia_height))
            .await
            .unwrap()
            .unwrap();

        let combined_proof = merkle_prover
            .get_account_and_storage_proof(
                &alloy::hex::encode(&storage_slot_key),
                &read_ethereum_vault_contract_address(),
                sepolia_height,
            )
            .await
            .unwrap();

        let simple_proof = EthereumSimpleProof::from_combined_proof(combined_proof);
        assert!(simple_proof
            .verify(block.header.state_root.as_slice())
            .unwrap());
    }
}
