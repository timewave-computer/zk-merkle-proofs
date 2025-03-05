use std::time::Instant;
pub const MERKLE_ELF: &[u8] = include_elf!("merkle-program");

/// entry point for the proving service
/// this function will be used to prove the merkle-program execution
/// the merkle-program will use verify_merkle_proof to verify one or more opening(s)
use common::ProofInput;
use sp1_sdk::{include_elf, ProverClient, SP1Stdin};
pub fn prove(input: ProofInput) {
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
    use std::{env, io::Read, str::FromStr};

    use alloy::{
        hex::FromHex,
        providers::{Provider, ProviderBuilder},
        serde::JsonStorageKey,
    };
    use alloy_primitives::{Address, FixedBytes};
    use common::MerkleProof;
    use dotenvy::dotenv;
    use ethereum::{keccak::digest_keccak, EvmProver};
    use url::Url;

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
            // add .block_id(block_id) in prod
            .await
            .expect("Failed to get proof!");

        let raw_storage_proofs: Vec<(Vec<Vec<u8>>, JsonStorageKey)> = proof
            .storage_proof
            .iter()
            .cloned()
            .map(|p| (p.proof.into_iter().map(|b| b.to_vec()).collect(), p.key))
            .collect();
        let nodes: Vec<Vec<u8>> = raw_storage_proofs.first().unwrap().0.clone();
        let root = proof.storage_hash.to_vec();
        prove(common::ProofInput {
            // pass a list of storage proofs to be verified in zk
            // for now we pass only one ETHEREUM merkle proof for the SUPPLY slot of the USDT contract
            proofs: vec![MerkleProof {
                nodes,
                key: digest_keccak(
                    &raw_storage_proofs
                        .first()
                        .unwrap()
                        .1
                        .as_b256()
                        .bytes()
                        .collect::<Result<Vec<u8>, _>>()
                        .unwrap(),
                )
                .to_vec(),
                domain: common::Domain::ETHEREUM,
                root: root,
            }],
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
