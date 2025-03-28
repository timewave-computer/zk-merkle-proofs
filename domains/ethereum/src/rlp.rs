use alloy::{
    consensus::{Receipt, ReceiptWithBloom, TxReceipt, TxType},
    rpc::types::TransactionReceipt,
};

pub const fn adjust_index_for_rlp(i: usize, len: usize) -> usize {
    if i > 0x7f {
        i
    } else if i == 0x7f || i + 1 == len {
        0
    } else {
        i + 1
    }
}

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
