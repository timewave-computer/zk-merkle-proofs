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
