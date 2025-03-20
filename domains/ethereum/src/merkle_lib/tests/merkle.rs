#[cfg(test)]
#[cfg(feature = "no-sp1")]
mod tests {
    use crate::merkle_lib::{
        tests::defaults::{get_ethereum_test_vector_storage_proof, TEST_VECTOR_ETH_ACCOUNT_PROOF},
        types::EthereumMerkleProof,
    };
    use common::merkle::types::MerkleVerifiable;

    #[tokio::test]
    async fn test_verify_storage_proof_single() {
        let mut eth_proof: EthereumMerkleProof = get_ethereum_test_vector_storage_proof().await;
        let leaf = eth_proof.proof.last().unwrap().to_owned();
        let leaf_decoded: Vec<alloy_primitives::Bytes> = alloy_rlp::decode_exact(&leaf).unwrap();
        let value_encoded = leaf_decoded.get(1).unwrap();
        assert_eq!(value_encoded.to_vec(), eth_proof.value);
        eth_proof.hash_key();
        eth_proof.verify(&eth_proof.root.to_vec());
    }

    #[cfg(feature = "no-sp1")]
    #[tokio::test]
    async fn test_counter_contract_on_sepolia() {
        use crate::merkle_lib::tests::defaults::{get_ethereum_storage_proof, read_sepolia_height};
        let mut eth_proof: EthereumMerkleProof = get_ethereum_storage_proof(
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            "0x567A801BdE180DFDd25Ab71e75630B9e26b757e1",
            read_sepolia_height(),
        )
        .await;
        let leaf = eth_proof.proof.last().unwrap().to_owned();
        let leaf_decoded: Vec<alloy_primitives::Bytes> = alloy_rlp::decode_exact(&leaf).unwrap();
        let value_encoded = leaf_decoded.get(1).unwrap();
        assert_eq!(value_encoded.to_vec(), eth_proof.value);
        eth_proof.hash_key();
        eth_proof.verify(&eth_proof.root.to_vec());
    }

    #[cfg(feature = "no-sp1")]
    #[tokio::test]
    async fn test_stored_value_from_dictionary_on_sepolia() {
        use crate::merkle_lib::{
            keccak::digest_keccak,
            tests::defaults::{
                get_ethereum_storage_proof, read_sepolia_default_account_address,
                read_sepolia_height,
            },
        };
        use alloy::hex::FromHex;
        use alloy_primitives::U256;
        use alloy_sol_types::SolValue;

        let address =
            alloy_primitives::Address::from_hex(read_sepolia_default_account_address()).unwrap();
        let slot: U256 = alloy_primitives::U256::from(0);
        let encoded_key = (address, slot).abi_encode();
        let keccak_key = digest_keccak(&encoded_key).to_vec();
        let mut eth_proof: EthereumMerkleProof = get_ethereum_storage_proof(
            &alloy::hex::encode(keccak_key),
            "0x8119a4eCD758D2B9B5f06D813BdE7e7aba323A4E",
            read_sepolia_height(),
        )
        .await;
        let leaf = eth_proof.proof.last().unwrap().to_owned();
        let leaf_decoded: Vec<alloy_primitives::Bytes> = alloy_rlp::decode_exact(&leaf).unwrap();
        let value_encoded = leaf_decoded.get(1).unwrap();
        assert_eq!(value_encoded.to_vec(), eth_proof.value);
        eth_proof.hash_key();
        eth_proof.verify(&eth_proof.root.to_vec());
    }

    #[tokio::test]
    async fn test_verify_account_proof_single() {
        let mut eth_proof: EthereumMerkleProof =
            serde_json::from_slice(&TEST_VECTOR_ETH_ACCOUNT_PROOF).unwrap();
        eth_proof.hash_key();
        eth_proof.verify(&eth_proof.root.to_vec());
    }
}
