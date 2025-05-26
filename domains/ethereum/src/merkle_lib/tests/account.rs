#[cfg(feature = "no-zkvm")]
#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{
        ethereum_rpc::rpc::EvmMerkleRpcClient,
        merkle_lib::{
            tests::defaults::constants::{read_sepolia_height, read_sepolia_url},
            types::EthereumAccount,
            RlpDecodable,
        },
    };
    use alloy::providers::{Provider, ProviderBuilder};
    use common::merkle::types::MerkleVerifiable;
    use hex::FromHex;
    use tracing::info;
    use url::Url;

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

        println!("Account Proof: {:?}", account_proof.address);

        let block = provider
            .get_block_by_number(alloy::eips::BlockNumberOrTag::Number(sepolia_height))
            .await
            .unwrap()
            .unwrap();

        let state_root = block.header.state_root.as_slice();

        let account_decoded = EthereumAccount::rlp_decode(&account_proof.value).unwrap();
        info!("Account Decoded: {:?}", account_decoded);

        assert!(account_proof.verify(&state_root).unwrap());
    }
}
