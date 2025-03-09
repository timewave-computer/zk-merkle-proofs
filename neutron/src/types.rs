use common::MerkleProver;
use serde::{Deserialize, Serialize};
use tendermint::{block::Height, merkle::proof::ProofOps};
use tendermint_rpc::{Client, HttpClient};

#[derive(Serialize, Deserialize)]
pub struct NeutronProof {
    pub proofs: ProofOps,
    pub key: (String, String),
    pub value: Vec<u8>,
}
#[derive(Serialize, Deserialize)]
pub struct NeutronProofBatch {
    pub batch: Vec<NeutronProof>,
}
// we might want to rename this IF this can be generalized to something like "cosmos" or "ics23-common"
pub struct NeutronProver {
    pub rpc_url: String,
}

impl MerkleProver for NeutronProver {
    // chunk[0] = prefix string, chunk[1] = hex encoded key
    #[allow(unused)]
    async fn get_storage_proof(&self, keys: Vec<&str>, address: &str, height: u64) -> Vec<u8> {
        let mut batch: NeutronProofBatch = NeutronProofBatch { batch: vec![] };
        let client = HttpClient::new(self.rpc_url.as_str()).unwrap();
        for encoded_key in keys.chunks(2) {
            if let [prefix, key] = encoded_key {
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
                batch.batch.push(NeutronProof {
                    proofs: response.proof.unwrap(),
                    key: (prefix.to_string(), key.to_string()),
                    value: response.value,
                });
            }
        }
        serde_json::to_vec(&batch).unwrap()
    }
}
