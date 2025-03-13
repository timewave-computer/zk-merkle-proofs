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

    let mut eth_diff: u32 = 0;
    let mut neutron_diff: u32 = 0;

    // eth precision must be increased by eth_diff
    if inputs.neutron_precision > inputs.eth_precision {
        eth_diff = inputs.neutron_precision - inputs.eth_precision

    // neutron precision must be increased by neutron_diff
    } else if inputs.eth_precision > inputs.neutron_precision {
        neutron_diff = inputs.eth_precision - inputs.neutron_precision
    }

    let ethereum_balance: U256 =
        alloy_rlp::decode_exact(&inputs.ethereum_balance_proof.value).unwrap();

    let adjusted_ethereum_balance: U256 = adjust_balance(ethereum_balance, eth_diff);

    let ethereum_mint_amount: U256 =
        alloy_rlp::decode_exact(&inputs.ethereum_mint_proof.value).unwrap();

    let adjusted_ethereum_mint_amount: U256 = adjust_balance(ethereum_mint_amount, eth_diff);

    let neutron_balance: U256 =
        U256::from(decode_neutron_value(inputs.neutron_balance_proof.value));

    let adjusted_neutron_balance: U256 = adjust_balance(neutron_balance, neutron_diff);

    let neutron_mint_amount: U256 =
        U256::from(decode_neutron_value(inputs.neutron_mint_proof.value));

    let adjusted_neutron_mint_amount: U256 = adjust_balance(neutron_mint_amount, neutron_diff);

    // calculate the current rate
    let output_rate: U256 = (adjusted_ethereum_mint_amount + adjusted_neutron_mint_amount)
        / (adjusted_ethereum_balance + adjusted_neutron_balance);

    sp1_zkvm::io::commit_slice(
        &serde_json::to_vec(&RateProgramOutputs {
            neutron_root: inputs.neutron_root,
            ethereum_root: inputs.ethereum_root,
            rate_encoded: output_rate.to_be_bytes_vec(),
        })
        .unwrap(),
    );
}

fn adjust_balance(balance_before: U256, exponent: u32) -> U256 {
    balance_before * U256::from(10u32.pow(exponent))
}

// decode bytes to u128
fn decode_neutron_value(bytes: Vec<u8>) -> u128 {
    let string = String::from_utf8(bytes).unwrap();
    u128::from_str_radix(&string, 10).unwrap()
}
