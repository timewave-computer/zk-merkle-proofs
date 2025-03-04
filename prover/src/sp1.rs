use std::time::Instant;
pub const MERKLE_ELF: &[u8] = include_elf!("merkle-program");

/// entry point for the proving service
/// this function will be used to prove the merkle-program execution
/// the merkle-program will use verify_merkle_proof to verify one or more opening(s)
use common::ProofInput;
use sp1_sdk::{include_elf, ProverClient, SP1Stdin};
pub fn prove(input: ProofInput) {
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
