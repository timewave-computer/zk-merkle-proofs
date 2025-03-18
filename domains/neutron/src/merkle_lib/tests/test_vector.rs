#[cfg(feature = "no-sp1")]
use {
    crate::merkle_lib::types::{MerkleProverNeutron, NeutronKey, NeutronMerkleProof},
    common::merkle::types::MerkleProver,
    dotenvy::dotenv,
    std::env,
};

#[cfg(all(feature = "no-sp1", feature = "tests-online"))]
#[tokio::test]
async fn test_get_neutron_contract_value_from_dict() {
    use cosmrs::AccountId;
    use cosmwasm_std::Addr;
    use std::str::FromStr;
    let contract_address =
        AccountId::from_str("neutron1nyuryl5u5z04dx4zsqgvsuw7fe8gl2f77yufynauuhklnnmnjncqcls0tj")
            .unwrap();
    let height: u64 = 918;
    let initial_address = "neutron1m9l358xunhhwds0568za49mzhvuxx9ux8xafx2";
    let mut key_bytes = vec![0x03];
    key_bytes.append(&mut contract_address.to_bytes());
    let length_bytes = (b"store".len() as u32).to_be_bytes();
    let relevant_bytes = [length_bytes[2], length_bytes[3]];
    key_bytes.extend_from_slice(&relevant_bytes);
    key_bytes.extend_from_slice(b"store");
    key_bytes.append(&mut Addr::unchecked(initial_address).as_bytes().to_vec());
    let rpc_url = read_rpc_url();
    let prover = MerkleProverNeutron { rpc_url };
    let neutron_key = NeutronKey {
        prefix: "wasm".to_string(),
        prefix_len: 4,
        key: hex::encode(key_bytes),
    };
    let proofs = prover
        .get_merkle_proof_from_rpc(&neutron_key.serialize(), "", height)
        .await;
    //let x: NeutronMerkleProof = serde_json::from_slice(&proofs).unwrap();
}

#[cfg(all(feature = "no-sp1", feature = "tests-online"))]
// first verifies account state, then a single storage proof
// currently the variables need to be manually set before running the test
pub async fn get_neutron_test_vector_bank_store_supply() -> NeutronMerkleProof {
    let rpc_url = read_rpc_url();
    let supply_key = construct_supply_key(&read_test_vector_denom(), vec![0x00]);
    let prover = MerkleProverNeutron { rpc_url };
    let neutron_key = NeutronKey {
        prefix: "bank".to_string(),
        prefix_len: 4,
        key: hex::encode(supply_key),
    };
    let proofs = prover
        .get_merkle_proof_from_rpc(&neutron_key.serialize(), "", read_test_vector_height())
        .await;
    serde_json::from_slice(&proofs).unwrap()
}

#[cfg(feature = "no-sp1")]
pub fn read_rpc_url() -> String {
    dotenv().ok();
    env::var("NEUTRON_RPC").expect("Missing Neutron RPC url!")
}

#[cfg(feature = "no-sp1")]
pub fn read_test_vector_denom() -> String {
    return "untrn".to_string();
}

#[cfg(feature = "no-sp1")]
pub fn read_test_vector_height() -> u64 {
    dotenv().ok();
    env::var("TEST_VECTOR_HEIGHT_NEUTRON")
        .expect("Missing Neutron TEST VECTOR: HEIGHT!")
        .parse::<u64>()
        .expect("Failed to parse test vector as u64: Amount")
}

#[cfg(feature = "no-sp1")]
pub fn read_test_vector_merkle_root() -> String {
    dotenv().ok();
    env::var("TEST_VECTOR_MERKLE_ROOT_NEUTRON").expect("Missing Neutron TEST VECTOR: ROOT!")
}

#[cfg(feature = "no-sp1")]
pub fn construct_supply_key(denom: &str, prefix: Vec<u8>) -> Vec<u8> {
    let mut key = prefix; // Prefix for supply query in the Cosmos SDK
    key.extend_from_slice(denom.as_bytes()); // Append the denom in UTF-8 encoding
    key
}

