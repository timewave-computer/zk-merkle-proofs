use std::time::Instant;
pub const MERKLE_ELF: &[u8] = include_elf!("cross-chain-merkle-guest");

use alloy::dyn_abi::SolType;
use cross_chain_merkle_program_types::EthereumProofBatch;
use prover_utils::merkle::types::MerkleProofInput;
/// entry point for the proving service
/// this function will be used to prove the merkle-program execution
/// the merkle-program will use verify_merkle_proof to verify one or more opening(s)
use sp1_sdk::{include_elf, ProverClient, SP1Stdin};

pub fn prove(input: MerkleProofInput) {
    let start_time = Instant::now();
    #[allow(deprecated)]
    let client = ProverClient::new();
    let mut stdin = SP1Stdin::new();
    // note that when verifying the merkle proof a trusted root should be used
    // instead of the root hash from input
    let proof_input = serde_json::to_vec(&input).unwrap();
    stdin.write(&proof_input);
    let (pk, vk) = client.setup(MERKLE_ELF);
    let proof = client
        .prove(&pk, &stdin)
        .groth16()
        .run()
        .expect("Failed to generate proof!");
    client.verify(&proof, &vk).expect("Failed to verify proof!");
    let circuit_out =
        EthereumProofBatch::abi_decode(&proof.public_values.as_slice(), false).unwrap();
    println!("Ethereum Proof Outputs: {:?}", &circuit_out);
    let duration = start_time.elapsed();
    println!("Elapsed time: {:?}", duration);
}

#[cfg(feature = "zk-tests")]
#[cfg(test)]
mod tests {
    use crate::prove;
    use ethereum::merkle_lib::tests::test_vector::TEST_VECTOR_ETH_STORAGE_PROOF;
    use neutron::merkle_lib::{
        tests::test_vector::{read_test_vector_merkle_root, TEST_VECTOR_NEUTRON_STORAGE_PROOF},
        types::NeutronMerkleProofWithRoot,
    };
    use prover_utils::merkle::types::MerkleProofInput;

    #[tokio::test]
    async fn test_generate_proof_cross_chain_merkle_program() {
        let eth_proof: ethereum::merkle_lib::types::EthereumMerkleProof =
            serde_json::from_slice(&TEST_VECTOR_ETH_STORAGE_PROOF).unwrap();
        // we need to hash the key unless this is a receipt proof
        let neutron_proof: neutron::merkle_lib::types::NeutronMerkleProof =
            serde_json::from_slice(&TEST_VECTOR_NEUTRON_STORAGE_PROOF).unwrap();
        prove(MerkleProofInput {
            // pass a list of storage proofs to be verified in zk
            // for now we pass only one ETHEREUM merkle proof for the SUPPLY slot of the USDT contract
            ethereum_proofs: vec![eth_proof],
            neutron_proofs: vec![NeutronMerkleProofWithRoot {
                proof: neutron_proof,
                #[allow(deprecated)]
                root: base64::decode(read_test_vector_merkle_root()).unwrap(),
            }],
        });
    }
}

/*
Ethereum Proof Outputs: EthereumProofBatch { proofs: [EthereumMerkleProofOutput { root: 0x65a7cbd233dbf35c15b158468ab9c756b751ee553003dd66cbc096c4f676c417, key: 0x290decd9548b62a8d60345a988386fc84ba6bc95484008f6362f93160ef3e563, value: 0xf49d39548b62a8d60345a988386fc84ba6bc95484008f6362f93160ef3e5639594c6cde7c39eb2f0f0095f41570af89efc2c1ea828 }, EthereumMerkleProofOutput { root: 0xc6e3cbe15b7f52a5cebd87da54db04e6baad3aa0778f55088b618b07b4af3cd6, key: 0x7b22707265666978223a2262616e6b222c227072656669785f6c656e223a342c226b6579223a22303037353665373437323665227d, value: 0x373030303030303030303030303030 }] }
*/
