//! Helper functions for working with Merkle proofs on the Neutron blockchain.
//!
//! This module provides utility functions for converting between different
//! Merkle proof formats and handling proof-related operations.

use {cosmrs::proto::prost, ics23::CommitmentProof, tendermint::merkle::proof::ProofOps};

/// Converts a Tendermint proof to an ICS23 commitment proof.
///
/// This function takes a Tendermint proof operations structure and converts it
/// into a vector of ICS23 commitment proofs. The conversion is specific to the
/// Neutron blockchain's implementation of the ICS23 standard.
///
/// # Arguments
///
/// * `tm_proof` - The Tendermint proof operations to convert
///
/// # Returns
///
/// A vector of ICS23 commitment proofs
pub fn convert_tm_to_ics_merkle_proof(tm_proof: &ProofOps) -> Vec<CommitmentProof> {
    let mut out: Vec<CommitmentProof> = vec![];
    assert_eq!(tm_proof.ops.len(), 2);
    let proof_op = tm_proof
        .ops
        .first()
        //.find(|op| op.r#field_type == "ics23:iavl")
        .unwrap();

    let mut parsed = CommitmentProof { proof: None };
    prost::Message::merge(&mut parsed, proof_op.data.as_slice()).unwrap();
    out.push(parsed);

    let proof_op = tm_proof.ops.last().unwrap();
    let mut parsed = CommitmentProof { proof: None };
    prost::Message::merge(&mut parsed, proof_op.data.as_slice()).unwrap();
    out.push(parsed);
    out
}
