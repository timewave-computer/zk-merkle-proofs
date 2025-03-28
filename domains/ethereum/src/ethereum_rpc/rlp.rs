//! RLP (Recursive Length Prefix) encoding utilities for Ethereum data structures.
//!
//! This module provides functions for encoding Ethereum-specific data structures
//! using the RLP encoding scheme.

use alloy::{
    consensus::{Receipt, ReceiptWithBloom, TxReceipt, TxType},
    rpc::types::TransactionReceipt,
};

/// Adjusts an index for RLP encoding based on its value and the total length.
///
/// # Arguments
/// * `i` - The index to adjust
/// * `len` - The total length of the sequence
///
/// # Returns
/// The adjusted index value
pub const fn adjust_index_for_rlp(i: usize, len: usize) -> usize {
    if i > 0x7f {
        i
    } else if i == 0x7f || i + 1 == len {
        0
    } else {
        i + 1
    }
}

/// Encodes a transaction receipt into RLP format.
///
/// # Arguments
/// * `receipt` - The transaction receipt to encode
///
/// # Returns
/// The RLP-encoded receipt as a vector of bytes
pub fn encode_receipt(receipt: &TransactionReceipt) -> Vec<u8> {
    let tx_type = receipt.transaction_type();
    let receipt = receipt.inner.as_receipt_with_bloom().unwrap();
    let logs = receipt
        .logs()
        .iter()
        .map(|l| l.inner.clone())
        .collect::<Vec<_>>();

    let consensus_receipt = Receipt {
        cumulative_gas_used: receipt.cumulative_gas_used(),
        status: receipt.status_or_post_state(),
        logs,
    };

    let rwb = ReceiptWithBloom::new(consensus_receipt, receipt.bloom());
    let encoded = alloy::rlp::encode(rwb);

    match tx_type {
        TxType::Legacy => encoded,
        _ => [vec![tx_type as u8], encoded].concat(),
    }
}
