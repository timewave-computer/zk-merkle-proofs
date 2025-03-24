#[cfg(test)]
mod tests {}

#[cfg(feature = "no-sp1")]
use {dotenvy::dotenv, std::env};

#[cfg(feature = "no-sp1")]
pub fn read_rpc_url() -> String {
    dotenv().ok();
    env::var("NEUTRON_RPC").expect("Missing Neutron RPC url!")
}

#[cfg(feature = "no-sp1")]
pub fn read_test_vector_height() -> u64 {
    dotenv().ok();
    env::var("HEIGHT_NEUTRON")
        .expect("Missing Neutron TEST VECTOR: HEIGHT!")
        .parse::<u64>()
        .expect("Failed to parse test vector as u64: Amount")
}

#[cfg(feature = "no-sp1")]
pub fn read_test_vector_merkle_root() -> String {
    dotenv().ok();
    env::var("MERKLE_ROOT_NEUTRON").expect("Missing Neutron TEST VECTOR: ROOT!")
}

#[cfg(feature = "no-sp1")]
pub fn read_pion_1_vault_contract_address() -> String {
    dotenv().ok();
    env::var("NEUTRON_PION_1_VAULT_EXAMPLE_CONTRACT_ADDRESS")
        .expect("Missing Pion 1 Vault Contract Address!")
}

#[cfg(feature = "no-sp1")]
pub fn read_pion_1_default_account_address() -> String {
    dotenv().ok();
    env::var("NEUTRON_DEFAULT_ACCOUNT_ADDRESS").expect("Missing Neutron Default Account Address!")
}

use std::fs;
use std::path::PathBuf;

fn read_bytes_from_file(path: &str) -> std::io::Result<Vec<u8>> {
    fs::read(path)
}

pub fn get_test_vector_neutron_storage_proof() -> Vec<u8> {
    let path: PathBuf = [
        env!("CARGO_MANIFEST_DIR"),
        "src/merkle_lib/tests/data/storage_proof.bin",
    ]
    .iter()
    .collect();
    read_bytes_from_file(path.to_str().unwrap()).unwrap()
}

pub const TEST_VECTOR_NEUTRON_ROOT: &str = "xuPL4Vt/UqXOvYfaVNsE5rqtOqB3j1UIi2GLB7SvPNY=";
