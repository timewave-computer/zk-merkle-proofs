#[cfg(feature = "no-sp1")]
use {
    crate::encode,
    alloy::{
        consensus::{Receipt, ReceiptEnvelope, ReceiptWithBloom, TxReceipt},
        rpc::types::{Log as AlloyLog, TransactionReceipt},
        serde::JsonStorageKey,
    },
    alloy_primitives::Address,
    alloy_rlp::RlpEncodableWrapper,
    alloy_rlp::{BufMut, Encodable},
    eth_trie::{EthTrie, MemoryDB, Trie},
};

#[cfg(feature = "no-sp1")]
pub fn insert_receipt(
    r: ReceiptWithBloom<Receipt<AlloyLog>>,
    trie: &mut EthTrie<MemoryDB>,
    index_encoded: Vec<u8>,
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
    trie.insert(&index_encoded, &out).expect("Failed to insert");
}

#[cfg(feature = "no-sp1")]
#[derive(Debug, Clone)]
pub struct Log {
    pub address: Address,
    pub topics: Vec<H256>,
    pub data: Vec<u8>,
}

#[cfg(feature = "no-sp1")]
impl Log {
    fn rlp_header(&self) -> alloy_rlp::Header {
        let payload_length =
            self.address.length() + self.topics.length() + self.data.as_slice().length();
        alloy_rlp::Header {
            list: true,
            payload_length,
        }
    }
}

#[cfg(feature = "no-sp1")]
impl Encodable for Log {
    fn encode(&self, out: &mut dyn alloy_rlp::BufMut) {
        let header = self.rlp_header();
        encode!(out, header, self.address, self.topics, self.data.as_slice());
    }
    fn length(&self) -> usize {
        let rlp_head = self.rlp_header();
        alloy_rlp::length_of_length(rlp_head.payload_length) + rlp_head.payload_length
    }
}

#[cfg(feature = "no-sp1")]
#[derive(Debug, RlpEncodableWrapper, PartialEq, Clone)]
pub struct H256(pub [u8; 32]);

#[cfg(feature = "no-sp1")]
impl H256 {
    pub fn zero() -> Self {
        Self([0u8; 32])
    }
    pub fn from_slice(slice: &[u8]) -> Self {
        let mut bytes = [0u8; 32];
        bytes[..slice.len()].copy_from_slice(slice);
        Self(bytes)
    }
}
