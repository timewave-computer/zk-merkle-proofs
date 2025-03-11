#![no_main]
sp1_zkvm::entrypoint!(main);
use std::str::FromStr;

use alloy_primitives::{Address, U256};
use alloy_sol_types::{sol, SolCall};
use common::merkle::types::ProgramOutputs;
use cross_chain_vault_types::VaultProgramInput;

sol! {
    #[derive(Debug, PartialEq, Eq)]
    contract ERC20 {
        mapping(address account => uint256) public balanceOf;
        constructor(string name, string symbol);
        event Transfer(address indexed from, address indexed to, uint256 value);
        event Approval(address indexed owner, address indexed spender, uint256 value);
        function totalSupply() external view returns (uint256);
        function transfer(address to, uint256 amount) external returns (bool);
        function allowance(address owner, address spender) external view returns (uint256);
        function approve(address spender, uint256 amount) external returns (bool);
        function transferFrom(address from, address to, uint256 amount) external returns (bool);
    }
}

pub fn main() {
    let mut outputs: ProgramOutputs = ProgramOutputs {
        outputs: vec![],
        executable_messages: vec![],
    };
    let transfer_arguments: VaultProgramInput =
        serde_json::from_slice(&sp1_zkvm::io::read::<Vec<u8>>()).unwrap();
    //
    sp1_zkvm::io::commit_slice(&serde_json::to_vec(&outputs).unwrap());
}
