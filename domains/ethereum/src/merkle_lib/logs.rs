#[cfg(feature = "no-sp1")]
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

#[cfg(feature = "no-sp1")]
pub fn insert_receipt(
    r: ReceiptWithBloom<Receipt<AlloyLog>>,
    trie: &mut EthTrie<MemoryDB>,
    index_encoded: Vec<u8>,
    prefix: Option<u8>,
) {
    //use alloy::hex;
    //use alloy_primitives::U256;
    let status = r.status();
    let cumulative_gas_used = r.cumulative_gas_used();
    let bloom = r.logs_bloom;
    let mut logs: Vec<Log> = Vec::new();
    /*let transfer_event_signature =
    hex::decode("ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef").unwrap();*/
    for l in r.logs() {
        let mut topics: Vec<H256> = Vec::new();
        for t in l.topics() {
            /*if t.to_vec() == transfer_event_signature {
                println!("Found an ERC20 Transfer!");
                let from = Address::from_slice(&l.topics()[1].0[12..]);
                let to = Address::from_slice(&l.topics()[2].0[12..]);
                let amount = U256::from_be_bytes::<32>(
                    l.data().data.to_vec().as_slice().try_into().unwrap(),
                );
                println!("From: {:?}, To: {:?}, Amount: {:?}", from, to, amount);
            }*/
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
