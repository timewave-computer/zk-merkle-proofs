use {dotenvy::dotenv, std::env};

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use alloy::providers::{Provider, ProviderBuilder};
    use url::Url;

    use crate::merkle_lib::tests::defaults::read_sepolia_height;

    #[tokio::test]
    async fn test_get_receipt_proof() {
        use crate::merkle_lib::{tests::defaults::read_sepolia_url, types::MerkleProverEvm};
        use common::merkle::types::MerkleVerifiable;
        let rpc_url = read_sepolia_url();
        let prover = MerkleProverEvm { rpc_url };
        let receipt_proof = prover
            // erc20 transfers etc. will be located in the logs
            .get_receipt_proof(read_sepolia_height().await, 1)
            .await;

        let provider = ProviderBuilder::new().on_http(Url::from_str(&read_sepolia_url()).unwrap());
        let block = provider
            .get_block_by_number(alloy::eips::BlockNumberOrTag::Number(
                read_sepolia_height().await,
            ))
            .await
            .expect("Failed to get Block!")
            .expect("Block not found!");

        receipt_proof.verify(&block.header.receipts_root.to_vec(), 0);
    }
}

pub fn read_ethereum_vault_contract_address() -> String {
    dotenv().ok();
    env::var("ETHEREUM_SEPOLIA_VAULT_EXAMPLE_CONTRACT_ADDRESS")
        .expect("Missing Sepolia Vault Contract Address!")
}

pub fn read_sepolia_default_account_address() -> String {
    dotenv().ok();
    env::var("ETHEREUM_DEFAULT_ACCOUNT_ADDRESS").expect("Missing Ethereum Default Account Address!")
}

pub fn read_sepolia_url() -> String {
    dotenv().ok();
    env::var("ETHEREUM_URL").expect("Missing Sepolia url!")
}

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

pub fn get_test_vector_eth_storage_proof() -> Vec<u8> {
    include_bytes!("data/storage_proof.bin").to_vec()
}

pub fn get_test_vector_eth_account_proof() -> Vec<u8> {
    include_bytes!("data/account_proof.bin").to_vec()
}
