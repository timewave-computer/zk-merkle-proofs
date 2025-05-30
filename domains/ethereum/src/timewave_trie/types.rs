use crate::timewave_trie::constants::*;
use arrayvec::ArrayVec;
use nybbles::Nibbles;

extern crate alloc;
use alloc::vec::Vec;

use crate::{
    merkle_lib::digest_keccak,
    timewave_rlp::{self, alloy_bytes::Bytes, Decodable},
};

#[derive(PartialEq, Eq)]
pub struct RlpNode(ArrayVec<u8, MAX>);

#[derive(PartialEq, Eq, Debug, Clone, Copy, Default)]
pub struct TrieMask(u16);

impl TrieMask {
    #[inline]
    pub const fn is_bit_set(self, index: u8) -> bool {
        self.0 & (1u16 << index) != 0
    }
    pub fn set_bit(&mut self, index: u8) {
        self.0 |= 1u16 << index;
    }
}

impl timewave_rlp::Decodable for RlpNode {
    fn decode(buf: &mut &[u8]) -> timewave_rlp::Result<Self> {
        let bytes = timewave_rlp::Header::decode_bytes(buf, false)?;
        Self::from_raw_rlp(bytes)
    }
}

impl core::ops::Deref for RlpNode {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for RlpNode {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AsRef<[u8]> for RlpNode {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl core::fmt::Debug for RlpNode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "RlpNode({})", hex::encode_prefixed(&self.0))
    }
}

impl RlpNode {
    /// Creates a new RLP-encoded node from the given data.
    ///
    /// Returns `None` if the data is too large (greater than 33 bytes).
    #[inline]
    pub fn from_raw(data: &[u8]) -> Option<Self> {
        let mut arr = ArrayVec::new();
        arr.try_extend_from_slice(data).ok()?;
        Some(Self(arr))
    }

    /// Creates a new RLP-encoded node from the given data.
    #[inline]
    pub fn from_raw_rlp(data: &[u8]) -> timewave_rlp::Result<Self> {
        Self::from_raw(data).ok_or(timewave_rlp::Error::Custom("RLP node too large"))
    }

    /// Given an RLP-encoded node, returns it either as `rlp(node)` or `rlp(keccak(rlp(node)))`.
    #[doc(alias = "rlp_node")]
    #[inline]
    pub fn from_rlp(rlp: &[u8]) -> Self {
        if rlp.len() < 32 {
            // SAFETY: `rlp` is less than max capacity (33).
            unsafe { Self::from_raw(rlp).unwrap_unchecked() }
        } else {
            Self::word_rlp(&digest_keccak(rlp))
        }
    }

    /// RLP-encodes the given word and returns it as a new RLP node.
    #[inline]
    pub fn word_rlp(word: &[u8; 32]) -> Self {
        let mut arr = [0u8; 33];
        arr[0] = EMPTY_STRING_CODE + 32;
        arr[1..].copy_from_slice(word.as_slice());
        Self(ArrayVec::from(arr))
    }

    /// Returns the RLP-encoded node as a slice.
    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    /// Returns hash if this is an RLP-encoded hash
    #[inline]
    pub fn as_hash(&self) -> Option<[u8; 32]> {
        if self.len() == 32 + 1 {
            Some(self.0[1..].try_into().unwrap())
        } else {
            None
        }
    }
}

/// Represents a node in the Ethereum state trie.
///
/// This enum defines the different types of nodes that can exist in an Ethereum
/// state trie. Each variant represents a specific node type with its associated
/// data structure.
#[derive(Debug)]
pub enum TrieNode {
    /// An empty root node, representing an empty trie
    EmptyRoot,
    /// A branch node that can have up to 16 children
    Branch(BranchNode),
    /// An extension node that shares a common prefix with its child
    Extension(ExtensionNode),
    /// A leaf node containing the final value
    Leaf(LeafNode),
}

/// A branch node in the trie that can have up to 16 children.
///
/// Branch nodes are used when multiple paths diverge at a particular point in
/// the trie. Each branch node can have up to 16 children, one for each possible
/// nibble value (0-15).
#[derive(Debug, Default)]
pub struct BranchNode {
    /// The collection of RLP encoded children.
    pub stack: Vec<RlpNode>,
    /// The bitmask indicating the presence of children at the respective nibble positions
    pub state_mask: TrieMask,
}

/// A reference to a branch node's data.
///
/// This struct provides a view into a branch node's data without taking ownership.
/// It's used for efficient traversal and verification of the trie structure.
pub struct BranchNodeRef<'a> {
    /// Reference to the collection of RLP encoded nodes.
    /// NOTE: The referenced stack might have more items than the number of children
    /// for this node. We should only ever access items starting from
    /// [BranchNodeRef::first_child_index].
    pub stack: &'a [RlpNode],
    /// Reference to bitmask indicating the presence of children at
    /// the respective nibble positions.
    pub state_mask: TrieMask,
}

