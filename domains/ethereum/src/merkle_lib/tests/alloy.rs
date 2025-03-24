// block hash example: 0xc3e7838359382f8ecc52ec0d8951c4c76a55524494eff38b93f317221ef27f73
// get the balance of a user in a contract from a key on eth
// get the balance of a user from the bank store  on neutron
// do the calculation and construct the messages for either side accordingly

// an example using the alloy-trie instead of eth-trie
// we can consider replacing the eth-trie with the alloy-trie,
// but I dislike the API so let's only consider this if the alloy-trie
// offers better performance.

// since currently prrformance is not our top-priority, let's
// delay this decision and stick with eth_trie for now.
// It's always good to have an alternative in case something breaks.
#[cfg(all(feature = "no-sp1", feature = "no-sp1"))]
#[cfg(test)]
mod tests {
    use crate::merkle_lib::keccak::digest_keccak;
    use alloy::hex;
    use alloy_primitives::{Bytes, FixedBytes, B256};
    use alloy_trie::{
        proof::{ProofNodes, ProofRetainer},
        HashBuilder, Nibbles,
    };
    use std::str::FromStr;
    #[test]
    fn test_alloy_trie() {
        let existing_keys = [
            hex!("0000000000000000000000000000000000000000000000000000000000000000"),
            hex!("3a00000000000000000000000000000000000000000000000000000000000000"),
            hex!("3c15000000000000000000000000000000000000000000000000000000000000"),
        ];
        let target = Nibbles::unpack(
            B256::from_str("0x3c19000000000000000000000000000000000000000000000000000000000000")
                .unwrap(),
        );
        let value = B256::with_last_byte(1);
        let retainer = ProofRetainer::from_iter([target.clone()]);
        let mut hash_builder = HashBuilder::default().with_proof_retainer(retainer);
        for key in &existing_keys {
            hash_builder.add_leaf(Nibbles::unpack(B256::from_slice(key)), &value[..]);
        }
        let root = hash_builder.root();
        assert_eq!(
            root,
            triehash_trie_root(existing_keys.map(|key| (B256::from_slice(&key), value)))
        );
        let proof = hash_builder.take_proof_nodes();
        assert_eq!(proof, ProofNodes::from_iter([
            (Nibbles::default(), Bytes::from_str("f851a0c530c099d779362b6bd0be05039b51ccd0a8ed39e0b2abacab8fe0e3441251878080a07d4ee4f073ae7ce32a6cbcdb015eb73dd2616f33ed2e9fb6ba51c1f9ad5b697b80808080808080808080808080").unwrap()),
            (Nibbles::from_vec(vec![0x3]), Bytes::from_str("f85180808080808080808080a057fcbd3f97b1093cd39d0f58dafd5058e2d9f79a419e88c2498ff3952cb11a8480a07520d69a83a2bdad373a68b2c9c8c0e1e1c99b6ec80b4b933084da76d644081980808080").unwrap()),
            (Nibbles::from_vec(vec![0x3, 0xc]), Bytes::from_str("f842a02015000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000001").unwrap())
        ]));
    }
    fn triehash_trie_root<I, K, V>(iter: I) -> B256
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<[u8]> + Ord,
        V: AsRef<[u8]>,
    {
        struct Keccak256Hasher;
        impl hash_db::Hasher for Keccak256Hasher {
            type Out = B256;
            type StdHasher = plain_hasher::PlainHasher;
            const LENGTH: usize = 32;
            fn hash(x: &[u8]) -> Self::Out {
                FixedBytes::from_slice(&digest_keccak(&x))
            }
        }
        triehash::trie_root::<Keccak256Hasher, _, _, _>(iter)
    }
}
