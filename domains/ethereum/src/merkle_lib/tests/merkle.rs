#[cfg(test)]
mod tests {
    use alloy::hex;
    use common::merkle::types::MerkleVerifiable;

    use crate::merkle_lib::{
        tests::defaults::{get_test_vector_eth_account_proof, get_test_vector_eth_storage_proof},
        types::EthereumMerkleProof,
    };

    const BLOCK_ROOT: &str = "0xe93cb3564551bb8ec869ecd49bbc4f86e7670f495d4b3d5d1632357bd6ee21ac";
    const ACCOUNT_ROOT: &str = "0xdcf78f0817731beac9de067f6cc78ebd8faae572c69162a24ae9faa570d2face";

    #[tokio::test]
    async fn test_verify_account_proof() {
        let eth_proof: EthereumMerkleProof =
            serde_json::from_slice(&get_test_vector_eth_account_proof()).unwrap();
        eth_proof.verify(hex::decode(BLOCK_ROOT).unwrap().as_slice(), 0);
    }

    #[tokio::test]
    async fn test_verify_storage_proof() {
        let eth_proof: EthereumMerkleProof =
            serde_json::from_slice(&get_test_vector_eth_storage_proof()).unwrap();
        eth_proof.verify(hex::decode(ACCOUNT_ROOT).unwrap().as_slice(), 0);
    }
}
