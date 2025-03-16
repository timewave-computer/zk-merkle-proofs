use alloy_primitives::{Address, FixedBytes};
use std::{env, io::Read, str::FromStr};

#[cfg(feature = "web")]
use {
    alloy::{
        hex::{self, FromHex, ToHex},
        providers::{Provider, ProviderBuilder},
        serde::JsonStorageKey,
    },
    dotenvy::dotenv,
    url::Url,
};

#[cfg(feature = "web")]
use crate::{merkle_lib::types::EthereumProof, merkle_lib::types::EvmProver};
pub const USDT_CONTRACT_ADDRESS: &str = "0xdAC17F958D2ee523a2206206994597C13D831ec7";
pub const DEFAULT_STORAGE_KEY_ETHEREUM: &str =
    "0x0000000000000000000000000000000000000000000000000000000000000000";
pub const DEFAULT_ETH_BLOCK_HEIGHT: u64 = 22040634;

#[cfg(feature = "web")]
pub async fn get_ethereum_test_vector_storage_proof() -> EthereumProof {
    use alloy::rpc::types::EIP1186AccountProofResponse;
    use common::merkle::types::MerkleProver;
    let rpc_url = read_rpc_url() + &read_api_key();
    let storage_key: FixedBytes<32> = FixedBytes::from_hex(DEFAULT_STORAGE_KEY_ETHEREUM).unwrap();
    let merkle_prover = EvmProver { rpc_url };
    let proof = merkle_prover
        .get_storage_proof(
            DEFAULT_STORAGE_KEY_ETHEREUM,
            USDT_CONTRACT_ADDRESS,
            DEFAULT_ETH_BLOCK_HEIGHT,
        )
        .await;
    let proof_deserialized: EIP1186AccountProofResponse = serde_json::from_slice(&proof).unwrap();
    let raw_storage_proofs: Vec<(Vec<Vec<u8>>, JsonStorageKey)> = proof_deserialized
        .storage_proof
        .iter()
        .cloned()
        .map(|p| (p.proof.into_iter().map(|b| b.to_vec()).collect(), p.key))
        .collect();
    let first_proof = raw_storage_proofs.first().unwrap();
    assert_eq!(
        first_proof
            .1
            .as_b256()
            .bytes()
            .collect::<Result<Vec<u8>, _>>()
            .unwrap()
            .to_vec(),
        storage_key.to_vec()
    );
    EthereumProof {
        root: proof_deserialized.storage_hash.to_vec(),
        proof: first_proof.0.clone(),
        key: first_proof
            .1
            .as_b256()
            .bytes()
            .collect::<Result<Vec<u8>, _>>()
            .unwrap()
            .to_vec(),
        value: alloy_rlp::encode(&proof_deserialized.storage_proof.first().unwrap().value),
    }
}

#[cfg(feature = "web")]
pub async fn get_ethereum_test_vector_account_proof() -> EthereumProof {
    use alloy::rpc::types::EIP1186AccountProofResponse;
    use common::merkle::types::MerkleProver;
    let rpc_url = read_rpc_url() + &read_api_key();
    let state_root =
        hex::decode("0xf4da06dccd5bc3891b4d43b75e4a83ccea460f0bd5cde1901f368472e5ad7e4a").unwrap();
    let merkle_prover = EvmProver { rpc_url };
    let proof = merkle_prover
        .get_storage_proof(
            DEFAULT_STORAGE_KEY_ETHEREUM,
            USDT_CONTRACT_ADDRESS,
            DEFAULT_ETH_BLOCK_HEIGHT,
        )
        .await;
    let proof_deserialized: EIP1186AccountProofResponse = serde_json::from_slice(&proof).unwrap();
    let account_proof: Vec<Vec<u8>> = proof_deserialized
        .account_proof
        .iter()
        .map(|b| b.to_vec())
        .collect();
    EthereumProof {
        root: state_root.to_vec(),
        proof: account_proof.clone(),
        key: hex::decode(&USDT_CONTRACT_ADDRESS).unwrap(),
        value: account_proof.last().unwrap().to_vec(),
    }
}

#[cfg(feature = "web")]
#[tokio::test]
async fn test_get_receipt_proof() {
    use common::merkle::types::MerkleVerifiable;
    let rpc_url = read_rpc_url() + &read_api_key();
    let prover = EvmProver { rpc_url };
    let receipt_proof = prover
        // get a real ERC20 transfer
        .get_receipt_proof(
            "0xf03c8324b58076355c2e51bf354f3f8f95daf4a130f04794e245e98a972bf7ce",
            1,
        )
        .await;
    receipt_proof.verify(&receipt_proof.root);
}

#[cfg(feature = "web")]
pub fn read_api_key() -> String {
    dotenv().ok();
    env::var("INFURA").expect("Missing Infura API key!")
}

#[cfg(feature = "web")]
pub fn read_rpc_url() -> String {
    dotenv().ok();
    env::var("ETH_RPC").expect("Missing Ethereum RPC url!")
}
