//! Proof verification logic.

use core::ops::Deref;

use crate::timewave_rlp::{Decodable, EMPTY_STRING_CODE};
use crate::{
    timewave_rlp::{self, alloy_bytes::Bytes},
    timewave_trie::types::{BranchNode, RlpNode, TrieNode, CHILD_INDEX_RANGE},
};

#[derive(PartialEq, Eq, Debug)]
pub enum ProofVerificationError {
    /// State root does not match the expected.
    RootMismatch {
        /// Computed state root.
        got: [u8; 32],
        /// State root provided to verify function.
        expected: [u8; 32],
    },
    /// The node value does not match at specified path.
    ValueMismatch {
        /// Path at which error occurred.
        path: Nibbles,
        /// Value in the proof.
        got: Option<Bytes>,
        /// Expected value.
        expected: Option<Bytes>,
    },
    /// Encountered unexpected empty root node.
    UnexpectedEmptyRoot,
    /// Error during RLP decoding of trie node.
    Rlp(timewave_rlp::Error),
}

extern crate alloc;
use alloc::vec::Vec;
use nybbles::Nibbles;

/// Verify the proof for given key value pair against the provided state root.
///
/// The expected node value can be either [Some] if it's expected to be present
/// in the tree or [None] if this is an exclusion proof.
pub fn verify_proof<'a, I>(
    root: &[u8; 32],
    key: Nibbles,
    expected_value: Option<Vec<u8>>,
    proof: I,
) -> Result<(), ProofVerificationError>
where
    I: IntoIterator<Item = &'a Bytes>,
{
    let mut proof = proof.into_iter().peekable();

    const EMPTY_ROOT_HASH: &str =
        "56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421";
    #[allow(non_snake_case)]
    let EMPTY_ROOT_HASH_BYTES: [u8; 32] = hex::decode(EMPTY_ROOT_HASH)
        .expect("Invalid hex or wrong length")
        .try_into()
        .expect("Invalid hex or wrong length");

    // If the proof is empty or contains only an empty node, the expected value must be None.
    if proof
        .peek()
        .map_or(true, |node| node.as_ref() == [EMPTY_STRING_CODE])
    {
        return if root == &EMPTY_ROOT_HASH_BYTES {
            if expected_value.is_none() {
                Ok(())
            } else {
                Err(ProofVerificationError::ValueMismatch {
                    path: key,
                    got: None,
                    expected: expected_value.map(Bytes::from),
                })
            }
        } else {
            Err(ProofVerificationError::RootMismatch {
                got: EMPTY_ROOT_HASH_BYTES,
                expected: *root,
            })
        };
    }

    let mut walked_path = Nibbles::with_capacity(key.len());
    let mut last_decoded_node = Some(NodeDecodingResult::Node(RlpNode::word_rlp(root)));
    for node in proof {
        // Check if the node that we just decoded (or root node, if we just started) matches
        // the expected node from the proof.
        if Some(RlpNode::from_rlp(node).as_slice()) != last_decoded_node.as_deref() {
            let got = Some(Bytes::copy_from_slice(node));
            let expected = last_decoded_node.as_deref().map(Bytes::copy_from_slice);
            return Err(ProofVerificationError::ValueMismatch {
                path: walked_path,
                got,
                expected,
            });
        }

        // Decode the next node from the proof.
        last_decoded_node = match TrieNode::decode(&mut &node[..]).unwrap() {
            TrieNode::Branch(branch) => process_branch(branch, &mut walked_path, &key)?,
            TrieNode::Extension(extension) => {
                walked_path.extend_from_slice(&extension.key);
                Some(NodeDecodingResult::Node(extension.child))
            }
            TrieNode::Leaf(leaf) => {
                walked_path.extend_from_slice(&leaf.key);
                Some(NodeDecodingResult::Value(leaf.value))
            }
            TrieNode::EmptyRoot => return Err(ProofVerificationError::UnexpectedEmptyRoot),
        };
    }

    // Last decoded node should have the key that we are looking for.
    last_decoded_node = last_decoded_node.filter(|_| walked_path == key);
    if last_decoded_node.as_deref() == expected_value.as_deref() {
        Ok(())
    } else {
        Err(ProofVerificationError::ValueMismatch {
            path: key,
            got: last_decoded_node.as_deref().map(Bytes::copy_from_slice),
            expected: expected_value.map(Bytes::from),
        })
    }
}

/// The result of decoding a node from the proof.
///
/// - [`TrieNode::Branch`] is decoded into a [`NodeDecodingResult::Value`] if the node at the
///   specified nibble was decoded into an in-place encoded [`TrieNode::Leaf`], or into a
///   [`NodeDecodingResult::Node`] otherwise.
/// - [`TrieNode::Extension`] is always decoded into a [`NodeDecodingResult::Node`].
/// - [`TrieNode::Leaf`] is always decoded into a [`NodeDecodingResult::Value`].
#[derive(Debug, PartialEq, Eq)]
enum NodeDecodingResult {
    Node(RlpNode),
    Value(Vec<u8>),
}

impl Deref for NodeDecodingResult {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Node(node) => node.as_slice(),
            Self::Value(value) => value,
        }
    }
}

#[inline]
fn process_branch(
    mut branch: BranchNode,
    walked_path: &mut Nibbles,
    key: &Nibbles,
) -> Result<Option<NodeDecodingResult>, ProofVerificationError> {
    if let Some(next) = key.get(walked_path.len()) {
        let mut stack_ptr = branch.as_ref().first_child_index();
        for index in CHILD_INDEX_RANGE {
            if branch.state_mask.is_bit_set(index) {
                if index == *next {
                    walked_path.push(*next);

                    let child = branch.stack.remove(stack_ptr);
                    if child.len() == 33 {
                        return Ok(Some(NodeDecodingResult::Node(child)));
                    } else {
                        // This node is encoded in-place.
                        match TrieNode::decode(&mut &child[..]).unwrap() {
                            TrieNode::Branch(child_branch) => {
                                // An in-place branch node can only have direct, also in-place
                                // encoded, leaf children, as anything else overflows this branch
                                // node, making it impossible to be encoded in-place in the first
                                // place.
                                return process_branch(child_branch, walked_path, key);
                            }
                            TrieNode::Extension(child_extension) => {
                                walked_path.extend_from_slice(&child_extension.key);

                                // If the extension node's child is a hash, the encoded extension
                                // node itself wouldn't fit for encoding in-place. So this extension
                                // node must have a child that is also encoded in-place.
                                //
                                // Since the child cannot be a leaf node (otherwise this node itself
                                // would be a leaf node, not an extension node), the child must be a
                                // branch node encoded in-place.
                                match TrieNode::decode(&mut &child_extension.child[..]).unwrap() {
                                    TrieNode::Branch(extension_child_branch) => {
                                        return process_branch(
                                            extension_child_branch,
                                            walked_path,
                                            key,
                                        );
                                    }
                                    node @ (TrieNode::EmptyRoot
                                    | TrieNode::Extension(_)
                                    | TrieNode::Leaf(_)) => {
                                        unreachable!("unexpected extension node child: {node:?}")
                                    }
                                }
                            }
                            TrieNode::Leaf(child_leaf) => {
                                walked_path.extend_from_slice(&child_leaf.key);
                                return Ok(Some(NodeDecodingResult::Value(child_leaf.value)));
                            }
                            TrieNode::EmptyRoot => {
                                return Err(ProofVerificationError::UnexpectedEmptyRoot)
                            }
                        }
                    };
                }
                stack_ptr += 1;
            }
        }
    }

    Ok(None)
}