pub const TEST_VECTOR_NEUTRON_STORAGE_PROOF: &[u8] = &[
    123, 34, 112, 114, 111, 111, 102, 34, 58, 123, 34, 111, 112, 115, 34, 58, 91, 123, 34, 102,
    105, 101, 108, 100, 95, 116, 121, 112, 101, 34, 58, 34, 105, 99, 115, 50, 51, 58, 105, 97, 118,
    108, 34, 44, 34, 107, 101, 121, 34, 58, 34, 65, 72, 86, 117, 100, 72, 74, 117, 34, 44, 34, 100,
    97, 116, 97, 34, 58, 34, 67, 111, 85, 67, 67, 103, 89, 65, 100, 87, 53, 48, 99, 109, 52, 83,
    68, 122, 99, 119, 77, 68, 65, 119, 77, 68, 65, 119, 77, 68, 65, 119, 77, 68, 65, 119, 77, 66,
    111, 76, 67, 65, 69, 89, 65, 83, 65, 66, 75, 103, 77, 65, 65, 103, 73, 105, 75, 119, 103, 66,
    69, 103, 81, 67, 66, 65, 73, 103, 71, 105, 69, 103, 112, 87, 82, 48, 81, 116, 57, 107, 87, 77,
    108, 43, 67, 70, 52, 74, 65, 111, 108, 117, 106, 89, 57, 82, 86, 120, 106, 97, 78, 109, 51, 83,
    105, 98, 85, 81, 112, 87, 81, 48, 99, 50, 73, 105, 75, 81, 103, 66, 69, 105, 85, 69, 67, 65,
    73, 103, 97, 90, 101, 73, 101, 111, 47, 75, 88, 90, 121, 75, 52, 51, 47, 43, 101, 74, 105, 100,
    73, 71, 90, 77, 117, 82, 112, 74, 55, 56, 115, 105, 79, 73, 76, 122, 106, 68, 121, 48, 48, 90,
    77, 103, 73, 105, 115, 73, 65, 82, 73, 69, 66, 104, 65, 67, 73, 66, 111, 104, 73, 70, 119, 74,
    120, 115, 77, 77, 67, 105, 76, 97, 47, 97, 100, 47, 48, 47, 57, 114, 56, 74, 52, 106, 82, 108,
    85, 65, 103, 73, 112, 101, 90, 104, 69, 102, 77, 100, 65, 114, 101, 57, 72, 88, 73, 105, 115,
    73, 65, 82, 73, 69, 67, 67, 65, 67, 73, 66, 111, 104, 73, 74, 66, 71, 88, 71, 107, 120, 69, 48,
    114, 75, 81, 68, 80, 77, 90, 120, 101, 99, 55, 78, 71, 69, 87, 49, 97, 81, 53, 75, 122, 56,
    103, 100, 72, 101, 83, 107, 66, 117, 109, 85, 66, 112, 73, 105, 115, 73, 65, 82, 73, 69, 67,
    107, 65, 67, 73, 66, 111, 104, 73, 76, 74, 99, 54, 79, 43, 65, 117, 66, 50, 80, 118, 107, 106,
    66, 83, 90, 113, 78, 112, 116, 108, 105, 121, 100, 112, 43, 53, 80, 99, 106, 100, 107, 43, 108,
    65, 55, 56, 77, 50, 105, 103, 115, 34, 125, 44, 123, 34, 102, 105, 101, 108, 100, 95, 116, 121,
    112, 101, 34, 58, 34, 105, 99, 115, 50, 51, 58, 115, 105, 109, 112, 108, 101, 34, 44, 34, 107,
    101, 121, 34, 58, 34, 89, 109, 70, 117, 97, 119, 61, 61, 34, 44, 34, 100, 97, 116, 97, 34, 58,
    34, 67, 113, 99, 67, 67, 103, 82, 105, 89, 87, 53, 114, 69, 105, 65, 119, 82, 112, 83, 71, 53,
    113, 86, 80, 111, 87, 86, 116, 106, 75, 114, 71, 57, 97, 117, 119, 109, 115, 113, 112, 74, 87,
    85, 122, 114, 114, 116, 112, 71, 51, 109, 78, 53, 66, 82, 120, 118, 104, 111, 74, 67, 65, 69,
    89, 65, 83, 65, 66, 75, 103, 69, 65, 73, 105, 99, 73, 65, 82, 73, 66, 65, 82, 111, 103, 51,
    109, 105, 115, 114, 116, 77, 111, 81, 72, 68, 115, 101, 50, 103, 98, 85, 56, 113, 104, 107, 56,
    74, 55, 111, 121, 79, 97, 69, 80, 116, 67, 55, 111, 100, 104, 114, 112, 47, 75, 99, 98, 77,
    105, 74, 119, 103, 66, 69, 103, 69, 66, 71, 105, 68, 98, 89, 55, 119, 120, 116, 104, 105, 68,
    73, 67, 48, 56, 101, 69, 111, 116, 67, 104, 97, 87, 122, 90, 54, 72, 89, 57, 69, 71, 87, 108,
    43, 65, 84, 51, 87, 52, 71, 86, 52, 118, 107, 121, 73, 108, 67, 65, 69, 83, 73, 81, 69, 119,
    121, 120, 49, 76, 117, 119, 70, 113, 101, 68, 51, 106, 121, 100, 55, 72, 112, 78, 52, 118, 49,
    118, 103, 74, 102, 83, 107, 104, 105, 108, 110, 116, 84, 70, 118, 82, 84, 53, 100, 97, 76, 67,
    73, 110, 67, 65, 69, 83, 65, 81, 69, 97, 73, 74, 89, 54, 77, 77, 101, 111, 56, 78, 56, 75, 119,
    103, 114, 107, 65, 65, 86, 72, 114, 49, 101, 74, 109, 50, 117, 74, 70, 68, 48, 50, 69, 86, 120,
    117, 43, 54, 115, 102, 80, 55, 88, 72, 73, 105, 99, 73, 65, 82, 73, 66, 65, 82, 111, 103, 118,
    100, 90, 104, 99, 113, 50, 102, 103, 52, 54, 56, 43, 57, 100, 47, 70, 80, 87, 78, 108, 105, 67,
    52, 67, 73, 111, 67, 81, 77, 112, 114, 76, 47, 105, 51, 78, 70, 56, 70, 72, 118, 52, 105, 74,
    119, 103, 66, 69, 103, 69, 66, 71, 105, 67, 71, 67, 109, 99, 111, 113, 88, 97, 114, 89, 70, 79,
    56, 82, 67, 118, 81, 78, 54, 103, 107, 111, 109, 99, 102, 120, 69, 71, 116, 102, 85, 74, 119,
    66, 69, 110, 80, 70, 82, 103, 80, 47, 103, 61, 61, 34, 125, 93, 125, 44, 34, 107, 101, 121, 34,
    58, 123, 34, 112, 114, 101, 102, 105, 120, 34, 58, 34, 98, 97, 110, 107, 34, 44, 34, 112, 114,
    101, 102, 105, 120, 95, 108, 101, 110, 34, 58, 52, 44, 34, 107, 101, 121, 34, 58, 34, 48, 48,
    55, 53, 54, 101, 55, 52, 55, 50, 54, 101, 34, 125, 44, 34, 118, 97, 108, 117, 101, 34, 58, 91,
    53, 53, 44, 52, 56, 44, 52, 56, 44, 52, 56, 44, 52, 56, 44, 52, 56, 44, 52, 56, 44, 52, 56, 44,
    52, 56, 44, 52, 56, 44, 52, 56, 44, 52, 56, 44, 52, 56, 44, 52, 56, 44, 52, 56, 93, 125,
];

pub const TEST_VECTOR_NEUTRON_ROOT: &str = "xuPL4Vt/UqXOvYfaVNsE5rqtOqB3j1UIi2GLB7SvPNY=";
pub const TEST_VECTOR_NEUTRON_HEIGHT: &str = "19";
