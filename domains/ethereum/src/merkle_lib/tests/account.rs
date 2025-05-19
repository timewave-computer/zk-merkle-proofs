#[cfg(feature = "no-zkvm")]
#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{
        ethereum_rpc::rpc::EvmMerkleRpcClient,
        merkle_lib::{
            tests::defaults::constants::{
                get_test_vector_eth_account_proof, get_test_vector_eth_block_root,
                read_sepolia_height, read_sepolia_url,
            },
            types::{rlp_decode_account, EthereumMerkleProof},
        },
        timewave_rlp::alloy_bytes::Bytes,
        timewave_trie::verify::verify_proof,
    };
    use alloy::providers::{Provider, ProviderBuilder};
    use common::merkle::types::MerkleVerifiable;
    use hex::FromHex;
    use nybbles::Nibbles;
    use tracing::info;
    use url::Url;

    #[test]
    fn test_verify_account_proof() {
        let block_root: Vec<u8> = get_test_vector_eth_block_root();
        let eth_proof: EthereumMerkleProof =
            serde_json::from_slice(&get_test_vector_eth_account_proof()).unwrap();
        let proof_nodes: Vec<Bytes> = eth_proof
            .proof
            .iter()
            .map(|node| Bytes::copy_from_slice(node))
            .collect();
        let key = Nibbles::unpack(eth_proof.key);
        let result = verify_proof(
            &block_root.try_into().unwrap(),
            key,
            Some(eth_proof.value.to_vec()),
            proof_nodes.iter(),
        );
        assert!(result.is_ok(), "Proof verification failed: {:?}", result);
    }

    #[tokio::test]
    async fn test_account_balance_proof() {
        tracing_subscriber::fmt::init();

        let sepolia_height = read_sepolia_height().await.unwrap();
        let address =
            alloy_primitives::Address::from_hex("0x89efEA02Dc92FD8CcCEefabb59a1104759dF352d")
                .unwrap();

        let url = read_sepolia_url();
        let provider = ProviderBuilder::new().on_http(Url::from_str(&url).unwrap());

        let merkle_prover = EvmMerkleRpcClient {
            rpc_url: url.to_string(),
        };

        let account_proof = merkle_prover
            .get_account_proof(&address.to_string(), sepolia_height)
            .await
            .unwrap();

        let block = provider
            .get_block_by_number(alloy::eips::BlockNumberOrTag::Number(sepolia_height))
            .await
            .unwrap()
            .unwrap();

        let state_root = block.header.state_root.as_slice();

        let account_decoded = rlp_decode_account(&account_proof.value).unwrap();
        info!("Account Decoded: {:?}", account_decoded);

        assert!(account_proof.verify(&state_root).unwrap());
    }
}
