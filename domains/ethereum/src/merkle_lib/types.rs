#[cfg(feature = "web")]
use alloy::{
    consensus::{Receipt, ReceiptEnvelope, ReceiptWithBloom, TxReceipt},
    rpc::types::{Log as AlloyLog, TransactionReceipt},
};
use alloy_primitives::{FixedBytes, B256};
#[cfg(feature = "web")]
use alloy_rlp::{BufMut, Encodable, RlpEncodableWrapper};
use common::{merkle::types::MerkleProofOutput, merkle::types::MerkleVerifiable};
use eth_trie::{EthTrie, MemoryDB, Trie, DB};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EthereumProof {
    pub proof: Vec<Vec<u8>>,
    pub key: Vec<u8>,
    pub root: Vec<u8>,
}

#[cfg(feature = "web")]
use crate::encode;
#[cfg(feature = "web")]
use {
    alloy::hex::FromHex,
    alloy::providers::{Provider, ProviderBuilder},
    alloy::rpc::types::EIP1186AccountProofResponse,
    alloy_primitives::Address,
    common::merkle::types::MerkleProver,
    std::str::FromStr,
    url::Url,
};

#[cfg(feature = "web")]
pub struct EvmProver {
    pub rpc_url: String,
}
#[cfg(feature = "web")]
impl MerkleProver for EvmProver {
    /// returns an account proof object for the requested address
    /// that contains a list of storage proofs for the requested keys
    /// we can verify the combined proof or extract the account proof
    /// and individual storage proofs
    #[allow(unused)]
    async fn get_storage_proof(&self, keys: Vec<&str>, address: &str, height: u64) -> Vec<u8> {
        let address_object = Address::from_hex(address).unwrap();
        let provider = ProviderBuilder::new().on_http(Url::from_str(&self.rpc_url).unwrap());
        let proof: EIP1186AccountProofResponse = provider
            .get_proof(
                address_object,
                keys.iter()
                    .map(|k| FixedBytes::from_hex(k).unwrap())
                    .collect(),
            )
            // use this in production!
            //.block_id(height.try_into().unwrap())
            .await
            .expect("Failed to get storage proof!");
        serde_json::to_vec(&proof).expect("Failed to serialize proof!")
    }
}

#[cfg(feature = "web")]
impl EvmProver {
    pub async fn get_receipt_proof(&self, block_hash: &str, target_index: u32) -> EthereumProof {
        let provider = ProviderBuilder::new().on_http(Url::from_str(&self.rpc_url).unwrap());
        let block_hash_b256 = B256::from_str(block_hash).unwrap();
        let block = provider
            .get_block_by_hash(
                B256::from_str(block_hash).unwrap(),
                alloy::rpc::types::BlockTransactionsKind::Full,
            )
            .await
            .expect("Failed to get Block!")
            .expect("Block not found!");
        let receipts: Vec<TransactionReceipt> = provider
            .get_block_receipts(alloy::eips::BlockId::Hash(block_hash_b256.into()))
            .await
            .unwrap()
            .unwrap();
        let memdb = Arc::new(MemoryDB::new(true));
        let mut trie = EthTrie::new(memdb.clone());

        for (index, receipt) in receipts.into_iter().enumerate() {
            let inner: ReceiptEnvelope<alloy::rpc::types::Log> = receipt.inner;
            let mut out: Vec<u8> = Vec::new();
            let index_encoded = alloy_rlp::encode(index);
            match inner {
                ReceiptEnvelope::Eip2930(r) => {
                    let prefix: u8 = 0x01;
                    insert_receipt(r, &mut trie, index_encoded, Some(prefix));
                }
                ReceiptEnvelope::Eip1559(r) => {
                    let prefix: u8 = 0x02;
                    insert_receipt(r, &mut trie, index_encoded, Some(prefix));
                }
                ReceiptEnvelope::Eip4844(r) => {
                    let prefix: u8 = 0x03;
                    out.put_u8(0x03);
                    insert_receipt(r, &mut trie, index_encoded, Some(prefix));
                }
                ReceiptEnvelope::Eip7702(r) => {
                    let prefix: u8 = 0x04;
                    out.put_u8(0x04);
                    insert_receipt(r, &mut trie, index_encoded, Some(prefix));
                }
                ReceiptEnvelope::Legacy(r) => {
                    insert_receipt(r, &mut trie, index_encoded, None);
                }
                _ => {
                    eprintln!("Unknown Receipt Type!")
                }
            }
        }
        trie.root_hash().unwrap();
        let receipt_key: Vec<u8> = alloy_rlp::encode(target_index);
        let proof = trie.get_proof(&receipt_key).unwrap();
        EthereumProof {
            proof,
            root: block.header.receipts_root.to_vec(),
            key: receipt_key,
        }
    }
}

#[cfg(feature = "web")]
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

#[cfg(feature = "web")]
pub struct Log {
    pub address: Address,
    pub topics: Vec<H256>,
    pub data: Vec<u8>,
}

#[cfg(feature = "web")]
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

#[cfg(feature = "web")]
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

#[cfg(feature = "web")]
#[derive(Debug, RlpEncodableWrapper, PartialEq, Clone)]
pub struct H256(pub [u8; 32]);

#[cfg(feature = "web")]
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

impl MerkleVerifiable for EthereumProof {
    fn verify(&self, expected_root: &[u8]) -> MerkleProofOutput {
        let root_hash = FixedBytes::from_slice(expected_root);
        let proof_db = Arc::new(MemoryDB::new(true));
        for node_encoded in &self.proof.clone() {
            let hash: B256 = crate::merkle_lib::keccak::digest_keccak(node_encoded).into();
            proof_db
                .insert(hash.as_slice(), node_encoded.to_vec())
                .unwrap();
        }
        let mut trie = EthTrie::from(proof_db, root_hash).expect("Invalid merkle proof");
        assert_eq!(root_hash, trie.root_hash().unwrap());
        trie.verify_proof(root_hash, &self.key, self.proof.clone())
            .expect("Failed to verify Merkle Proof")
            .expect("Key does not exist!");

        MerkleProofOutput {
            root: expected_root.to_vec(),
            key: self.key.clone(),
            // for Ethereum the value is the last node (a leaf) in the proof
            value: self.proof.last().unwrap().to_vec(),
            domain: common::merkle::types::Domain::ETHEREUM,
        }
    }
}

// block hash example: 0xc3e7838359382f8ecc52ec0d8951c4c76a55524494eff38b93f317221ef27f73
// get the balance of a user in a contract from a key on eth
// get the balance of a user from the bank store  on neutron
// do the calculation and construct the messages for either side accordingly
