#![no_main]
sp1_zkvm::entrypoint!(main);
use alloy_primitives::FixedBytes;
use alloy_sol_types::SolValue;
use common::merkle::types::MerkleProofOutput;
use cross_chain_merkle_program_types::{EthereumMerkleProofOutput, EthereumProofBatch};
use prover_utils::merkle::{types::MerkleProofInput, verify_merkle_proof};
pub fn main() {
    let mut outputs: Vec<MerkleProofOutput> = vec![];
    let proof_batch: MerkleProofInput =
        serde_json::from_slice(&sp1_zkvm::io::read::<Vec<u8>>()).unwrap();
    // verify and commit a batch of Ethereum merkle proofs
    for mut proof in proof_batch.ethereum_proofs {
        let raw_key = proof.key.clone();
        proof.hash_key();
        let verification_output = verify_merkle_proof(proof.clone(), &proof.root.clone());
        outputs.push(MerkleProofOutput {
            root: verification_output.root,
            key: raw_key,
            value: verification_output.value,
            domain: common::merkle::types::Domain::ETHEREUM,
        });
        outputs.push(verify_merkle_proof(proof.clone(), &proof.root.clone()));
    }
    // verify and commit a batch of neutron storage proofs
    for proof in proof_batch.neutron_proofs {
        outputs.push(verify_merkle_proof(proof.clone(), &proof.root));
    }
    let mut ethereum_abi_encoded_proof_batch: Vec<EthereumMerkleProofOutput> = vec![];
    for proof in outputs {
        let ethereum_abi_encoded_merkle_proof_outputs = EthereumMerkleProofOutput {
            root: FixedBytes::<32>::from_slice(&proof.root),
            key: proof.key.into(),
            value: proof.value.into(),
        };
        ethereum_abi_encoded_proof_batch.push(ethereum_abi_encoded_merkle_proof_outputs);
    }
    let ethereum_outputs = EthereumProofBatch::abi_encode(&EthereumProofBatch {
        proofs: ethereum_abi_encoded_proof_batch,
    });
    sp1_zkvm::io::commit_slice(&ethereum_outputs);
}
