use crate::merkle_lib::helpers::convert_tm_to_ics_merkle_proof;
use common::{types::MerkleProofOutput, MerkleProver, MerkleVerifiable};
use ics23::{
    calculate_existence_root, commitment_proof::Proof, iavl_spec, tendermint_spec,
    verify_membership,
};
use serde::{Deserialize, Serialize};
use tendermint::{block::Height, merkle::proof::ProofOps};
#[cfg(feature = "web")]
use tendermint_rpc::{Client, HttpClient};
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NeutronKey {
    pub prefix: String,
    pub key: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NeutronProof {
    pub proof: ProofOps,
    pub key: NeutronKey,
    pub value: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NeutronProofWithRoot {
    pub proof: NeutronProof,
    pub root: Vec<u8>,
}
impl MerkleVerifiable for NeutronProofWithRoot {
    fn verify(&self, expected_root: &[u8]) -> MerkleProofOutput {
        self.proof.verify(expected_root)
    }
}

impl MerkleVerifiable for NeutronProof {
    fn verify(&self, expected_root: &[u8]) -> MerkleProofOutput {
        let proof_decoded = convert_tm_to_ics_merkle_proof(&self.proof);
        let inner_proof = proof_decoded.first().unwrap();
        let Some(Proof::Exist(existence_proof)) = &inner_proof.proof else {
            panic!("Wrong proof type!");
        };
        let inner_root =
            calculate_existence_root::<ics23::HostFunctionsManager>(&existence_proof).unwrap();
        let is_valid = verify_membership::<ics23::HostFunctionsManager>(
            &inner_proof,
            &iavl_spec(),
            &inner_root,
            &hex::decode(&self.key.key).unwrap(),
            &self.value,
        );
        assert!(is_valid);
        let outer_proof = proof_decoded.last().unwrap();
        let is_valid = verify_membership::<ics23::HostFunctionsManager>(
            &outer_proof,
            &tendermint_spec(),
            &expected_root.to_vec(),
            &self.key.prefix.as_bytes(),
            &inner_root,
        );
        assert!(is_valid);
        MerkleProofOutput {
            root: expected_root.to_vec(),
            key: serde_json::to_vec(&self.key).unwrap(),
            value: self.value.clone(),
            domain: common::Domain::NEUTRON,
        }
    }
}
// we might want to rename this IF this can be generalized to something like "cosmos" or "ics23-common"
pub struct NeutronProver {
    pub rpc_url: String,
}

#[cfg(feature = "web")]
impl MerkleProver for NeutronProver {
    // chunk[0] = prefix string, chunk[1] = hex encoded key
    #[allow(unused)]
    async fn get_storage_proof(&self, keys: Vec<&str>, address: &str, height: u64) -> Vec<u8> {
        let client = HttpClient::new(self.rpc_url.as_str()).unwrap();

        assert_eq!(keys.len(), 2);
        let prefix = keys.first().unwrap();
        let key = keys.last().unwrap();
        let response: tendermint_rpc::endpoint::abci_query::AbciQuery = client
            .abci_query(
                // "store/bank/key", "store/wasm/key", ...
                Some(format!("{}{}{}", "store/", prefix.to_string(), "/key")),
                hex::decode(key).unwrap(),
                Some(Height::from(height as u32)),
                true, // Include proof
            )
            .await
            .unwrap();
        let proof = response.proof.unwrap();
        serde_json::to_vec(&NeutronProof {
            proof: proof.clone(),
            key: NeutronKey {
                prefix: prefix.to_string(),
                key: key.to_string(),
            },
            value: response.value,
        })
        .unwrap()
    }
}
