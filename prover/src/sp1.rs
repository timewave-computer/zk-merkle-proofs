use std::time::Instant;
pub const MERKLE_ELF: &[u8] = include_elf!("merkle-program");

/// entry point for the proving service
/// this function will be used to prove the merkle-program execution
/// the merkle-program will use verify_merkle_proof to verify one or more opening(s)
use sp1_sdk::{include_elf, ProverClient, SP1Stdin};
use verifier::MerkleProofInput;
pub fn prove(input: MerkleProofInput) {
    let start_time = Instant::now();
    let client = ProverClient::new();
    let mut stdin = SP1Stdin::new();

    // note that when verifying the merkle proof a trusted root should be used
    // instead of the root hash from input
    let proof_input = serde_json::to_vec(&input).unwrap();
    stdin.write(&proof_input);
    let (pk, vk) = client.setup(MERKLE_ELF);
    let proof = client
        .prove(&pk, &stdin)
        .run()
        .expect("Failed to generate proof!");
    client.verify(&proof, &vk).expect("Failed to verify proof!");
    let duration = start_time.elapsed();
    println!("Elapsed time: {:?}", duration);
}

#[cfg(test)]
mod tests {
    use alloy::{
        hex::FromHex,
        providers::{Provider, ProviderBuilder},
        serde::JsonStorageKey,
    };
    use alloy_primitives::{Address, FixedBytes};
    use dotenvy::dotenv;
    use ethereum::{keccak::digest_keccak, EthereumProof, EvmProver};
    use std::{env, io::Read, str::FromStr};
    use url::Url;
    use verifier::MerkleProofInput;

    use crate::sp1::prove;
    const USDT_CONTRACT_ADDRESS: &str = "0xdAC17F958D2ee523a2206206994597C13D831ec7";
    const DEFAULT_STORAGE_KEY_ETHEREUM: &str =
        "0x0000000000000000000000000000000000000000000000000000000000000000";

    #[tokio::test]
    async fn test_generate_proof() {
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
        prove(MerkleProofInput {
            // pass a list of storage proofs to be verified in zk
            // for now we pass only one ETHEREUM merkle proof for the SUPPLY slot of the USDT contract
            ethereum_proofs: vec![custom_proof],
            neutron_proofs: vec![],
        });
    }

    fn read_api_key() -> String {
        dotenv().ok();
        env::var("INFURA").expect("Missing Infura API key!")
    }

    fn read_rpc_url() -> String {
        dotenv().ok();
        env::var("ETH_RPC").expect("Missing Infura API key!")
    }
}
