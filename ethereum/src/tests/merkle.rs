#[cfg(test)]
#[cfg(feature = "web")]
mod tests {
    use alloy::{
        hex::FromHex,
        primitives::{Address, FixedBytes},
        providers::{Provider, ProviderBuilder},
        serde::JsonStorageKey,
    };
    use common::Verifiable;
    use dotenvy::dotenv;
    use std::{env, io::Read, str::FromStr};
    use url::Url;

    use crate::{keccak::digest_keccak, EthereumProof, EvmProver};
    const USDT_CONTRACT_ADDRESS: &str = "0xdAC17F958D2ee523a2206206994597C13D831ec7";
    const DEFAULT_STORAGE_KEY_ETHEREUM: &str =
        "0x0000000000000000000000000000000000000000000000000000000000000000";
    #[tokio::test]
    // first verifies account state, then a single storage proof
    async fn test_verify_storage_proof_single() {
        let rpc_url = read_rpc_url() + &read_api_key();
        let prover = EvmProver { rpc_url };
        let provider = ProviderBuilder::new().on_http(Url::from_str(&prover.rpc_url).unwrap());
        let proof = provider
            .get_proof(
                Address::from_hex(USDT_CONTRACT_ADDRESS).unwrap(),
                vec![FixedBytes::from_hex(DEFAULT_STORAGE_KEY_ETHEREUM).unwrap()],
            )
            .await
            .expect("Failed to get proof!");
        let raw_storage_proofs: Vec<(Vec<Vec<u8>>, JsonStorageKey)> = proof
            .storage_proof
            .iter()
            .cloned()
            .map(|p| (p.proof.into_iter().map(|b| b.to_vec()).collect(), p.key))
            .collect();
        let first_proof = raw_storage_proofs.first().unwrap();
        let custom_proof = EthereumProof {
            root: proof.storage_hash.to_vec(),
            proof: first_proof.0.clone(),
            key: digest_keccak(
                &first_proof
                    .1
                    .as_b256()
                    .bytes()
                    .collect::<Result<Vec<u8>, _>>()
                    .unwrap()
                    .to_vec(),
            )
            .to_vec(),
        };
        custom_proof.verify(&proof.storage_hash.to_vec());
    }

    // todo: add a test for a transaction receipt e.g. verify (erc20) events

    fn read_api_key() -> String {
        dotenv().ok();
        env::var("INFURA").expect("Missing Infura API key!")
    }

    fn read_rpc_url() -> String {
        dotenv().ok();
        env::var("ETH_RPC").expect("Missing Ethereum RPC url!")
    }
}
