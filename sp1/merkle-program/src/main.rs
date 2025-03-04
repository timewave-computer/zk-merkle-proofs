#![no_main]
sp1_zkvm::entrypoint!(main);
use common::{Domain, ProofInput, ProofOutput};
use ethereum::verify_merkle_proof;
use serde_json;
/// the logic that is to be proven
/// will likely call external functions, primarily verify_merkle_proof
/// enable sp1 as a feature to use keccak precompile
pub fn main() {
    let mut output: ProofOutput = ProofOutput {
        roots: vec![],
        keys: vec![],
        values: vec![],
    };
    let proof_batch: ProofInput = serde_json::from_slice(&sp1_zkvm::io::read::<Vec<u8>>()).unwrap();
    for proof in proof_batch.proofs {
        match proof.domain {
            Domain::ETHEREUM => {
                // verify an ethereum proof
                verify_merkle_proof(proof.root.clone(), proof.nodes.clone(), &proof.key);
                output.roots.push((proof.domain, proof.root.clone()));
                output.keys.push(proof.key.clone());
                output
                    .values
                    // push the leaf that contains the rlp encoded value
                    .push(serde_json::to_vec(&proof.nodes.last().unwrap()).unwrap());
            }
            Domain::COSMOS => {
                todo!("Cosmos support is pending, try with Ethereum for now!")
                // verify a cosmos proof
            }
        }
    }
    // commit the serialized state roots and read values
    sp1_zkvm::io::commit_slice(&serde_json::to_vec(&output).unwrap());
}
