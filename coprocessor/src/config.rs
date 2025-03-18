use neutron::keys::NeutronKey;
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

/*
ETH_RPC="https://mainnet.infura.io/v3/"
INFURA="8a860169610f4d6085a9914fcced4499"
TEST_VECTOR_DENOM_NEUTRON="untrn"
TEST_VECTOR_HEIGHT_NEUTRON="2"
TEST_VECTOR_MERKLE_ROOT_NEUTRON="4+uLhY4xjREfoNoGuASRvudkbCTMorsu19oBV3l2kLM="
NEUTRON_RPC="http://localhost:26657"
*/
