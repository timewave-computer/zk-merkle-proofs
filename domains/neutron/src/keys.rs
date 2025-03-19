use std::str::FromStr;

use serde::{Deserialize, Serialize};
#[cfg(feature = "no-sp1")]
use {cosmrs::AccountId, cosmwasm_std::Addr};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct NeutronKey {
    pub prefix: String,
    pub prefix_len: usize,
    pub key: String,
}

impl NeutronKey {
    /// Serializes the NeutronKey by encoding prefix_len explicitly.
    /// Maximum prefix length is 3 digits e.g. 999
    pub fn serialize(&self) -> String {
        format!("{:03}{}{}", self.prefix_len, self.prefix, self.key)
    }
    /// Deserializes a string back into a NeutronKey.
    pub fn deserialize(encoded: &str) -> Self {
        let prefix_len: usize = encoded[..3].parse().expect("Invalid prefix length");
        let prefix = &encoded[3..(3 + prefix_len)];
        let key = &encoded[(3 + prefix_len)..];

        NeutronKey {
            prefix: prefix.to_string(),
            prefix_len,
            key: key.to_string(),
        }
    }
    // create a new neutron key for a mapping from address:value that lives under some contract
    // this is useful for examples where users are assigned balances
    #[cfg(feature = "no-sp1")]
    pub fn new_wasm_account_mapping(store: &[u8], key: &str, contract_address: &str) -> Self {
        let mut key_bytes = vec![0x03];
        key_bytes.append(
            &mut AccountId::from_str(contract_address)
                .expect("Invalid contract address")
                .to_bytes(),
        );
        let length_bytes = (store.len() as u32).to_be_bytes();
        let relevant_bytes = [length_bytes[2], length_bytes[3]];
        key_bytes.extend_from_slice(&relevant_bytes);
        key_bytes.extend_from_slice(store);
        key_bytes.append(&mut Addr::unchecked(key).as_bytes().to_vec());
        Self {
            prefix: "wasm".to_string(),
            prefix_len: 4,
            key: hex::encode(&key_bytes),
        }
    }

    #[cfg(feature = "no-sp1")]
    pub fn new_wasm_stored_value(store: &str, contract_address: &str) -> Self {
        let mut key_bytes = vec![0x03];
        key_bytes.append(
            &mut AccountId::from_str(contract_address)
                .expect("Invalid contract address")
                .to_bytes(),
        );
        key_bytes.extend_from_slice(store.as_bytes());
        Self {
            prefix: "wasm".to_string(),
            prefix_len: 4,
            key: hex::encode(&key_bytes),
        }
    }

    #[cfg(feature = "no-sp1")]
    pub fn new_bank_total_supply(denom: &str) -> Self {
        // supply prefix is 0x00
        // see https://protective-bearberry-a26.notion.site/Query-the-state-of-a-Cosmos-chain-and-verify-the-proof-1a55cfa0622c8055816ae6e6aec7f319?pvs=4
        let mut key_bytes = vec![0x00];
        key_bytes.extend_from_slice(denom.as_bytes());
        Self {
            prefix: "bank".to_string(),
            prefix_len: 4,
            key: hex::encode(key_bytes),
        }
    }

    #[cfg(feature = "no-sp1")]
    pub fn new_bank_account_balance(denom: &str, address: &str) -> Self {
        // balance prefix is 0x02
        // see https://protective-bearberry-a26.notion.site/Query-the-state-of-a-Cosmos-chain-and-verify-the-proof-1a55cfa0622c8055816ae6e6aec7f319?pvs=4
        let mut key_bytes = vec![0x02];
        let account_id = AccountId::from_str(address).expect("Invalid account address");
        let address_bytes = account_id.to_bytes();
        key_bytes.push(address_bytes.len() as u8);
        key_bytes.extend_from_slice(&address_bytes);
        key_bytes.extend_from_slice(denom.as_bytes());
        Self {
            prefix: "bank".to_string(),
            prefix_len: 4,
            key: hex::encode(key_bytes),
        }
    }
}
