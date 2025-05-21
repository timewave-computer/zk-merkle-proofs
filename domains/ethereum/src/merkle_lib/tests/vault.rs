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
    use crate::merkle_lib::types::{EthereumAccountProof, EthereumProofType, EthereumStorageProof};
    use crate::merkle_lib::{digest_keccak, rlp_decode_account, rlp_decode_bytes};
    use crate::timewave_rlp;
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
        combined_nodes.push(account_proof_len.to_be_bytes().to_vec());
        combined_nodes.push(storage_proof_len.to_be_bytes().to_vec());

        // Add the actual proof nodes
        combined_nodes.extend(combined_proof.account_proof.proof.clone());
        combined_nodes.extend(combined_proof.storage_proof.proof.clone());

        let mut combined_key: Vec<u8> = Vec::new();
        println!(
            "combined_proof.account_proof.address: {:?}",
            combined_proof.account_proof.address.len()
        );
        // combine the address and storage proof nodes into a single, flattened proof
        combined_key.extend(combined_proof.account_proof.address.clone());
        combined_key.extend(combined_proof.storage_proof.key);

        // Skip the length nodes when getting the actual proof nodes
        let account_proof_nodes = combined_nodes[2..2 + account_proof_len].to_vec();
        let storage_proof_nodes = combined_nodes[2 + account_proof_len..].to_vec();

        assert_eq!(account_proof_nodes, combined_proof.account_proof.proof);
        let storage_proof_key = combined_key[20..].to_vec();

        let mut account_hash_from_storage_proof = "".to_string();

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
            TrieNode::Leaf(leaf) => {
                println!("leaf: {:?}", leaf);
            }
            TrieNode::Branch(branch) => {
                // Crazy RLP magic - this could also be an Extension node, must handle accordingly
                let x: Vec<alloy_trie::nodes::RlpNode> = branch.stack.clone();
                // Create a list of 17 elements (16 children + value)
                let mut rlp_list = Vec::new();
                let mut stack_idx = 0;
                for i in 0..16 {
                    if branch.state_mask.is_bit_set(i as u8) {
                        rlp_list.push(x[stack_idx].as_slice().to_vec());
                        stack_idx += 1;
                    } else {
                        rlp_list.push(vec![0x80]); // empty string
                    }
                }
                rlp_list.push(vec![0x80]); // value is always empty string

                // RLP encode the list using helper function
                let mut out = Vec::new();
                let header = timewave_rlp::Header {
                    list: true,
                    payload_length: rlp_list.iter().map(|x| x.len()).sum::<usize>(),
                };
                header.encode(&mut out);
                for item in rlp_list {
                    out.extend_from_slice(&item);
                }
                let encoded = out;

                let hash = digest_keccak(&encoded);
                account_hash_from_storage_proof = hex::encode(hash);
            }
            TrieNode::Extension(extension) => {
                println!("extension: {:?}", extension);
            }
            _ => panic!("Account proof is not a node of any kind"),
        }

        // Assert that the storage proof is under the storage root used in the account proof
        let account_decoded = rlp_decode_account(&combined_proof.account_proof.value).unwrap();
        let account_hash = hex::encode(account_decoded.storage_root);
        assert_eq!(account_hash, account_hash_from_storage_proof);

        let leaf_node_decoded = rlp_decode_bytes(&account_proof_nodes.last().unwrap()).unwrap();
        let account_proof = EthereumAccountProof::new(
            account_proof_nodes.clone(),
            combined_proof.account_proof.address,
            leaf_node_decoded.last().unwrap().to_vec(),
        );

        let account_result = account_proof
            .verify(block.header.state_root.as_slice())
            .unwrap();

        assert!(account_result);
    }
}
