use alloy::rpc::types::EIP1186AccountProofResponse;
use common::merkle::types::MerkleProver;
use ethereum::merkle_lib::types::{EthereumProof, MerkleProverEvm};
use neutron::merkle_lib::types::{MerkleProverNeutron, NeutronKey, NeutronProof};
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct CoprocessorConfig {
    // the neutron keys we are interested in for our valence program
    pub neutron_keys: Vec<NeutronKey>,
    // the ethereum keys we are interested in for our valence program
    // a tuple of (Key, Address)
    pub ethereum_keys: Vec<(String, String)>,
    pub neutron_rpc: String,
    pub ethereum_rpc: String,
}

#[derive(Serialize, Deserialize)]
pub struct Coprocessor {
    pub config: CoprocessorConfig,
}

impl Coprocessor {
    pub async fn get_ethereum_proofs(
        &self,
        height: u64,
        block_state_root: &[u8],
    ) -> Vec<(EthereumProof, EthereumProof)> {
        // pair of account proof and storage proof for that account
        let mut eth_proofs: Vec<(EthereumProof, EthereumProof)> = vec![];
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

    pub async fn get_neutron_proofs(&self, height: u64) -> Vec<NeutronProof> {
        // neutron proof with combined account & storage proof
        let mut neutron_proofs: Vec<NeutronProof> = vec![];
        let merkle_prover = MerkleProverNeutron {
            rpc_url: self.config.neutron_rpc.clone(),
        };
        for key in &self.config.neutron_keys {
            let proof = merkle_prover
                .get_merkle_proof_from_rpc(&key.serialize(), "", height)
                .await;
            let neutron_proof: NeutronProof = serde_json::from_slice(&proof).unwrap();
            neutron_proofs.push(neutron_proof);
        }
        neutron_proofs
    }
}

#[cfg(test)]
mod test {
    use super::{Coprocessor, CoprocessorConfig};
    use alloy::hex;
    use common::merkle::types::MerkleVerifiable;
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
        for proof in ethereum_proofs {
            proof.0.verify(&state_root);
            // must equal the storage hash of the account
            proof.1.verify(&proof.1.root);
        }
        for proof in neutron_proofs {
            proof.verify(&base64::decode(read_test_vector_merkle_root()).unwrap());
        }
    }
}

/*
ETH_RPC="https://mainnet.infura.io/v3/"
INFURA="8a860169610f4d6085a9914fcced4499"
TEST_VECTOR_DENOM_NEUTRON="untrn"
TEST_VECTOR_HEIGHT_NEUTRON="2"
TEST_VECTOR_MERKLE_ROOT_NEUTRON="4+uLhY4xjREfoNoGuASRvudkbCTMorsu19oBV3l2kLM="
NEUTRON_RPC="http://localhost:26657"
*/
