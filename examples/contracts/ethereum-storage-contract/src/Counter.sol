// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

contract Counter {
    mapping(address => uint256) public numbers;

    constructor(address initialAddress, uint256 initialNumber) {
        numbers[initialAddress] = initialNumber;
    }

    function setNumber(uint256 newNumber) public {
        numbers[msg.sender] = newNumber;
    }

    function increment() public {
        numbers[msg.sender]++;
    }
}
