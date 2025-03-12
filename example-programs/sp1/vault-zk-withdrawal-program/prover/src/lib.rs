use std::time::Instant;
pub const MERKLE_ELF: &[u8] = include_elf!("vault-zk-withdrawal-guest");
use sp1_sdk::{include_elf, ProverClient, SP1Stdin};
pub fn prove() {}

#[cfg(test)]
mod tests {
    use crate::prove;

    #[tokio::test]
    async fn test_generate_zk_withdrawal_proof() {}
}
