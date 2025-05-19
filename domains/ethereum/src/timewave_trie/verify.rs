//! Proof verification logic for Ethereum state trie.
//!
//! This module provides functionality for verifying Merkle proofs against the
//! Ethereum state trie. It handles the verification of account proofs, storage
//! proofs, and receipt proofs.

use core::ops::Deref;

use crate::timewave_rlp::{Decodable, EMPTY_STRING_CODE};
use crate::{
    timewave_rlp::{self, alloy_bytes::Bytes},
    timewave_trie::{
        constants::{CHILD_INDEX_RANGE, EMPTY_ROOT_HASH_BYTES},
        types::{BranchNode, RlpNode, TrieNode},
    },
};

/// Errors that can occur during proof verification.
///
/// This enum represents the various ways in which a proof verification can fail,
/// including root mismatches, value mismatches, and decoding errors.
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
/// This function verifies that a given key-value pair exists in the state trie
/// by checking the provided proof against the expected state root. It traverses
/// the proof nodes and verifies that they form a valid path from the leaf to
/// the root.
///
/// # Arguments
/// * `root` - The expected state root hash to verify against
/// * `key` - The key to verify the proof for
/// * `expected_value` - The expected value for the key, or None for exclusion proofs
/// * `proof` - An iterator over the proof nodes
///
/// # Returns
/// * `Ok(())` if the proof is valid
/// * `Err(ProofVerificationError)` if the proof is invalid
///
/// # Errors
/// * `RootMismatch` if the computed root doesn't match the expected root
/// * `ValueMismatch` if the value doesn't match the expected value
/// * `UnexpectedEmptyRoot` if an empty root node is encountered unexpectedly
/// * `Rlp` if there's an error decoding the RLP data
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
    // If the proof is empty or contains only an empty node, the expected value must be None.
    if proof
        .peek()
        .map(|node| node.as_ref() == [EMPTY_STRING_CODE])
        .unwrap_or(true)
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

/// Result of decoding a trie node during proof verification.
///
/// This enum represents the possible outcomes when decoding a trie node during
/// proof verification. It can either be a node that needs further processing
/// or a value that has been found.
#[derive(Debug, PartialEq, Eq)]
enum NodeDecodingResult {
    /// A node that needs further processing
    Node(RlpNode),
    /// A value that has been found
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

/// Process a branch node during proof verification.
///
/// This function handles the processing of a branch node during proof verification,
/// traversing the appropriate child node based on the current path.
///
/// # Arguments
/// * `branch` - The branch node to process
/// * `walked_path` - The path that has been traversed so far
/// * `key` - The complete key being verified
///
/// # Returns
/// * `Ok(Some(NodeDecodingResult))` if a node or value was found
/// * `Ok(None)` if no matching node was found
/// * `Err(ProofVerificationError)` if an error occurred during processing
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
