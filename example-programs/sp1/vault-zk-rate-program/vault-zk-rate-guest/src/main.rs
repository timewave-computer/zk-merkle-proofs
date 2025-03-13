#![no_main]

use alloy_primitives::U256;
use prover_utils::merkle::verify_merkle_proof;
use vault_zk_rate_types::{RateProgramInputs, RateProgramOutputs};
sp1_zkvm::entrypoint!(main);

pub fn main() {
    let inputs: RateProgramInputs =
        serde_json::from_slice(&sp1_zkvm::io::read::<Vec<u8>>()).unwrap();
    // todo: avoid cloning, implement for &Proof
    verify_merkle_proof(inputs.ethereum_balance_proof.clone(), &inputs.ethereum_root);
    verify_merkle_proof(inputs.neutron_balance_proof.clone(), &inputs.neutron_root);
    verify_merkle_proof(inputs.ethereum_mint_proof.clone(), &inputs.ethereum_root);
    verify_merkle_proof(inputs.neutron_mint_proof.clone(), &inputs.neutron_root);

    let ethereum_balance: U256 =
        alloy_rlp::decode_exact(&inputs.ethereum_balance_proof.value).unwrap();
    let ethereum_mint_amount: U256 =
        alloy_rlp::decode_exact(&inputs.ethereum_mint_proof.value).unwrap();
    let neutron_balance: U256 =
        U256::from(decode_neutron_value(inputs.neutron_balance_proof.value));
    let neutron_mint_amount: U256 =
        U256::from(decode_neutron_value(inputs.neutron_mint_proof.value));

    // calculate the current rate
    let output_rate: U256 =
        (ethereum_mint_amount + neutron_mint_amount) / (neutron_balance + ethereum_balance);

    sp1_zkvm::io::commit_slice(
        &serde_json::to_vec(&RateProgramOutputs {
            neutron_root: inputs.neutron_root,
            ethereum_root: inputs.ethereum_root,
            rate_encoded: output_rate.to_be_bytes_vec(),
        })
        .unwrap(),
    );
}

// decode bytes to u128
fn decode_neutron_value(bytes: Vec<u8>) -> u128 {
    let string = String::from_utf8(bytes).unwrap();
    u128::from_str_radix(&string, 10).unwrap()
}
