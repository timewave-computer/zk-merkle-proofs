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
    use crate::merkle_lib::types::{EthereumAccountProof, EthereumStorageProof};
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
    async fn test_fancy_new_stuff() {
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

        let mut combined_nodes: Vec<Vec<u8>> = Vec::new();
        let account_proof_len = combined_proof.account_proof.proof.len();
        let storage_proof_len = combined_proof.storage_proof.proof.len();

        // Add length information as the first two nodes
        // We assume proof lengths will never exceed 65,535 bytes (u16::MAX)
        combined_nodes.push((account_proof_len as u16).to_be_bytes().to_vec());
        combined_nodes.push((storage_proof_len as u16).to_be_bytes().to_vec());

        // Add the actual proof nodes
        combined_nodes.extend(combined_proof.account_proof.proof.clone());
        combined_nodes.extend(combined_proof.storage_proof.proof.clone());

        let mut combined_key: Vec<u8> = Vec::new();
        // Declare the keys
        let account_key = combined_proof.account_proof.address.clone();
        let storage_key = combined_proof.storage_proof.key.clone();

        // Add key length information (using u16 to be consistent with node lengths)
        // We assume key lengths will never exceed 65,535 bytes (u16::MAX)
        let key_length = (account_key.len() + storage_key.len()) as u16;
        combined_key.extend(key_length.to_be_bytes().to_vec());

        // combine the address and storage proof nodes into a single, flattened proof
        combined_key.extend(account_key);
        combined_key.extend(storage_key);

        // Declare the values
        let account_value = combined_proof.account_proof.value.clone();
        let storage_value = combined_proof.storage_proof.value.clone();

        // Create combined values with length information
        let mut combined_values: Vec<u8> = Vec::new();
        // We assume value lengths will never exceed 65,535 bytes (u16::MAX)
        let value_length = (account_value.len() + storage_value.len()) as u16;
        combined_values.extend(value_length.to_be_bytes().to_vec());
        combined_values.extend(account_value);
        combined_values.extend(storage_value);

        // Skip the length nodes when getting the actual proof nodes
        let account_proof_nodes = combined_nodes[2..2 + account_proof_len].to_vec();
        let storage_proof_nodes = combined_nodes[2 + account_proof_len..].to_vec();

        // Skip the length bytes when getting the actual key parts (now using 2 bytes for length)
        let account_key_part =
            combined_key[2..2 + combined_proof.account_proof.address.len()].to_vec();
        let storage_key_part =
            combined_key[2 + combined_proof.account_proof.address.len()..].to_vec();

        // Skip the length bytes when getting the actual value parts (now using 2 bytes for length)
        let account_value_part =
            combined_values[2..2 + combined_proof.account_proof.value.len()].to_vec();
        let storage_value_part =
            combined_values[2 + combined_proof.account_proof.value.len()..].to_vec();

        let mut account_hash_from_storage_proof = "".to_string();

        let mut is_extension = false;

        match TrieNode::decode(&mut &account_proof_nodes.last().unwrap()[..]).unwrap() {
            TrieNode::Leaf(leaf) => {
                println!("leaf: {:?}", leaf);
            }
            TrieNode::Branch(branch) => {
                println!("branch: {:?}", branch);
            }
            TrieNode::Extension(extension) => {
                println!("extension: {:?}", extension);
            }
            _ => panic!("Account proof is not a node of any kind"),
        }

        match TrieNode::decode(&mut &storage_proof_nodes.first().unwrap()[..]).unwrap() {
            TrieNode::Branch(branch) => {
                // Create a list of 17 elements (16 children + value)
                let mut rlp_list: Vec<Vec<u8>> = Vec::new();
                let mut stack_idx = 0;

                // Process each child index in order
                for i in CHILD_INDEX_RANGE {
                    if branch.state_mask.is_bit_set(i as u8) {
                        rlp_list.push(branch.stack[stack_idx].as_slice().to_vec());
                        stack_idx += 1;
                    } else {
                        rlp_list.push(vec![0x80]); // Empty string for unset children
                    }
                }
                rlp_list.push(vec![0x80]); // Empty value

                // Encode the list
                let mut encoded = Vec::new();
                let header = timewave_rlp::Header {
                    list: true,
                    payload_length: rlp_list.iter().map(|x| x.len()).sum::<usize>(),
                };
                header.encode(&mut encoded);
                for item in rlp_list {
                    encoded.extend_from_slice(&item);
                }
                let hash = digest_keccak(&encoded);
                account_hash_from_storage_proof = hex::encode(hash);
            }
            TrieNode::Extension(extension) => {
                // we need to handle extension roots
            }
            TrieNode::Leaf(leaf) => {
                // we need to handle leaf nodes
            }
            _ => panic!("Account proof is not a node of any kind"),
        }

        // Assert that the storage proof is under the storage root used in the account proof
        let account_decoded = rlp_decode_account(&combined_proof.account_proof.value).unwrap();
        let account_hash = hex::encode(account_decoded.storage_root.clone());
        assert_eq!(account_hash, account_hash_from_storage_proof);

        let account_proof = EthereumAccountProof::new(
            account_proof_nodes.clone(),
            account_key_part,
            account_value_part,
        );

        let account_result = account_proof
            .verify(block.header.state_root.as_slice())
            .unwrap();
        assert!(account_result);

        let storage_proof = EthereumStorageProof::new(
            storage_proof_nodes.clone(),
            storage_key_part,
            storage_value_part,
        );

        let account_decoded = rlp_decode_bytes(&combined_proof.account_proof.value)
            .unwrap()
            .to_vec();

        let storage_result = storage_proof
            .verify(&account_decoded.get(2).unwrap())
            .unwrap();

        assert!(storage_result);
    }
}
