#[cfg(feature = "no-sp1")]
#[cfg(test)]
mod tests {
    use crate::merkle_lib::{
        keccak::digest_keccak,
        tests::defaults::{
            get_ethereum_storage_proof, read_ethereum_vault_contract_address,
            read_sepolia_default_account_address, read_sepolia_height,
        },
        types::EthereumMerkleProof,
    };
    use alloy::hex::FromHex;
    use alloy_primitives::U256;
    use alloy_sol_types::SolValue;
    use common::merkle::types::MerkleVerifiable;

    #[tokio::test]
    async fn test_vault_contract_balance_on_sepolia() {
        let address =
            alloy_primitives::Address::from_hex(read_sepolia_default_account_address()).unwrap();
        let slot: U256 = alloy_primitives::U256::from(0);
        let encoded_key = (address, slot).abi_encode();
        let keccak_key = digest_keccak(&encoded_key).to_vec();
        let mut eth_proof: EthereumMerkleProof = get_ethereum_storage_proof(
            &alloy::hex::encode(keccak_key),
            &read_ethereum_vault_contract_address(),
            read_sepolia_height().await,
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
    async fn test_vault_contract_shares_on_sepolia() {
        use alloy::hex;
        let storage_slot_key =
            hex::decode("0x0000000000000000000000000000000000000000000000000000000000000001")
                .unwrap();
        let mut eth_proof: EthereumMerkleProof = get_ethereum_storage_proof(
            &alloy::hex::encode(storage_slot_key),
            &read_ethereum_vault_contract_address(),
            read_sepolia_height().await,
        )
        .await;
        let leaf = eth_proof.proof.last().unwrap().to_owned();
        let leaf_decoded: Vec<alloy_primitives::Bytes> = alloy_rlp::decode_exact(&leaf).unwrap();
        let value_encoded = leaf_decoded.get(1).unwrap();
        assert_eq!(value_encoded.to_vec(), eth_proof.value);
        eth_proof.hash_key();
        eth_proof.verify(&eth_proof.root.to_vec());
    }
}
