use common::merkle::types::MerkleRpcClient;
use tendermint::block::Height;
use tendermint_rpc::{Client, HttpClient};

use crate::{keys::NeutronKey, merkle_lib::types::NeutronMerkleProof};

/// A prover implementation for retrieving Merkle proofs from a Neutron RPC endpoint.
///
/// This type provides functionality to interact with a Neutron node's RPC interface
/// to retrieve Merkle proofs for specific state queries.
pub struct NeutronMerkleRpcClient {
    /// The URL of the Neutron RPC endpoint
    pub rpc_url: String,
}

impl MerkleRpcClient for NeutronMerkleRpcClient {
    #[allow(unused)]
    async fn get_proof(&self, key: &str, address: &str, height: u64) -> Vec<u8> {
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
        assert!(!response.value.is_empty());
        serde_json::to_vec(&NeutronMerkleProof {
            proof: proof.clone(),
            key: neutron_key,
            value: response.value,
        })
        .unwrap()
    }
}
