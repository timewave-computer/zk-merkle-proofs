//! Ethereum transaction receipt logs handling module.
//!
//! This module provides functionality for working with Ethereum transaction receipt logs,
//! including encoding, decoding, and handling of ERC20 transfer events. It implements
//! the RLP encoding for log entries and provides utilities for working with log topics
//! and data.

use {
    crate::encode,
    alloy::{
        consensus::{Receipt, ReceiptWithBloom, TxReceipt},
        rpc::types::Log as AlloyLog,
    },
    alloy_primitives::Address,
    alloy_rlp::RlpEncodableWrapper,
    alloy_rlp::{BufMut, Encodable},
    eth_trie::{EthTrie, MemoryDB, Trie},
};

/// Inserts a transaction receipt into a Merkle Patricia Trie.
///
/// This function takes a receipt and inserts it into the trie at the specified index.
/// The receipt is RLP encoded and optionally prefixed before insertion.
///
/// # Arguments
/// * `r` - The receipt to insert
/// * `trie` - The trie to insert into
/// * `index_encoded` - The encoded index for the receipt
/// * `prefix` - Optional prefix byte for the receipt
///
/// # Panics
/// Panics if the insertion into the trie fails
pub fn insert_receipt(
    r: ReceiptWithBloom<Receipt<AlloyLog>>,
    trie: &mut EthTrie<MemoryDB>,
    index_encoded: &[u8],
    prefix: Option<u8>,
) {
    let status = r.status();
    let cumulative_gas_used = r.cumulative_gas_used();
    let bloom = r.logs_bloom;
    let mut logs: Vec<Log> = Vec::new();
    for l in r.logs() {
        let mut topics: Vec<H256> = Vec::new();
        for t in l.topics() {
            topics.push(H256::from_slice(t.as_ref()));
        }
        logs.push(Log {
            address: l.address(),
            topics,
            data: l.data().data.to_vec(),
        });
    }
    let list_encode: [&dyn Encodable; 4] = [&status, &cumulative_gas_used, &bloom, &logs];
    let mut payload: Vec<u8> = Vec::new();
    alloy_rlp::encode_list::<_, dyn Encodable>(&list_encode, &mut payload);
    let mut out: Vec<u8> = Vec::new();
    if let Some(prefix) = prefix {
        out.put_u8(prefix);
    };
    out.put_slice(&payload);
    trie.insert(index_encoded, &out).expect("Failed to insert");
}

/// Represents an Ethereum log entry containing event data from a transaction.
///
/// This struct represents a log entry emitted by a smart contract during transaction
/// execution. It contains the contract address that emitted the log, the indexed
/// parameters (topics), and the non-indexed parameters (data).
///
/// # Fields
/// * `address` - The address of the contract that emitted the log
/// * `topics` - Array of indexed parameters from the event
/// * `data` - The non-indexed parameters of the event
#[derive(Debug, Clone)]
pub struct Log {
    /// The address of the contract that emitted the log
    pub address: Address,
    /// Array of indexed parameters from the event
    pub topics: Vec<H256>,
    /// The non-indexed parameters of the event
    pub data: Vec<u8>,
}

impl Log {
    /// Calculates the RLP header for this log entry.
    ///
    /// The header contains the list flag and the total length of the encoded payload.
    ///
    /// # Returns
    /// The RLP header containing the list flag and total payload length
    fn rlp_header(&self) -> alloy_rlp::Header {
        let payload_length =
            self.address.length() + self.topics.length() + self.data.as_slice().length();
        alloy_rlp::Header {
            list: true,
            payload_length,
        }
    }
}

impl Encodable for Log {
    /// Encodes the log entry using RLP encoding.
    ///
    /// The encoding follows the Ethereum RLP specification for log entries:
    /// [address, topics[], data]
    ///
    /// # Arguments
    /// * `out` - The buffer to write the encoded data to
    fn encode(&self, out: &mut dyn alloy_rlp::BufMut) {
        let header = self.rlp_header();
        encode!(out, header, self.address, self.topics, self.data.as_slice());
    }

    /// Returns the length of the RLP encoded log entry.
    ///
    /// This includes the length of the header and the encoded payload.
    ///
    /// # Returns
    /// The total length of the encoded data
    fn length(&self) -> usize {
        let rlp_head = self.rlp_header();
        alloy_rlp::length_of_length(rlp_head.payload_length) + rlp_head.payload_length
    }
}

/// A 32-byte hash type used for Ethereum topics and other hash values.
///
/// This type is used to represent 32-byte hashes in the Ethereum protocol,
/// such as event topics and transaction hashes. It provides methods for
/// creating and manipulating these hash values.
///
/// # Fields
/// * `0` - The 32-byte array containing the hash value
#[derive(Debug, RlpEncodableWrapper, PartialEq, Clone)]
pub struct H256(pub [u8; 32]);

impl H256 {
    /// Creates a new H256 with all bytes set to zero.
    ///
    /// # Returns
    /// A new H256 instance with all bytes set to zero
    pub fn zero() -> Self {
        Self([0u8; 32])
    }

    /// Creates a new H256 from a byte slice.
    ///
    /// # Arguments
    /// * `slice` - The byte slice to create the H256 from
    ///
    /// # Returns
    /// A new H256 instance containing the bytes from the slice
    ///
    /// # Panics
    /// Panics if the slice is longer than 32 bytes
    pub fn from_slice(slice: &[u8]) -> Self {
        if !validate_slice(slice) {
            panic!("Topic out of bounds");
        }
        let mut bytes = [0u8; 32];
        bytes[..slice.len()].copy_from_slice(slice);
        Self(bytes)
    }
}

/// Validates that a byte slice is suitable for creating an H256.
///
/// # Arguments
/// * `slice` - The byte slice to validate
///
/// # Returns
/// A boolean indicating whether the slice is valid
fn validate_slice(slice: &[u8]) -> bool {
    slice.len() <= 32
}
