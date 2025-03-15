use common::merkle::types::MerkleProver;
use ethereum::merkle_lib::types::{EthereumProof, EvmProver};
use neutron::merkle_lib::types::NeutronKey;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct CoprocessorConfig {
    // the neutron keys we are interested in for our valence program
    pub neutron_keys: Vec<NeutronKey>,
    // the ethereum keys we are interested in for our valence program
    // a tuple of (Key, Address)
    pub ethereum_keys: Vec<(Vec<String>, String)>,
    pub neutron_rpc: String,
    pub ethereum_rpc: String,
}

#[derive(Serialize, Deserialize)]
pub struct Coprocessor {
    pub config: CoprocessorConfig,
}

impl Coprocessor {
    pub async fn get_ethereum_proofs(&self, height: u64) {
        let mut proofs: Vec<EthereumProof> = vec![];
        for key in &self.config.ethereum_keys {
            let merkle_prover = EvmProver {
                rpc_url: self.config.ethereum_rpc.clone(),
            };
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
