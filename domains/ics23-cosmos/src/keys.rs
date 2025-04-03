//! Key management for Ics23 blockchain state queries.
//!
//! This module provides functionality for creating and managing keys used to query
//! different types of state on the Ics23 blockchain, including bank balances,
//! WASM contract state, and other storage types.

use core::fmt;
use std::fmt::Display;

use serde::{Deserialize, Serialize};
#[cfg(feature = "no-zkvm")]
use {cosmrs::AccountId, cosmwasm_std::Addr, std::str::FromStr};

/// Represents a key used to query state on the Ics23 blockchain.
///
/// The key consists of a prefix (e.g., "bank", "wasm") and a key string that identifies
/// the specific state to query. The prefix_len field is used for serialization purposes.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Ics23Key {
    /// The prefix indicating the type of state (e.g., "bank", "wasm")
    pub prefix: String,
    /// The length of the prefix string
    pub prefix_len: usize,
    /// The specific key identifying the state to query
    pub key: String,
}

impl Display for Ics23Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:03}{}{}", self.prefix_len, self.prefix, self.key)
    }
}

impl Ics23Key {
    /// Deserializes a string back into a Ics23Key.
    pub fn from_string(encoded: &str) -> Self {
        let prefix_len: usize = encoded[..3].parse().expect("Invalid prefix length");
        let prefix = &encoded[3..(3 + prefix_len)];
        let key = &encoded[(3 + prefix_len)..];

        Ics23Key {
            prefix: prefix.to_string(),
            prefix_len,
            key: key.to_string(),
        }
    }
    // create a new neutron key for a mapping from address:value that lives under some contract
    // this is useful for examples where users are assigned balances
    // store: name of the storage module (bank, wasm, etc)
    // key: the key of the mapping
    // contract_address: the address of the contract under which the mapping lives
    #[cfg(feature = "no-zkvm")]
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

    // create a new neutron key for a stored value under a WASM contract
    // this is useful for accessing simple key-value storage in a contract
    // key: the key of the mapping e.g. "shares"
    // contract_address: the address of the contract where the value is stored
    #[cfg(feature = "no-zkvm")]
    pub fn new_wasm_stored_value(key: &str, contract_address: &str) -> Self {
        let mut key_bytes = vec![0x03];
        key_bytes.append(
            &mut AccountId::from_str(contract_address)
                .expect("Invalid contract address")
                .to_bytes(),
        );
        key_bytes.extend_from_slice(key.as_bytes());
        Self {
            prefix: "wasm".to_string(),
            prefix_len: 4,
            key: hex::encode(&key_bytes),
        }
    }

    // create a new neutron key for the total supply of a denom
    // this is useful for accessing the total supply of a denom in the bank module
    // denom: the denom of the supply to query
    #[cfg(feature = "no-zkvm")]
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

    // create a new neutron key for the balance of an account
    // this is useful for accessing the balance of an account in the bank module
    // denom: the denom of the balance to query
    // address: the address of the account to query
    #[cfg(feature = "no-zkvm")]
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
