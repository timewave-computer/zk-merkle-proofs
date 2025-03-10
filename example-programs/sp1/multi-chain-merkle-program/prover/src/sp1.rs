use std::time::Instant;
pub const MERKLE_ELF: &[u8] = include_elf!("multi-chain-merkle-guest");

/// entry point for the proving service
/// this function will be used to prove the merkle-program execution
/// the merkle-program will use verify_merkle_proof to verify one or more opening(s)
use sp1_sdk::{include_elf, ProverClient, SP1Stdin};
use verification_logic::MerkleProofInput;
pub fn prove(input: MerkleProofInput) {
    let start_time = Instant::now();
    let client = ProverClient::new();
    let mut stdin = SP1Stdin::new();

    // note that when verifying the merkle proof a trusted root should be used
    // instead of the root hash from input
    let proof_input = serde_json::to_vec(&input).unwrap();
    stdin.write(&proof_input);
    let (pk, vk) = client.setup(MERKLE_ELF);
    let proof = client
        .prove(&pk, &stdin)
        .run()
        .expect("Failed to generate proof!");
    client.verify(&proof, &vk).expect("Failed to verify proof!");
    let duration = start_time.elapsed();
    println!("Elapsed time: {:?}", duration);
}

#[cfg(test)]
mod tests {
    use crate::sp1::prove;
    use ethereum::merkle_lib::test_vector::get_ethereum_test_vector_storage_proof;
    use neutron::{
        merkle_lib::test_vector::{
            get_neutron_test_vector_bank_store_supply, read_test_vector_merkle_root,
        },
        merkle_lib::types::NeutronProofWithRoot,
    };
    use verification_logic::MerkleProofInput;

    #[tokio::test]
    async fn test_generate_proof_multi_chain_merkle_program() {
        let eth_proof = get_ethereum_test_vector_storage_proof().await;
        let proof = get_neutron_test_vector_bank_store_supply().await;
        prove(MerkleProofInput {
            // pass a list of storage proofs to be verified in zk
            // for now we pass only one ETHEREUM merkle proof for the SUPPLY slot of the USDT contract
            ethereum_proofs: vec![eth_proof],
            neutron_proofs: vec![NeutronProofWithRoot {
                proof: proof,
                root: base64::decode(read_test_vector_merkle_root()).unwrap(),
            }],
        });
    }
}
