#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::merkle_lib::tests::defaults::read_sepolia_url;
    use crate::merkle_lib::types::MerkleProverEvm;
    use crate::merkle_lib::{
        keccak::digest_keccak,
        tests::defaults::{
            read_ethereum_vault_contract_address, read_sepolia_default_account_address,
            read_sepolia_height,
        },
    };
    use alloy::{
        consensus::Account,
        hex::FromHex,
        providers::{Provider, ProviderBuilder},
    };
    use alloy_primitives::U256;
    use alloy_sol_types::SolValue;
    use common::merkle::types::MerkleVerifiable;
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
        let merkle_prover = MerkleProverEvm {
            rpc_url: read_sepolia_url().to_string(),
        };
        let (mut account_proof, mut eth_proof) = merkle_prover
            .get_account_and_storage_proof(
                &alloy::hex::encode(&keccak_key),
                &read_ethereum_vault_contract_address(),
                sepolia_height,
            )
            .await;
        assert_eq!(eth_proof.height, sepolia_height);
        assert_eq!(account_proof.height, sepolia_height);
        let block = provider
            .get_block_by_number(alloy::eips::BlockNumberOrTag::Number(sepolia_height))
            .await
            .expect("Failed to get Block!")
            .expect("Block not found!");
        // todo: remove the need for this
        account_proof.hash_key();
        let proof_output = account_proof.verify(&block.header.state_root.to_vec(), 0);
        let account: Account = alloy_rlp::decode_exact(&proof_output.value).unwrap();
        let leaf = eth_proof.proof.last().unwrap().to_owned();
        // todo: remove the need for this
        eth_proof.hash_key();
        eth_proof.verify(account.storage_root.as_slice(), 0);
        // verify the stored value matches the expected value
        let leaf_decoded: Vec<alloy_primitives::Bytes> = alloy_rlp::decode_exact(&leaf).unwrap();
        let value_encoded = leaf_decoded.get(1).unwrap();
        assert_eq!(value_encoded.to_vec(), eth_proof.value);
    }

    #[tokio::test]
    async fn test_vault_contract_shares_on_sepolia() {
        let sepolia_height = read_sepolia_height().await;
        use alloy::hex;
        let storage_slot_key =
            hex::decode("0x0000000000000000000000000000000000000000000000000000000000000001")
                .unwrap();

        let provider = ProviderBuilder::new().on_http(Url::from_str(&read_sepolia_url()).unwrap());
        let merkle_prover = MerkleProverEvm {
            rpc_url: read_sepolia_url().to_string(),
        };
        let (mut account_proof, mut eth_proof) = merkle_prover
            .get_account_and_storage_proof(
                &alloy::hex::encode(&storage_slot_key),
                &read_ethereum_vault_contract_address(),
                sepolia_height,
            )
            .await;
        assert_eq!(eth_proof.height, sepolia_height);
        assert_eq!(account_proof.height, sepolia_height);
        let block = provider
            .get_block_by_number(alloy::eips::BlockNumberOrTag::Number(sepolia_height))
            .await
            .expect("Failed to get Block!")
            .expect("Block not found!");
        // todo: remove the need for this
        account_proof.hash_key();
        let proof_output = account_proof.verify(&block.header.state_root.to_vec(), 0);
        let account: Account = alloy_rlp::decode_exact(&proof_output.value).unwrap();
        let leaf = eth_proof.proof.last().unwrap().to_owned();
        // todo: remove the need for this
        eth_proof.hash_key();
        eth_proof.verify(account.storage_root.as_slice(), 0);
        // verify the stored value matches the expected value
        let leaf_decoded: Vec<alloy_primitives::Bytes> = alloy_rlp::decode_exact(&leaf).unwrap();
        let value_encoded = leaf_decoded.get(1).unwrap();
        assert_eq!(value_encoded.to_vec(), eth_proof.value);
    }
}
