use crate::merkle_lib::helpers::convert_tm_to_ics_merkle_proof;
use common::{
    merkle::types::MerkleProofOutput, merkle::types::MerkleProver, merkle::types::MerkleVerifiable,
};
use ics23::{
    calculate_existence_root, commitment_proof::Proof, iavl_spec, tendermint_spec,
    verify_membership,
};
use serde::{Deserialize, Serialize};
use tendermint::{block::Height, merkle::proof::ProofOps};

#[cfg(feature = "no-sp1")]
use tendermint_rpc::{Client, HttpClient};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NeutronKey {
    pub prefix: String,
    pub prefix_len: usize,
    pub key: String,
}

impl NeutronKey {
    /// Serializes the NeutronKey by encoding prefix_len explicitly.
    pub fn serialize(&self) -> String {
        format!("{:03}{}{}", self.prefix_len, self.prefix, self.key)
    }

    /// Deserializes a string back into a NeutronKey.
    pub fn deserialize(encoded: &str) -> Self {
        // Extract the first 3 characters as the prefix length
        let prefix_len: usize = encoded[..3].parse().expect("Invalid prefix length");
        // Extract the prefix and key based on prefix_len
        let prefix = &encoded[3..(3 + prefix_len)];
        let key = &encoded[(3 + prefix_len)..];

        NeutronKey {
            prefix: prefix.to_string(),
            prefix_len,
            key: key.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NeutronMerkleProof {
    pub proof: ProofOps,
    pub key: NeutronKey,
    pub value: Vec<u8>,
}

// this struct only exists as an input
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NeutronMerkleProofWithRoot {
    pub proof: NeutronMerkleProof,
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
// we might want to rename this IF this can be generalized to something like "cosmos" or "ics23-common"
pub struct MerkleProverNeutron {
    pub rpc_url: String,
}

#[cfg(feature = "no-sp1")]
impl MerkleProver for MerkleProverNeutron {
    // chunk[0] = prefix string, chunk[1] = hex encoded key
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
        serde_json::to_vec(&NeutronMerkleProof {
            proof: proof.clone(),
            key: neutron_key,
            value: response.value,
        })
        .unwrap()
    }
}
