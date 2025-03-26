#[cfg(test)]
mod tests {
    use crate::merkle_lib::{
        tests::defaults::{get_test_vector_eth_account_proof, get_test_vector_eth_storage_proof},
        types::EthereumMerkleProof,
    };
    use common::merkle::types::MerkleVerifiable;

    #[tokio::test]
    async fn test_verify_account_proof() {
        let block_root: Vec<u8> = vec![
            118, 40, 34, 157, 173, 69, 97, 39, 178, 229, 149, 0, 102, 49, 6, 23, 193, 155, 138,
            107, 206, 30, 43, 33, 235, 240, 232, 137, 63, 146, 55, 211,
        ];
        let eth_proof: EthereumMerkleProof =
            serde_json::from_slice(&get_test_vector_eth_account_proof()).unwrap();
        assert!(eth_proof.verify(&block_root));
    }

    #[tokio::test]
    async fn test_verify_storage_proof() {
        let account_root: Vec<u8> = vec![
            220, 247, 143, 8, 23, 115, 27, 234, 201, 222, 6, 127, 108, 199, 142, 189, 143, 170,
            229, 114, 198, 145, 98, 162, 74, 233, 250, 165, 112, 210, 250, 206,
        ];
        let eth_proof: EthereumMerkleProof =
            serde_json::from_slice(&get_test_vector_eth_storage_proof()).unwrap();
        assert!(eth_proof.verify(&account_root));
    }
}
