use serde::{Deserialize, Serialize};

use crate::Domain;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MerkleProof {
    // a list of serialized nodes
    // the last node should be the queried value on ethereum
    pub nodes: Vec<Vec<u8>>,
    // on neutron the value is supplied seperately
    pub value: Option<Vec<u8>>,
    // the key that we query
    pub key: Vec<u8>,
    // target domain
    pub domain: Domain,
    // serialized trie root
    pub root: Vec<u8>,
}

/// Circuit input - multiple proofs for multiple domains
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProofInput {
    pub proofs: Vec<MerkleProof>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProofOutput {
    // commit to the encoded roots
    pub root: (Domain, Vec<u8>),
    // the keys that were queried
    pub key: Vec<u8>,
    // commit to the encoded values
    pub value: Vec<u8>,
}
