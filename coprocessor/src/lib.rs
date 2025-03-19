use std::sync::Arc;

use alloy::rpc::types::EIP1186AccountProofResponse;
use common::merkle::types::MerkleProver;
use config::CoprocessorConfig;
use eth_trie::{EthTrie, MemoryDB, Trie};
use ethereum::merkle_lib::types::{EthereumMerkleProof, MerkleProverEvm};
use neutron::merkle_lib::types::{MerkleProverNeutron, NeutronMerkleProof};
use serde::{Deserialize, Serialize};
mod config;

#[derive(Serialize, Deserialize)]
pub struct Coprocessor {
    pub config: CoprocessorConfig,
}

#[derive(Debug)]
pub struct CoprocessorTrie {
    pub ethereum_trie: EthTrie<MemoryDB>,
    pub neutron_trie: EthTrie<MemoryDB>,
    pub coprocessor_trie: EthTrie<MemoryDB>,
}

impl Coprocessor {
    pub async fn get_ethereum_proofs(
        &self,
        height: u64,
        block_state_root: &[u8],
    ) -> Vec<(EthereumMerkleProof, EthereumMerkleProof)> {
        // pair of account proof and storage proof for that account
        let mut eth_proofs: Vec<(EthereumMerkleProof, EthereumMerkleProof)> = vec![];
        let merkle_prover = MerkleProverEvm {
            rpc_url: self.config.ethereum_rpc.clone(),
        };
        for key in &self.config.ethereum_keys {
            let raw_proof = merkle_prover
                .get_merkle_proof_from_rpc(&key.0, &key.1, height)
                .await;
            let proof_decoded: EIP1186AccountProofResponse =
                serde_json::from_slice(&raw_proof).unwrap();
            let account_storage_hash = proof_decoded.storage_hash;
            let batch = merkle_prover
                .get_account_and_storage_proof(
                    &key.0,
                    &key.1,
                    height,
                    block_state_root,
                    account_storage_hash.to_vec(),
                )
                .await;
            eth_proofs.push(batch);
        }
        eth_proofs
    }

    pub async fn get_neutron_proofs(&self, height: u64) -> Vec<NeutronMerkleProof> {
        // neutron proof with combined account & storage proof
        let mut neutron_proofs: Vec<NeutronMerkleProof> = vec![];
        let merkle_prover = MerkleProverNeutron {
            rpc_url: self.config.neutron_rpc.clone(),
        };
        for key in &self.config.neutron_keys {
            let proof = merkle_prover
                .get_merkle_proof_from_rpc(&key.serialize(), "", height)
                .await;
            let neutron_proof: NeutronMerkleProof = serde_json::from_slice(&proof).unwrap();
            assert!(neutron_proof.value.len() > 0);
            neutron_proofs.push(neutron_proof);
        }
        neutron_proofs
    }

    // this trie is meant to be built inside the circuit,
    // after the merkle proofs have been verified
    pub fn build_coprocessor_trie(
        &self,
        neutron_proofs: Vec<NeutronMerkleProof>,
        ethereum_proofs: Vec<EthereumMerkleProof>,
    ) -> CoprocessorTrie {
        let neutron_db = Arc::new(MemoryDB::new(true));
        let mut neutron_trie = EthTrie::new(neutron_db.clone());
        for proof in neutron_proofs {
            neutron_trie
                .insert(
                    &serde_json::to_vec(&proof.key.serialize()).unwrap(),
                    &proof.value,
                )
                .expect("Failed to insert into Neutron Trie");
        }
        let neutron_root = neutron_trie.root_hash().unwrap().to_vec();
        let eth_db = Arc::new(MemoryDB::new(true));
        let mut ethereum_trie = EthTrie::new(eth_db.clone());
        for proof in ethereum_proofs {
            // insert the rlp encoded value for the given key
            ethereum_trie
                .insert(&proof.key, &proof.value)
                .expect("Failed to insert into Ethereum Trie");
        }
        let eth_root = ethereum_trie.root_hash().unwrap().to_vec();
        let coprocessor_db = Arc::new(MemoryDB::new(true));
        let mut coprocessor_trie = EthTrie::new(coprocessor_db.clone());
        coprocessor_trie
            .insert(b"ethereum", &eth_root.to_vec())
            .expect("Failed to insert ethereum root into coprocessor trie");
        coprocessor_trie
            .insert(b"neutron", &neutron_root)
            .expect("Failed to insert neutron root into coprocessor trie");
        // the coprocessor trie can now be used to obtain merkle proofs for any ethereum/neutron values
        CoprocessorTrie {
            ethereum_trie,
            neutron_trie,
            coprocessor_trie,
        }
    }
}

