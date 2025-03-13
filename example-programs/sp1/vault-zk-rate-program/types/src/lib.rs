use ethereum::merkle_lib::types::EthereumProof;
use neutron::merkle_lib::types::NeutronProof;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RateProgramInputs {
    // the storage proof for the position balance on neutron
    pub neutron_balance_proof: NeutronProof,
    // the storage proof for the position balance on ethereum
    pub ethereum_balance_proof: EthereumProof,
    // can decode this either in or outside the circuit
    // the total amount of tokens minted on neutron
    pub total_mint_amount_neutron: Vec<u8>,
    // can decode this either in or outside the circuit
    // the total amount of tokens minted on ethereum
    pub total_mint_amount_ethereum: Vec<u8>,
    // the storage proof for the total mint amount on neutron
    pub neutron_mint_proof: NeutronProof,
    // the storage proof for the total mint amount on ethereum
    pub ethereum_mint_proof: EthereumProof,
    // the ethereum root we want to verify against
    pub ethereum_root: Vec<u8>,
    // the neutron root we want to verify against
    pub neutron_root: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RateProgramOutputs {
    // the neutron root that we verify against - ideally provided by a light client
    pub neutron_root: Vec<u8>,
    // the ethereum root that we verify against - ideally provided by a light client
    pub ethereum_root: Vec<u8>,
    // the cross-chain rate for the LP token
    pub rate_encoded: Vec<u8>,
}

// simple calculation
// (neutron_balance + eth_balance)/(mint_amount_neutron + mint_amount_eth)
