use alloy_sol_types::sol;
use cosmwasm_schema::cw_serde;
sol!(
    #[derive(Debug)]
    struct EthereumMerkleProofOutput{
        bytes32 root;
        bytes key;
        bytes value;
    }
);
sol!(
    #[derive(Debug)]
    struct EthereumProofBatch{
        EthereumMerkleProofOutput[] proofs;
    }
);

#[cw_serde]
pub struct CosmosMerkleProofOutput {
    // commit to the encoded roots
    pub root: Vec<u8>,
    // the keys that were queried
    pub key: Vec<u8>,
    // commit to the encoded values
    pub value: Vec<u8>,
}

#[cw_serde]
pub struct CosmosProofBatch {
    proofs: Vec<CosmosMerkleProofOutput>,
}
