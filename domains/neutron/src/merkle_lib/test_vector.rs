#[cfg(feature = "web")]
use {
    crate::merkle_lib::types::{NeutronProof, NeutronProver},
    common::merkle::types::MerkleProver,
    dotenvy::dotenv,
    std::env,
};

#[cfg(feature = "web")]
// first verifies account state, then a single storage proof
// currently the variables need to be manually set before running the test
pub async fn get_neutron_test_vector_bank_store_supply() -> NeutronProof {
    let rpc_url = read_rpc_url();
    let supply_key = construct_supply_key(&read_test_vector_denom(), vec![0x00]);
    let prover = NeutronProver { rpc_url };
    let proofs = prover
        .get_storage_proof(
            vec!["bank", &hex::encode(supply_key)],
            "",
            read_test_vector_height(),
        )
        .await;
    serde_json::from_slice(&proofs).unwrap()
}

#[cfg(feature = "web")]
pub fn read_rpc_url() -> String {
    dotenv().ok();
    env::var("NEUTRON_RPC").expect("Missing Neutron RPC url!")
}

#[cfg(feature = "web")]
pub fn read_test_vector_denom() -> String {
    dotenv().ok();
    env::var("TEST_VECTOR_DENOM_NEUTRON").expect("Missing Neutron TEST VECTOR: DENOM!")
}

#[cfg(feature = "web")]
pub fn read_test_vector_height() -> u64 {
    dotenv().ok();
    env::var("TEST_VECTOR_HEIGHT_NEUTRON")
        .expect("Missing Neutron TEST VECTOR: HEIGHT!")
        .parse::<u64>()
        .expect("Failed to parse test vector as u64: Amount")
}

#[cfg(feature = "web")]
pub fn read_test_vector_merkle_root() -> String {
    dotenv().ok();
    env::var("TEST_VECTOR_MERKLE_ROOT_NEUTRON").expect("Missing Neutron TEST VECTOR: ROOT!")
}

#[cfg(feature = "web")]
pub fn construct_supply_key(denom: &str, prefix: Vec<u8>) -> Vec<u8> {
    let mut key = prefix; // Prefix for supply query in the Cosmos SDK
    key.extend_from_slice(denom.as_bytes()); // Append the denom in UTF-8 encoding
    key
}
