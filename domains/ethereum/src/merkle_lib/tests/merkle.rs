#[cfg(test)]
mod tests {
    use alloy::hex;
    use common::merkle::types::MerkleVerifiable;

    use crate::merkle_lib::{
        tests::defaults::{get_test_vector_eth_account_proof, get_test_vector_eth_storage_proof},
        types::EthereumMerkleProof,
    };

    const BLOCK_ROOT: &str = "0x55abcc2a0c3779634d5f91adadff34aaddf8d3777b13187f511eb4f177d16e78";
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
