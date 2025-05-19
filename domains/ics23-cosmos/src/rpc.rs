use std::str::FromStr;

use anyhow::{Context, Result};
use base64::Engine;
use common::merkle::types::MerkleClient;
use tendermint::block::Height;
use tendermint_rpc::{Client, HttpClient, Url as TendermintUrl};

use crate::{keys::Ics23Key, merkle_lib::types::Ics23MerkleProof};

/// A prover implementation for retrieving Merkle proofs from a Neutron RPC endpoint.
///
/// This type provides functionality to interact with a Neutron node's RPC interface
/// to retrieve Merkle proofs for specific state queries.
pub struct Ics23MerkleRpcClient {
    /// The URL of the Neutron RPC endpoint
    pub rpc_url: String,
}

impl MerkleClient for Ics23MerkleRpcClient {
    #[allow(unused)]
    async fn get_proof(&self, key: &str, address: &str, height: u64) -> Result<Vec<u8>> {
        let client = HttpClient::new(self.rpc_url.as_str())?;
        let neutron_key = Ics23Key::from_string(key).unwrap();
        let response: tendermint_rpc::endpoint::abci_query::AbciQuery = client
            .abci_query(
                // "store/bank/key", "store/wasm/key", ...
                Some(format!("{}{}{}", "store/", neutron_key.prefix, "/key")),
                hex::decode(neutron_key.key.clone())?,
                Some(Height::from(height as u32)),
                true, // Include proof
            )
            .await?;
        let proof = response.proof.context("Failed to get proof")?;
        assert!(!response.value.is_empty());
        Ok(serde_json::to_vec(&Ics23MerkleProof {
            proof: proof.clone(),
            key: neutron_key,
            value: response.value,
        })?)
    }
}

impl Ics23MerkleRpcClient {
    pub async fn get_latest_root_and_height(&self) -> (Vec<u8>, u64) {
        let tendermint_client =
            tendermint_rpc::HttpClient::new(TendermintUrl::from_str(&self.rpc_url).unwrap())
                .unwrap();
        let latest_block = tendermint_client.latest_block().await.unwrap();
        let height = latest_block.block.header.height.value() - 1;
        let app_hash = base64::engine::general_purpose::STANDARD
            .encode(hex::decode(latest_block.block.header.app_hash.to_string()).unwrap());
        (
            base64::engine::general_purpose::STANDARD
                .decode(app_hash)
                .unwrap(),
            height,
        )
    }
}
