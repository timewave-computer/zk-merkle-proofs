pub mod types;
use common::{merkle::types::MerkleProofOutput, merkle::types::MerkleVerifiable};

pub fn verify_merkle_proof<T: MerkleVerifiable>(
    proof: T,
    expected_root: &[u8],
) -> MerkleProofOutput {
    proof.verify(expected_root)
}
