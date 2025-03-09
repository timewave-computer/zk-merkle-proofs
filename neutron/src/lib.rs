use helpers::convert_tm_to_ics_merkle_proof;
use ics23::{
    calculate_existence_root, commitment_proof::Proof, iavl_spec, tendermint_spec,
    verify_membership,
};
use types::NeutronProofBatch;

pub mod helpers;
mod tests;
pub mod types;

pub fn verify_merkle_proof(proofs: NeutronProofBatch, expected_app_hash: Vec<u8>) {
    for proof in proofs.batch {
        let proof_decoded = convert_tm_to_ics_merkle_proof(&proof.proofs);
        let inner_proof = proof_decoded.first().unwrap();
        let Some(Proof::Exist(existence_proof)) = &inner_proof.proof else {
            panic!("Wrong proof type!");
        };
        let inner_root =
            calculate_existence_root::<ics23::HostFunctionsManager>(existence_proof).unwrap();
        let is_valid = verify_membership::<ics23::HostFunctionsManager>(
            &inner_proof,
            &iavl_spec(),
            &inner_root,
            &hex::decode(proof.key.1).unwrap(),
            &proof.value,
        );
        assert!(is_valid);
        let outer_proof = proof_decoded.last().unwrap();
        let is_valid = verify_membership::<ics23::HostFunctionsManager>(
            &outer_proof,
            &tendermint_spec(),
            &expected_app_hash,
            &proof.key.0.as_bytes(),
            &inner_root,
        );
        assert!(is_valid);
    }
}
