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
const USDT_CONTRACT_ADDRESS: &str = "0xdAC17F958D2ee523a2206206994597C13D831ec7";
const DEFAULT_STORAGE_KEY_ETHEREUM: &str =
    "0x0000000000000000000000000000000000000000000000000000000000000000";

#[cfg(feature = "web")]
pub async fn get_ethereum_test_vector_storage_proof() -> EthereumProof {
    let rpc_url = read_rpc_url() + &read_api_key();
    let prover = EvmProver { rpc_url };
    let provider = ProviderBuilder::new().on_http(Url::from_str(&prover.rpc_url).unwrap());
    let storage_key = FixedBytes::from_hex(DEFAULT_STORAGE_KEY_ETHEREUM).unwrap();
    let proof = provider
        .get_proof(
            Address::from_hex(USDT_CONTRACT_ADDRESS).unwrap(),
            vec![storage_key],
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
    // for now we only want one storage proof at a time - refactor this later
    assert_eq!(raw_storage_proofs.len(), 1);
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
        root: proof.storage_hash.to_vec(),
        proof: first_proof.0.clone(),
        key: first_proof
            .1
            .as_b256()
            .bytes()
            .collect::<Result<Vec<u8>, _>>()
            .unwrap()
            .to_vec(),
        // rlp encoded value
        value: alloy_rlp::encode(&proof.storage_proof.first().unwrap().value),
    }
}

#[cfg(feature = "web")]
pub async fn get_ethereum_test_vector_account_proof() -> EthereumProof {
    let rpc_url = read_rpc_url() + &read_api_key();
    let prover = EvmProver { rpc_url };
    let provider = ProviderBuilder::new().on_http(Url::from_str(&prover.rpc_url).unwrap());
    let storage_key = FixedBytes::from_hex(DEFAULT_STORAGE_KEY_ETHEREUM).unwrap();
    let block = provider
        .get_block_by_number(
            alloy::eips::BlockNumberOrTag::from(22040634),
            //alloy::rpc::types::BlockTransactionsKind::Full,
        )
        .await
        .expect("failed to get block")
        .expect("failed to get block");
    let block_state_root = block.header.state_root;
    //let storage_key = FixedBytes::from_hex(DEFAULT_STORAGE_KEY_ETHEREUM).unwrap();
    let proof = provider
        .get_proof(
            Address::from_hex(USDT_CONTRACT_ADDRESS).unwrap(),
            vec![storage_key],
        )
        .block_id(alloy::eips::BlockId::from(22040634))
        .await
        .expect("Failed to get proof!");
    let account_proof: Vec<Vec<u8>> = proof.account_proof.iter().map(|b| b.to_vec()).collect();
    EthereumProof {
        root: block_state_root.to_vec(),
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
fn read_api_key() -> String {
    dotenv().ok();
    env::var("INFURA").expect("Missing Infura API key!")
}

#[cfg(feature = "web")]
fn read_rpc_url() -> String {
    dotenv().ok();
    env::var("ETH_RPC").expect("Missing Ethereum RPC url!")
}
