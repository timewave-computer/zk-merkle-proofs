//! RLP (Recursive Length Prefix) encoding utilities for Ethereum data structures.
//!
//! This module provides functions for encoding Ethereum-specific data structures
//! using the RLP encoding scheme. RLP is the main encoding method used to serialize
//! objects in Ethereum's execution layer.
use alloy::{
    consensus::{Receipt, ReceiptWithBloom, TxReceipt, TxType},
    rpc::types::TransactionReceipt,
};
use anyhow::{Context, Result};

/// Adjusts an index for RLP encoding based on its value and the total length.
///
/// This function is used to handle special cases in RLP encoding where certain
/// index values need to be adjusted to maintain canonical encoding.
///
/// # Arguments
/// * `i` - The index to adjust
/// * `len` - The total length of the sequence
///
/// # Returns
/// The adjusted index value according to RLP encoding rules
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
/// This function takes a transaction receipt and encodes it according to the
/// Ethereum RLP specification, handling both legacy and typed transactions.
///
/// # Arguments
/// * `receipt` - The transaction receipt to encode
///
/// # Returns
/// The RLP-encoded receipt as a vector of bytes
///
/// # Errors
/// Returns an error if the receipt cannot be properly encoded or if the inner
/// receipt data is invalid
pub fn encode_receipt(receipt: &TransactionReceipt) -> Result<Vec<u8>> {
    let tx_type = receipt.transaction_type();
    let receipt = receipt
        .inner
        .as_receipt_with_bloom()
        .context("Failed to extract inner receipts with bloom")?;
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
        TxType::Legacy => Ok(encoded),
        _ => Ok([vec![tx_type as u8], encoded].concat()),
    }
}
