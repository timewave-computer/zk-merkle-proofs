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
    ) -> EthTrie<MemoryDB> {
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
        let eth_db = Arc::new(MemoryDB::new(true));
        let mut ethereum_trie = EthTrie::new(eth_db.clone());
        for proof in ethereum_proofs {
            // insert the rlp encoded value for the given key
            ethereum_trie
                .insert(&proof.key, &proof.value)
                .expect("Failed to insert into Ethereum Trie");
        }
        let coprocessor_db = Arc::new(MemoryDB::new(true));
        let mut coprocessor_trie = EthTrie::new(coprocessor_db.clone());
        coprocessor_trie
            .insert(
                b"ethereum",
                &ethereum_trie
                    .root_hash()
                    .expect("Failed to compute ethereum trie root")
                    .to_vec(),
            )
            .expect("Failed to insert ethereum root into coprocessor trie");
        coprocessor_trie
            .insert(
                b"neutron",
                &neutron_trie
                    .root_hash()
                    .expect("Failed to compute neutron trie root")
                    .to_vec(),
            )
            .expect("Failed to insert neutron root into coprocessor trie");
        // the coprocessor trie can now be used to obtain merkle proofs for any ethereum/neutron values
        coprocessor_trie
    }
}

#[cfg(test)]
mod test {
    use super::{Coprocessor, CoprocessorConfig};
    use alloy::hex;
    use common::merkle::types::MerkleVerifiable;
    use eth_trie::Trie;
    use ethereum::merkle_lib::test_vector::{
        read_api_key, read_rpc_url as read_ethereum_rpc_url, DEFAULT_ETH_BLOCK_HEIGHT,
        DEFAULT_STORAGE_KEY_ETHEREUM, USDT_CONTRACT_ADDRESS,
    };
    use neutron::merkle_lib::{
        test_vector::{
            construct_supply_key, read_rpc_url as read_neutron_rpc_url, read_test_vector_denom,
            read_test_vector_height, read_test_vector_merkle_root,
        },
        types::NeutronKey,
    };
    #[tokio::test]
    async fn test_coprocessor() {
        let supply_key = construct_supply_key(&read_test_vector_denom(), vec![0x00]);
        let neutron_key = NeutronKey {
            prefix: "bank".to_string(),
            prefix_len: 4,
            key: hex::encode(supply_key),
        };
        let config = CoprocessorConfig {
            neutron_keys: vec![neutron_key],
            ethereum_keys: vec![(
                DEFAULT_STORAGE_KEY_ETHEREUM.to_string(),
                USDT_CONTRACT_ADDRESS.to_string(),
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
        for proof in ethereum_proofs.clone() {
            proof.0.verify(&state_root);
            // must equal the storage hash of the account
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
        let _ = coprocessor_trie.root_hash();
    }
}
