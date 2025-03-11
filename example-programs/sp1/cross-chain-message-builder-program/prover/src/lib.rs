use std::time::Instant;
pub const MERKLE_ELF: &[u8] = include_elf!("cross-chain-message-builder-guest");
/// entry point for the proving service
/// this function will be used to prove the merkle-program execution
/// the merkle-program will use verify_merkle_proof to verify one or more opening(s)
use sp1_sdk::{include_elf, ProverClient, SP1Stdin};
use types::MessageBuilderProgramInput;

pub fn prove(input: MessageBuilderProgramInput) {
    let proof_input = serde_json::to_vec(&input).unwrap();
    let start_time = Instant::now();
    let client = ProverClient::new();
    let mut stdin = SP1Stdin::new();
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
    use types::MessageBuilderProgramInput;

    use crate::prove;

    #[tokio::test]
    async fn test_generate_proof_cross_chain_message_builder_program() {
        prove(MessageBuilderProgramInput {
            from: "0x0000000000000000000000000000000000000000".to_string(),
            to: "0x0000000000000000000000000000000000000000".to_string(),
            amount: 1_000_000_000_000_000_000u64,
        });
    }
}