#[cfg(test)]
#[cfg(feature = "no-sp1")]
mod test {
    use super::{Coprocessor, CoprocessorConfig};
    use crate::dangerous_call_decode_leaf_node;
    use alloy::{hex, primitives::FixedBytes};
    use common::merkle::types::MerkleVerifiable;
    use eth_trie::Trie;
    use ethereum::merkle_lib::tests::test_vector::{
        read_api_key, read_rpc_url as read_ethereum_rpc_url, DEFAULT_ETH_BLOCK_HEIGHT,
        DEFAULT_STORAGE_KEY_ETHEREUM, MAINNET_USDT_CONTRACT_ADDRESS_ETHEREUM,
    };
    use neutron::{
        keys::NeutronKey,
        merkle_lib::tests::test_vector::{
            construct_supply_key, read_rpc_url as read_neutron_rpc_url, read_test_vector_height,
            read_test_vector_merkle_root,
        },
    };
    #[tokio::test]
    async fn test_coprocessor() {
        let supply_key = construct_supply_key(&"untrn", vec![0x00]);
        let neutron_key = NeutronKey {
            prefix: "bank".to_string(),
            prefix_len: 4,
            key: hex::encode(supply_key),
        };
        let config = CoprocessorConfig {
            neutron_keys: vec![neutron_key],
            ethereum_keys: vec![(
                DEFAULT_STORAGE_KEY_ETHEREUM.to_string(),
                MAINNET_USDT_CONTRACT_ADDRESS_ETHEREUM.to_string(),
            )],
            neutron_rpc: read_neutron_rpc_url(),
            ethereum_rpc: read_ethereum_rpc_url() + &read_api_key(),
        };
        let state_root =
            hex::decode("0xf4da06dccd5bc3891b4d43b75e4a83ccea460f0bd5cde1901f368472e5ad7e4a")
                .unwrap();
        let coprocessor = Coprocessor { config };
        let ethereum_proofs = coprocessor
            .get_ethereum_proofs(DEFAULT_ETH_BLOCK_HEIGHT, &state_root)
            .await;
        let neutron_proofs = coprocessor
            .get_neutron_proofs(read_test_vector_height())
            .await;
        for mut proof in ethereum_proofs.clone() {
            proof.0.hash_key();
            proof.0.verify(&state_root);
            // must equal the storage hash of the account
            proof.1.hash_key();
            proof.1.verify(&proof.1.root);
        }
        for proof in neutron_proofs.clone() {
            #[allow(deprecated)]
            proof.verify(&base64::decode(read_test_vector_merkle_root()).unwrap());
        }
        let mut coprocessor_trie = coprocessor.build_coprocessor_trie(
            neutron_proofs,
            ethereum_proofs
                .iter()
                .map(|batch| batch.1.clone())
                .collect(),
        );
        let coprocessor_trie_root = coprocessor_trie.coprocessor_trie.root_hash().unwrap();
        let ethereum_trie_root = coprocessor_trie.ethereum_trie.root_hash().unwrap().to_vec();
        // verify a storage proof against the coprocessor trie
        let ethereum_storage_proof = coprocessor_trie
            .ethereum_trie
            .get_proof(&hex::decode(DEFAULT_STORAGE_KEY_ETHEREUM).unwrap())
            .unwrap();
        coprocessor_trie
            .ethereum_trie
            .verify_proof(
                FixedBytes::from_slice(&ethereum_trie_root),
                &hex::decode(DEFAULT_STORAGE_KEY_ETHEREUM).unwrap(),
                ethereum_storage_proof,
            )
            .expect("Value not in Eth Trie");
        let coprocessor_storage_proof = coprocessor_trie
            .coprocessor_trie
            .get_proof(b"ethereum")
            .unwrap();
        coprocessor_trie
            .coprocessor_trie
            .verify_proof(
                coprocessor_trie_root,
                b"ethereum",
                coprocessor_storage_proof.clone(),
            )
            .expect("Value not in Coprocessor Trie");
        println!("Leaf: {:?}", &coprocessor_storage_proof.last().unwrap());
        println!(
            "Raw Stored Value: {:?}",
            dangerous_call_decode_leaf_node(&coprocessor_storage_proof.last().unwrap()).1
        );

        assert!(coprocessor_storage_proof
            .last()
            .unwrap()
            .ends_with(&ethereum_trie_root));
        // todo: figure out the prefix construction / encoding
        // match the exact leaf, not just the raw suffix
        println!("Root: {:?}", &ethereum_trie_root);
        assert_eq!(ethereum_trie_root.len(), 32);
    }
}

pub fn dangerous_call_decode_leaf_node(leaf_node: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let mut index = 0;
    let list_prefix = leaf_node[index];
    index += 1;
    let list_length = if list_prefix <= 0xf7 {
        (list_prefix - 0xc0) as usize
    } else {
        let len_of_length = (list_prefix - 0xf7) as usize;
        let mut length = 0usize;
        for _ in 0..len_of_length {
            length = (length << 8) | (leaf_node[index] as usize);
            index += 1;
        }
        length
    };
    let list_end = index + list_length;
    let key_prefix = leaf_node[index];
    index += 1;
    let odd_length = (key_prefix & 0x10) != 0;
    let mut nibbles = Vec::new();

    if odd_length {
        nibbles.push(key_prefix & 0x0F);
    }
    while index < list_end && leaf_node[index] != 0xa0 {
        let byte = leaf_node[index];
        nibbles.push(byte >> 4);
        nibbles.push(byte & 0x0F);
        index += 1;
    }
    let mut key = Vec::new();
    for chunk in nibbles.chunks(2) {
        if chunk.len() == 2 {
            key.push((chunk[0] << 4) | chunk[1]);
        } else {
            key.push(chunk[0] << 4);
        }
    }
    let value_prefix = leaf_node[index];
    index += 1;
    let value_length = if value_prefix <= 0xb7 {
        (value_prefix - 0x80) as usize
    } else {
        let len_of_length = (value_prefix - 0xb7) as usize;
        let mut length = 0usize;
        for _ in 0..len_of_length {
            length = (length << 8) | (leaf_node[index] as usize);
            index += 1;
        }
        length
    };
    let value = leaf_node[index..index + value_length].to_vec();
    // note that the key here is the hex encoded nibble key
    // i think we should use the full key for now
    (key, value)
}
