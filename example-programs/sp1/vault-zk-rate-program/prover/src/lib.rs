use std::time::Instant;
pub const MERKLE_ELF: &[u8] = include_elf!("cross-chain-message-builder-guest");
use cross_chain_message_builder_types::MessageBuilderProgramInput;
/// entry point for the proving service
/// this function will be used to prove the merkle-program execution
/// the merkle-program will use verify_merkle_proof to verify one or more opening(s)
use sp1_sdk::{include_elf, ProverClient, SP1Stdin};
pub fn prove() {
    
}

#[cfg(test)]
mod tests {
    use crate::prove;
    use cross_chain_message_builder_types::MessageBuilderProgramInput;

    #[tokio::test]
    async fn test_generate_zk_rate_proof() {

    }
}
