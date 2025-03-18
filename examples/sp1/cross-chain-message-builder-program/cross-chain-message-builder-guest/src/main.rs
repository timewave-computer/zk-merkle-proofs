#![no_main]
sp1_zkvm::entrypoint!(main);
use std::str::FromStr;

use alloy_primitives::{Address, U256};
use alloy_sol_types::{sol, SolCall};
use cross_chain_message_builder_types::MessageBuilderProgramInput;

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
    let transfer_arguments: MessageBuilderProgramInput =
        serde_json::from_slice(&sp1_zkvm::io::read::<Vec<u8>>()).unwrap();
    // construct messages for the target domain where this proof will be verified
    // we strive to make this experience more seamless by providing a cross-chain message encoder
    // as part of the core libraries that are implemented for each domain
    let erc20_transfer = ERC20::transferFromCall {
        from: Address::from_str(&transfer_arguments.from).unwrap(),
        to: Address::from_str(&transfer_arguments.to).unwrap(),
        amount: U256::from(transfer_arguments.amount),
    }
    .abi_encode();
    sp1_zkvm::io::commit_slice(&erc20_transfer);
}
