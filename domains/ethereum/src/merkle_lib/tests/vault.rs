#[cfg(test)]
mod tests {
    extern crate alloc;
    use core::str::FromStr;

    use crate::ethereum_rpc::rpc::EvmMerkleRpcClient;
    use crate::merkle_lib::tests::defaults::constants::{
        read_ethereum_vault_balances_storage_key, read_sepolia_url,
    };
    use crate::merkle_lib::tests::defaults::constants::{
        read_ethereum_vault_contract_address, read_sepolia_default_account_address,
        read_sepolia_height,
    };
    use crate::merkle_lib::types::{EthereumAccount, EthereumSimpleProof};
    use crate::merkle_lib::{digest_keccak, rlp_decode_bytes, RlpDecodable};
    use alloy::hex;
    use alloy::{
        hex::FromHex,
        providers::{Provider, ProviderBuilder},
    };
    use alloy_primitives::U256;
    use alloy_sol_types::SolValue;
    use common::merkle::types::MerkleVerifiable;
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

        let account_decoded =
            EthereumAccount::rlp_decode(&combined_proof.account_proof.value).unwrap();
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

    // this test needs to be updated manually with a recent root and height
    // because of this it's commented out and should only be used when needed
    // this test needs to be updated manually with a recent root and height
    // because of this it's commented out and should only be used when needed
    #[tokio::test]
    async fn test_decode_withdraw_mainnet() {
        let string_slot_hex = "92e85d02570a8092d09a6e3a57665bc3815a2699a4074001bf1ccabf660f5a39";
        let string_slot_key = hex::decode(string_slot_hex).unwrap();

        let hashed_slot = digest_keccak(&string_slot_key);
        let current_slot = U256::from_be_slice(&hashed_slot);
        let merkle_prover = EvmMerkleRpcClient {
            rpc_url: "https://eth-mainnet.g.alchemy.com/v2/D1CbidVntzlEbD4x7iyHnZZaPWzvDe9I"
                .to_string(),
        };
        let contract_address = "0xf2B85C389A771035a9Bd147D4BF87987A7F9cf98".to_string();
        let block_number = 22632564 - (32 * 10);

        let mut full_string = Vec::new();
        let mut i = 0;
        loop {
            let chunk_slot = current_slot + U256::from(i);
            let chunk_slot_hex = format!("{:064x}", chunk_slot);
            println!("Chunk slot hex: {:?}", chunk_slot_hex);
            let chunk_proof = match merkle_prover
                .get_account_and_storage_proof(&chunk_slot_hex, &contract_address, block_number)
                .await
            {
                Ok(proof) => proof,
                Err(e) => {
                    println!("Error: {:?}", e);
                    break;
                }
            };

            let simple_proof: EthereumSimpleProof =
                EthereumSimpleProof::from_combined_proof(chunk_proof.clone());

            let address = simple_proof.get_address();
            let address_hex = hex::encode(address);
            println!("Address: {:?}", address_hex);
            /*assert!(simple_proof
                .verify(
                    hex::decode("915b13c8d2fa07ab14cac8272ca3a02a2ea4e97b9d06d30ac7c1d80824e8f0b7")
                        .unwrap()
                        .as_slice(),
                )
                .unwrap());

            assert!(chunk_proof
                .verify(
                    &hex::decode(
                        "915b13c8d2fa07ab14cac8272ca3a02a2ea4e97b9d06d30ac7c1d80824e8f0b7"
                    )
                    .unwrap()
                )
                .unwrap());*/
            if chunk_proof.storage_proof.value.len() > 0 {
                full_string.extend_from_slice(&chunk_proof.storage_proof.value[1..]);
            } else {
                break;
            }

            /*let account_decoded =
                EthereumAccount::rlp_decode(&chunk_proof.account_proof.value).unwrap();
            assert!(chunk_proof
                .storage_proof
                .verify(&account_decoded.storage_root)
                .unwrap());*/
            i += 1;
        }

        let decoded_string = String::from_utf8_lossy(&full_string).to_string();
        println!("Decoded receiver string: {:?}", decoded_string);
    }
}
