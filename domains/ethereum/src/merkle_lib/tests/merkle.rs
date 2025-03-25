#[cfg(test)]
mod tests {
    use alloy::hex;
    use common::merkle::types::MerkleVerifiable;

    use crate::merkle_lib::{
        tests::defaults::{get_test_vector_eth_account_proof, get_test_vector_eth_storage_proof},
        types::EthereumMerkleProof,
    };

    const BLOCK_ROOT: &str = "0x019e551f35a2b407b2b316e42a6652242775ef16242e66c9f140fa0608e7243e";
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
