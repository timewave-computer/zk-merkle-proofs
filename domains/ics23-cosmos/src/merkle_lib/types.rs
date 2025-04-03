use crate::{keys::Ics23Key, merkle_lib::helpers::convert_tm_to_ics_merkle_proof};
use anyhow::{Context, Result};
use common::merkle::types::MerkleVerifiable;
use ics23::{
    calculate_existence_root, commitment_proof::Proof, iavl_spec, tendermint_spec,
    verify_membership,
};
use serde::{Deserialize, Serialize};
use tendermint::merkle::proof::ProofOps;
/// Represents a Merkle proof for state on the Neutron blockchain.
///
/// This type combines the proof data from Tendermint with the key and value
/// being proven, allowing for verification of state existence.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Ics23MerkleProof {
    /// The Tendermint proof operations
    pub proof: ProofOps,
    /// The key being proven
    pub key: Ics23Key,
    /// The value being proven
    pub value: Vec<u8>,
}

/// A wrapper type that combines a Merkle proof with its expected root hash.
///
/// This type is used as input for verification operations, providing both
/// the proof and the expected root hash that the proof should verify against.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Ics23MerkleProofWithRoot {
    /// The Merkle proof to verify
    pub proof: Ics23MerkleProof,
    /// The expected root hash
    pub root: Vec<u8>,
}

impl MerkleVerifiable for Ics23MerkleProofWithRoot {
    fn verify(&self, expected_root: &[u8]) -> Result<bool> {
        self.proof.verify(expected_root)
    }
}

impl MerkleVerifiable for Ics23MerkleProof {
    fn verify(&self, expected_root: &[u8]) -> Result<bool> {
        let proof_decoded = convert_tm_to_ics_merkle_proof(&self.proof)?;
        let inner_proof = proof_decoded.first().context("Failed to decode proof")?;
        let Some(Proof::Exist(existence_proof)) = &inner_proof.proof else {
            panic!("Wrong proof type!");
        };
        let inner_root = calculate_existence_root::<ics23::HostFunctionsManager>(existence_proof)?;
        let is_valid = verify_membership::<ics23::HostFunctionsManager>(
            inner_proof,
            &iavl_spec(),
            &inner_root,
            &hex::decode(&self.key.key)?,
            &self.value,
        );
        assert!(is_valid);
        let outer_proof = proof_decoded.last().context("Failed to decode proof")?;
        let is_valid = verify_membership::<ics23::HostFunctionsManager>(
            outer_proof,
            &tendermint_spec(),
            &expected_root.to_vec(),
            self.key.prefix.as_bytes(),
            &inner_root,
        );
        match is_valid {
            true => Ok(true),
            false => anyhow::bail!("Invalid proof"),
        }
    }
}

#[test]
fn test_neutron_key_serialization() {
    let key = Ics23Key {
        // max supported key length is 999, which is unrealistic for neutron.
        prefix: "some_long_key_to_rule_out_issues".to_string(),
        prefix_len: "some_long_key_to_rule_out_issues".to_string().len(),
        key: "0x000".to_string(),
    };
    let key_serialized = key.to_string();
    let key_deserialized = Ics23Key::from_string(&key_serialized);
    assert_eq!(key_deserialized, key);
}
