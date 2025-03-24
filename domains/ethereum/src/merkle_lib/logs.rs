//! Ethereum transaction receipt logs handling module.
//!
//! This module provides functionality for working with Ethereum transaction receipt logs,
//! including encoding, decoding, and handling of ERC20 transfer events.

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
    //use alloy::hex;
    //use alloy_primitives::U256;
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
/// # Fields
/// * `address` - The address of the contract that emitted the log
/// * `topics` - Array of indexed parameters from the event
/// * `data` - The non-indexed parameters of the event
#[derive(Debug, Clone)]
pub struct Log {
    pub address: Address,
    pub topics: Vec<H256>,
    pub data: Vec<u8>,
}

impl Log {
    /// Calculates the RLP header for this log entry.
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
    /// # Arguments
    /// * `out` - The buffer to write the encoded data to
    fn encode(&self, out: &mut dyn alloy_rlp::BufMut) {
        let header = self.rlp_header();
        encode!(out, header, self.address, self.topics, self.data.as_slice());
    }

    /// Returns the length of the RLP encoded log entry.
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
/// such as event topics and transaction hashes.
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
    /// # Note
    /// If the slice is shorter than 32 bytes, the remaining bytes will be zero
    pub fn from_slice(slice: &[u8]) -> Self {
        let mut bytes = [0u8; 32];
        bytes[..slice.len()].copy_from_slice(slice);
        Self(bytes)
    }
}
