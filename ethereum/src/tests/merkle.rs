#[cfg(test)]
#[cfg(feature = "web")]
mod tests {
    use alloy::{
        consensus::Account,
        hex::{self, FromHex, ToHexExt},
        primitives::{Address, FixedBytes},
        providers::{Provider, ProviderBuilder},
        rpc::types::EIP1186AccountProofResponse,
        serde::JsonStorageKey,
    };
    use common::MerkleProver;
    use dotenvy::dotenv;
    use std::{env, io::Read, str::FromStr};
    use url::Url;

    use crate::{keccak::digest_keccak, verify_merkle_proof, EvmProver};
    const USDT_CONTRACT_ADDRESS: &str = "0xdAC17F958D2ee523a2206206994597C13D831ec7";
    const DEFAULT_STORAGE_KEY_ETHEREUM: &str =
        "0x0000000000000000000000000000000000000000000000000000000000000000";
    #[tokio::test]
    // first verifies account state, then a single storage proof
    async fn test_verify_storage_proof_single() {
        let rpc_url = read_rpc_url() + &read_api_key();
        let prover = EvmProver { rpc_url };
        let provider = ProviderBuilder::new().on_http(Url::from_str(&prover.rpc_url).unwrap());
        let block = provider
            .get_block(
                alloy::eips::BlockId::Number(provider.get_block_number().await.unwrap().into()),
                alloy::rpc::types::BlockTransactionsKind::Full,
            )
            .await
            .unwrap()
            .unwrap();
        let proof = provider
            .get_proof(
                Address::from_hex(USDT_CONTRACT_ADDRESS).unwrap(),
                vec![FixedBytes::from_hex(DEFAULT_STORAGE_KEY_ETHEREUM).unwrap()],
            )
            .await
            .expect("Failed to get proof!");
        let account_proof: Vec<u8> = verify_merkle_proof(
            block.header.state_root.to_vec(),
            proof
                .account_proof
                .clone()
                .into_iter()
                .map(|b| b.to_vec())
                .collect(),
            &digest_keccak(&hex::decode(USDT_CONTRACT_ADDRESS).unwrap()),
        );
        let decoded_account: Account = alloy_rlp::decode_exact(&account_proof).unwrap();
        assert_eq!(
            decoded_account.storage_root.encode_hex(),
            hex::encode(&proof.storage_hash)
        );
        let storage_proof = prover
            .get_storage_proof(vec![DEFAULT_STORAGE_KEY_ETHEREUM], USDT_CONTRACT_ADDRESS)
            .await;
        let storage_proof_deserialized: EIP1186AccountProofResponse =
            serde_json::from_slice(&storage_proof).unwrap();
        let raw_storage_proofs: Vec<(Vec<Vec<u8>>, JsonStorageKey)> = storage_proof_deserialized
            .storage_proof
            .iter()
            .cloned()
            .map(|p| (p.proof.into_iter().map(|b| b.to_vec()).collect(), p.key))
            .collect();
        let first_proof = raw_storage_proofs.first().unwrap();
        verify_merkle_proof(
            proof.storage_hash.to_vec(),
            first_proof.0.clone(),
            &digest_keccak(
                &first_proof
                    .1
                    .as_b256()
                    .bytes()
                    .collect::<Result<Vec<u8>, _>>()
                    .unwrap(),
            ),
        );
    }

    // todo: add a test for a transaction receipt e.g. verify (erc20) events

    fn read_api_key() -> String {
        dotenv().ok();
        env::var("INFURA").expect("Missing Infura API key!")
    }

    fn read_rpc_url() -> String {
        dotenv().ok();
        env::var("ETH_RPC").expect("Missing Infura API key!")
    }
}
