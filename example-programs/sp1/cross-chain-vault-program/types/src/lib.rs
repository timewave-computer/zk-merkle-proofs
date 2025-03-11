use ethereum::merkle_lib::types::EthereumProof;
use neutron::merkle_lib::types::NeutronProofWithRoot;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Rules {
    // source chain vault allocation percent 0-100
    pub allocation_source: u32,
    // destination chain value allocation percent 0-100
    pub allocation_destination: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VaultProgramInput {
    pub source_proof: EthereumProof,
    pub destination_proof: NeutronProofWithRoot,
    pub rules: Rules,
    pub source_balance: u64,
    pub destination_balance: u64,
}
