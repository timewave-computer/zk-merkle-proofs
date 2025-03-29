#[cfg(feature = "no-zkvm")]
#[cfg(test)]
mod tests {
    use crate::merkle_lib::{
        tests::defaults::constants::{
            get_test_vector_eth_account_proof, get_test_vector_eth_block_root,
        },
        types::EthereumMerkleProof,
    };
    use alloy_primitives::{Bytes, B256};
    use alloy_trie::{proof::verify_proof, Nibbles};

    #[test]
    fn test_verify_account_proof() {
        let block_root: Vec<u8> = get_test_vector_eth_block_root();
        let eth_proof: EthereumMerkleProof =
            serde_json::from_slice(&get_test_vector_eth_account_proof()).unwrap();
        let root_hash = B256::from_slice(&block_root);
        let proof_nodes: Vec<Bytes> = eth_proof
            .proof
            .iter()
            .map(|node| Bytes::copy_from_slice(node))
            .collect();
        let key = Nibbles::unpack(eth_proof.key);
        let result = verify_proof(
            root_hash,
            key,
            Some(eth_proof.value.to_vec()),
            proof_nodes.iter(),
        );
        assert!(result.is_ok(), "Proof verification failed: {:?}", result);
    }
}
