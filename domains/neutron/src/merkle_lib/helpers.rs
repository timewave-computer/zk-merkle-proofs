use {cosmrs::proto::prost, ics23::CommitmentProof, tendermint::merkle::proof::ProofOps};

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
