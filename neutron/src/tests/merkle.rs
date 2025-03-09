#[cfg(test)]
mod tests {
    use crate::{
        types::{NeutronProofBatch, NeutronProver},
        verify_merkle_proof,
    };
    use common::MerkleProver;
    use dotenvy::dotenv;
    use std::env;

    #[tokio::test]
    // first verifies account state, then a single storage proof
    // currently the variables need to be manually set before running the test
    async fn test_verify_storage_proof_single() {
        let merkle_root =
            hex::decode("BDF53A9E4DEE71B9B7116B313E2F1D533F9294322868DB5C20B22FEF89B39F55")
                .unwrap();
        //let merkle_root = base64::decode("vfU6nk3ucbm3EWsxPi8dUz+SlDIoaNtcILIv74mzn1U=").unwrap();
        let rpc_url = read_rpc_url();
        let supply_key = construct_supply_key("untrn", vec![0x00]);
        let prover = NeutronProver { rpc_url };
        let proofs = prover
            .get_storage_proof(vec!["bank", &hex::encode(supply_key)], "", 7876)
            .await;
        let proofs_decoded: NeutronProofBatch = serde_json::from_slice(&proofs).unwrap();
        verify_merkle_proof(proofs_decoded, merkle_root);
    }

    fn read_rpc_url() -> String {
        dotenv().ok();
        env::var("NEUTRON_RPC").expect("Missing Neutron RPC url!")
    }

    fn construct_supply_key(denom: &str, prefix: Vec<u8>) -> Vec<u8> {
        let mut key = prefix; // Prefix for supply query in the Cosmos SDK
        key.extend_from_slice(denom.as_bytes()); // Append the denom in UTF-8 encoding
        key
    }
}
