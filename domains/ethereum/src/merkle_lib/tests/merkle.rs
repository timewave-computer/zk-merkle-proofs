#[cfg(test)]
#[cfg(feature = "no-sp1")]
mod tests {
    use crate::merkle_lib::{
        tests::defaults::{get_test_vector_eth_account_proof, get_test_vector_eth_storage_proof},
        types::EthereumMerkleProof,
    };
    use common::merkle::types::MerkleVerifiable;

    #[tokio::test]
    async fn test_verify_account_proof() {
        let mut eth_proof: EthereumMerkleProof =
            serde_json::from_slice(&get_test_vector_eth_account_proof()).unwrap();
        eth_proof.hash_key();
        eth_proof.verify(&eth_proof.root.to_vec());
    }

    #[tokio::test]
    async fn test_verify_storage_proof() {
        let mut eth_proof: EthereumMerkleProof =
            serde_json::from_slice(&get_test_vector_eth_storage_proof()).unwrap();
        eth_proof.hash_key();
        eth_proof.verify(&eth_proof.root.to_vec());
    }
}
