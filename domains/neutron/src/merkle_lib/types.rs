use crate::{keys::NeutronKey, merkle_lib::helpers::convert_tm_to_ics_merkle_proof};
use common::merkle::types::MerkleVerifiable;
use ics23::{
    calculate_existence_root, commitment_proof::Proof, iavl_spec, tendermint_spec,
    verify_membership,
};
use serde::{Deserialize, Serialize};
use tendermint::merkle::proof::ProofOps;
use anyhow::Result;
/// Represents a Merkle proof for state on the Neutron blockchain.
///
/// This type combines the proof data from Tendermint with the key and value
/// being proven, allowing for verification of state existence.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NeutronMerkleProof {
    /// The Tendermint proof operations
    pub proof: ProofOps,
    /// The key being proven
    pub key: NeutronKey,
    /// The value being proven
    pub value: Vec<u8>,
}

/// A wrapper type that combines a Merkle proof with its expected root hash.
///
/// This type is used as input for verification operations, providing both
/// the proof and the expected root hash that the proof should verify against.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NeutronMerkleProofWithRoot {
    /// The Merkle proof to verify
    pub proof: NeutronMerkleProof,
    /// The expected root hash
    pub root: Vec<u8>,
}

impl MerkleVerifiable for NeutronMerkleProofWithRoot {
    fn verify(&self, expected_root: &[u8]) -> Result<bool> {
        self.proof.verify(expected_root)
    }
}

impl MerkleVerifiable for NeutronMerkleProof {
    fn verify(&self, expected_root: &[u8]) -> Result<bool> {
        let proof_decoded = convert_tm_to_ics_merkle_proof(&self.proof);
        let inner_proof = proof_decoded.first().unwrap();
        let Some(Proof::Exist(existence_proof)) = &inner_proof.proof else {
            panic!("Wrong proof type!");
        };
        let inner_root =
            calculate_existence_root::<ics23::HostFunctionsManager>(existence_proof).unwrap();
        let is_valid = verify_membership::<ics23::HostFunctionsManager>(
            inner_proof,
            &iavl_spec(),
            &inner_root,
            &hex::decode(&self.key.key).unwrap(),
            &self.value,
        );
        assert!(is_valid);
        let outer_proof = proof_decoded.last().unwrap();
        let is_valid = verify_membership::<ics23::HostFunctionsManager>(
            outer_proof,
            &tendermint_spec(),
            &expected_root.to_vec(),
            self.key.prefix.as_bytes(),
            &inner_root,
        );
        Ok(is_valid)
    }
}

#[test]
fn test_neutron_key_serialization() {
    let key = NeutronKey {
        // max supported key length is 999, which is unrealistic for neutron.
        prefix: "some_long_key_to_rule_out_issues".to_string(),
        prefix_len: "some_long_key_to_rule_out_issues".to_string().len(),
        key: "0x000".to_string(),
    };
    let key_serialized = key.serialize();
    let key_deserialized = NeutronKey::deserialize(&key_serialized);
    assert_eq!(key_deserialized, key);
}
