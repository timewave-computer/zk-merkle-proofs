#[cfg(feature = "no-sp1")]
use {
    crate::{merkle_lib::types::EthereumMerkleProof, merkle_lib::types::MerkleProverEvm},
    alloy::{hex::FromHex, serde::JsonStorageKey},
    alloy_primitives::FixedBytes,
    dotenvy::dotenv,
    std::{env, io::Read},
};

#[cfg(feature = "no-sp1")]
pub async fn get_ethereum_storage_proof(
    key: &str,
    address: &str,
    height: u64,
) -> EthereumMerkleProof {
    use alloy::rpc::types::EIP1186AccountProofResponse;
    use common::merkle::types::MerkleProver;
    let rpc_url = read_sepolia_url();
    let storage_key: FixedBytes<32> = FixedBytes::from_hex(key).unwrap();
    let merkle_prover = MerkleProverEvm { rpc_url };
    let proof = merkle_prover
        .get_merkle_proof_from_rpc(key, address, height)
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
    EthereumMerkleProof {
        root: proof_deserialized.storage_hash.to_vec(),
        proof: first_proof.0.clone(),
        key: first_proof
            .1
            .as_b256()
            .bytes()
            .collect::<Result<Vec<u8>, _>>()
            .unwrap()
            .to_vec(),
        value: alloy_rlp::encode(proof_deserialized.storage_proof.first().unwrap().value),
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "no-sp1")]
    #[tokio::test]
    async fn test_get_receipt_proof() {
        use crate::merkle_lib::{tests::defaults::read_sepolia_url, types::MerkleProverEvm};
        use common::merkle::types::MerkleVerifiable;
        let rpc_url = read_sepolia_url();
        let prover = MerkleProverEvm { rpc_url };
        let receipt_proof = prover
            // erc20 transfers etc. will be located in the logs
            .get_receipt_proof(
                "0x529f947c793dee682b977b5904f1f48571b3176d015baa5055ace2f3661234d5",
                1,
            )
            .await;
        receipt_proof.verify(&receipt_proof.root);
    }
}

#[cfg(feature = "no-sp1")]
pub fn read_ethereum_vault_contract_address() -> String {
    dotenv().ok();
    env::var("ETHEREUM_SEPOLIA_VAULT_EXAMPLE_CONTRACT_ADDRESS")
        .expect("Missing Sepolia Vault Contract Address!")
}

#[cfg(feature = "no-sp1")]
pub fn read_sepolia_default_account_address() -> String {
    dotenv().ok();
    env::var("ETHEREUM_DEFAULT_ACCOUNT_ADDRESS").expect("Missing Ethereum Default Account Address!")
}

#[cfg(feature = "no-sp1")]
pub fn read_sepolia_url() -> String {
    dotenv().ok();
    env::var("ETHEREUM_URL").expect("Missing Sepolia url!")
}

#[cfg(feature = "no-sp1")]
pub async fn read_sepolia_height() -> u64 {
    use alloy::providers::{Provider, ProviderBuilder};
    use std::str::FromStr;
    use url::Url;
    let provider = ProviderBuilder::new().on_http(Url::from_str(&read_sepolia_url()).unwrap());
    let block = provider
        .get_block_by_number(
            alloy::eips::BlockNumberOrTag::Latest, // for alloy < 0.12
                                                   //alloy::rpc::types::BlockTransactionsKind::Full,
        )
        .await
        .expect("Failed to get Block!")
        .expect("Block not found!");
    block.header.number.into()
}

use std::fs;
use std::path::PathBuf;

fn read_bytes_from_file(path: &str) -> std::io::Result<Vec<u8>> {
    fs::read(path)
}

pub fn get_test_vector_eth_storage_proof() -> Vec<u8> {
    let path: PathBuf = [
        env!("CARGO_MANIFEST_DIR"),
        "src/merkle_lib/tests/data/storage_proof.bin",
    ]
    .iter()
    .collect();
    read_bytes_from_file(path.to_str().unwrap()).unwrap()
}

pub fn get_test_vector_eth_account_proof() -> Vec<u8> {
    let path: PathBuf = [
        env!("CARGO_MANIFEST_DIR"),
        "src/merkle_lib/tests/data/account_proof.bin",
    ]
    .iter()
    .collect();
    read_bytes_from_file(path.to_str().unwrap()).unwrap()
}
