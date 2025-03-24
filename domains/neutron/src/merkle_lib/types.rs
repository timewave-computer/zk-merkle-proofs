use crate::{keys::NeutronKey, merkle_lib::helpers::convert_tm_to_ics_merkle_proof};
use common::merkle::types::{MerkleProofOutput, MerkleVerifiable};
use ics23::{
    calculate_existence_root, commitment_proof::Proof, iavl_spec, tendermint_spec,
    verify_membership,
};
use serde::{Deserialize, Serialize};
use tendermint::merkle::proof::ProofOps;

#[cfg(feature = "no-zkvm")]
use {
    common::merkle::types::MerkleProver,
    tendermint::block::Height,
    tendermint_rpc::{Client, HttpClient},
};

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
    fn verify(&self, expected_root: &[u8]) -> MerkleProofOutput {
        self.proof.verify(expected_root)
    }
}

impl MerkleVerifiable for NeutronMerkleProof {
    fn verify(&self, expected_root: &[u8]) -> MerkleProofOutput {
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
        assert!(is_valid);
        MerkleProofOutput {
            root: expected_root.to_vec(),
            key: serde_json::to_vec(&self.key).unwrap(),
            value: self.value.clone(),
            domain: common::merkle::types::Domain::NEUTRON,
        }
    }
}

/// A prover implementation for retrieving Merkle proofs from a Neutron RPC endpoint.
///
/// This type provides functionality to interact with a Neutron node's RPC interface
/// to retrieve Merkle proofs for specific state queries.
pub struct MerkleProverNeutron {
    /// The URL of the Neutron RPC endpoint
    pub rpc_url: String,
}

#[cfg(feature = "no-zkvm")]
impl MerkleProver for MerkleProverNeutron {
    #[allow(unused)]
    async fn get_merkle_proof_from_rpc(&self, key: &str, address: &str, height: u64) -> Vec<u8> {
        let client = HttpClient::new(self.rpc_url.as_str()).unwrap();
        let neutron_key: NeutronKey = NeutronKey::deserialize(key);
        let response: tendermint_rpc::endpoint::abci_query::AbciQuery = client
            .abci_query(
                // "store/bank/key", "store/wasm/key", ...
                Some(format!("{}{}{}", "store/", neutron_key.prefix, "/key")),
                hex::decode(neutron_key.key.clone()).unwrap(),
                Some(Height::from(height as u32)),
                true, // Include proof
            )
            .await
            .unwrap();
        let proof = response.proof.unwrap();
        assert!(response.value.len() > 0);
        serde_json::to_vec(&NeutronMerkleProof {
            proof: proof.clone(),
            key: neutron_key,
            value: response.value,
        })
        .unwrap()
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