impl<'a> BranchNodeRef<'a> {
    /// Create a new branch node from the stack of nodes.
    #[inline]
    pub const fn new(stack: &'a [RlpNode], state_mask: TrieMask) -> Self {
        Self { stack, state_mask }
    }

    pub fn first_child_index(&self) -> usize {
        self.stack
            .len()
            .checked_sub(self.state_mask.0.count_ones() as usize)
            .unwrap()
    }
}

impl BranchNode {
    pub fn as_ref(&self) -> BranchNodeRef<'_> {
        BranchNodeRef::new(&self.stack, self.state_mask)
    }
}

/// An extension node in the trie that shares a common prefix.
///
/// Extension nodes are used to optimize the trie by sharing common prefixes
/// between multiple paths. They contain a key (the shared prefix) and a pointer
/// to the next node.
#[derive(Debug)]
pub struct ExtensionNode {
    /// The key for this extension node.
    pub key: Nibbles,
    /// A pointer to the child node.
    pub child: RlpNode,
}

impl ExtensionNode {
    pub fn new(key: Nibbles, child: RlpNode) -> Self {
        Self { key, child }
    }
}

/// A leaf node in the trie containing the final value.
///
/// Leaf nodes represent the end of a path in the trie and contain the actual
/// value associated with the key.
#[derive(Debug)]
pub struct LeafNode {
    /// The key for this leaf node.
    pub key: Nibbles,
    /// The node value.
    pub value: Vec<u8>,
}

impl LeafNode {
    pub fn new(key: Nibbles, value: Vec<u8>) -> Self {
        Self { key, value }
    }
}

impl Decodable for TrieNode {
    fn decode(buf: &mut &[u8]) -> timewave_rlp::Result<Self> {
        let mut items = match timewave_rlp::Header::decode_raw(buf)? {
            timewave_rlp::PayloadView::List(list) => list,
            timewave_rlp::PayloadView::String(val) => {
                return if val.is_empty() {
                    Ok(Self::EmptyRoot)
                } else {
                    Err(timewave_rlp::Error::UnexpectedString)
                }
            }
        };

        // A valid number of trie node items is either 17 (branch node)
        // or 2 (extension or leaf node).
        match items.len() {
            17 => {
                let mut branch = BranchNode::default();
                for (idx, item) in items.into_iter().enumerate() {
                    if idx == 16 {
                        if item != [EMPTY_STRING_CODE] {
                            return Err(timewave_rlp::Error::Custom(
                                "branch node values are not supported",
                            ));
                        }
                    } else if item != [EMPTY_STRING_CODE] {
                        branch.stack.push(RlpNode::from_raw_rlp(item)?);
                        branch.state_mask.set_bit(idx as u8);
                    }
                }
                Ok(Self::Branch(branch))
            }
            2 => {
                let mut key = items.remove(0);

                let encoded_key = timewave_rlp::Header::decode_bytes(&mut key, false)?;
                if encoded_key.is_empty() {
                    return Err(timewave_rlp::Error::Custom("trie node key empty"));
                }

                // extract the high order part of the nibble to then pick the odd nibble out
                let key_flag = encoded_key[0] & 0xf0;
                // Retrieve first byte. If it's [Some], then the nibbles are odd.
                let first = match key_flag {
                    ODD_FLAG => Some(encoded_key[0] & 0x0f),
                    EVEN_FLAG => None,
                    _ => return Err(timewave_rlp::Error::Custom("node is not extension or leaf")),
                };

                let key = unpack_path_to_nibbles(first, &encoded_key[1..]);
                let node = if key_flag == EVEN_FLAG || key_flag == ODD_FLAG {
                    let value = Bytes::decode(&mut items.remove(0))?.into();
                    Self::Leaf(LeafNode::new(key, value))
                } else {
                    // We don't decode value because it is expected to be RLP encoded.
                    Self::Extension(ExtensionNode::new(
                        key,
                        RlpNode::from_raw_rlp(items.remove(0))?,
                    ))
                };
                Ok(node)
            }
            _ => Err(timewave_rlp::Error::Custom(
                "invalid number of items in the list",
            )),
        }
    }
}

pub(crate) fn unpack_path_to_nibbles(first: Option<u8>, rest: &[u8]) -> Nibbles {
    let Some(first) = first else {
        return Nibbles::unpack(rest);
    };
    debug_assert!(first <= 0xf);
    let len = rest.len() * 2 + 1;
    // SAFETY: `len` is calculated correctly.
    unsafe {
        Nibbles::from_repr_unchecked(nybbles::smallvec_with(len, |buf| {
            let (f, r) = buf.split_first_mut().unwrap_unchecked();
            f.write(first);
            Nibbles::unpack_to_unchecked(rest, r);
        }))
    }
}
