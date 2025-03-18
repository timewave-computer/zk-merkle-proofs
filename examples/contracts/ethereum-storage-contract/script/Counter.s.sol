// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Script, console} from "forge-std/Script.sol";
import {Counter} from "../src/Counter.sol";

contract CounterScript is Script {
    Counter public counter;
    function setUp() public {}
    function run() public {
        vm.startBroadcast();
        counter = new Counter(0x51df57D545074bA4b2B04b5f973Efc008A2fde6E, 10);
        vm.stopBroadcast();
    }
}
