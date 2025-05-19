#[cfg(test)]
pub(crate) mod constants {
    use crate::rpc::Ics23MerkleRpcClient;

    use {dotenvy::dotenv, std::env};

    pub(crate) fn read_rpc_url() -> String {
        dotenv().ok();
        env::var("NEUTRON_RPC").expect("Missing Neutron RPC url!")
    }

    pub(crate) async fn get_latest_root_and_height() -> (Vec<u8>, u64) {
        let client = Ics23MerkleRpcClient {
            rpc_url: read_rpc_url(),
        };
        let (root, height) = client.get_latest_root_and_height().await;
        (root, height)
    }

    pub(crate) fn read_pion_1_vault_contract_address() -> String {
        dotenv().ok();
        env::var("NEUTRON_PION_1_VAULT_EXAMPLE_CONTRACT_ADDRESS")
            .expect("Missing Pion 1 Vault Contract Address!")
    }

    pub(crate) fn read_pion_1_default_account_address() -> String {
        dotenv().ok();
        env::var("NEUTRON_DEFAULT_ACCOUNT_ADDRESS")
            .expect("Missing Neutron Default Account Address!")
    }

    use std::fs;
    use std::path::PathBuf;

    fn read_bytes_from_file(path: &str) -> std::io::Result<Vec<u8>> {
        fs::read(path)
    }

    pub(crate) fn get_test_vector_neutron_storage_proof() -> Vec<u8> {
        let path: PathBuf = [
            env!("CARGO_MANIFEST_DIR"),
            "src/merkle_lib/tests/data/storage_proof.bin",
        ]
        .iter()
        .collect();
        read_bytes_from_file(path.to_str().unwrap()).unwrap()
    }

    pub(crate) const TEST_VECTOR_NEUTRON_ROOT: &str =
        "xuPL4Vt/UqXOvYfaVNsE5rqtOqB3j1UIi2GLB7SvPNY=";
}
