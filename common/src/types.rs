use serde::{Deserialize, Serialize};

use crate::Domain;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MerkleProofOutput {
    // commit to the encoded roots
    pub root: Vec<u8>,
    // the keys that were queried
    pub key: Vec<u8>,
    // commit to the encoded values
    pub value: Vec<u8>,
    // the domain
    pub domain: Domain,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MerkleProofOutputs {
    pub outputs: Vec<MerkleProofOutput>,
}
