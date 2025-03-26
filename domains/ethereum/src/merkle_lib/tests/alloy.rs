// block hash example: 0xc3e7838359382f8ecc52ec0d8951c4c76a55524494eff38b93f317221ef27f73
// get the balance of a user in a contract from a key on eth
// get the balance of a user from the bank store  on neutron
// do the calculation and construct the messages for either side accordingly

// an example using the alloy-trie instead of eth-trie
// we can consider replacing the eth-trie with the alloy-trie,
// but I dislike the API so let's only consider this if the alloy-trie
// offers better performance.

// since currently performance is not our top-priority, let's
// delay this decision and stick with eth_trie for now.
// It's always good to have an alternative in case something breaks.
#[cfg(feature = "no-zkvm")]
#[cfg(test)]
mod tests {
    use crate::merkle_lib::{
        tests::defaults::{get_test_vector_eth_account_proof, get_test_vector_eth_block_root},
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
