#[cfg(test)]
mod tests {
    use crate::merkle_lib::{
        tests::defaults::constants::{
            get_test_vector_eth_account_proof, get_test_vector_eth_account_root,
            get_test_vector_eth_block_root, get_test_vector_eth_storage_proof,
        },
        types::EthereumMerkleProof,
    };
    use common::merkle::types::MerkleVerifiable;

    #[tokio::test]
    async fn test_verify_account_proof() {
        let block_root: Vec<u8> = get_test_vector_eth_block_root();
        let eth_proof: EthereumMerkleProof =
            serde_json::from_slice(&get_test_vector_eth_account_proof()).unwrap();
        assert!(eth_proof.verify(&block_root).unwrap());
    }

    #[tokio::test]
    async fn test_verify_storage_proof() {
        let account_root: Vec<u8> = get_test_vector_eth_account_root();
        let eth_proof: EthereumMerkleProof =
            serde_json::from_slice(&get_test_vector_eth_storage_proof()).unwrap();
        assert!(eth_proof.verify(&account_root).unwrap());
    }
}
