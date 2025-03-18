# Neutron libraries

## Merkle Proofs
The Neutron `merkle_lib` library exposes functions to obtain different kinds of storage and account proofs from an Neutron node, as well as verify them against a `trusted root`.

In the case of Neutron the scope of these proofs includes, but is not limited to the `bank` and `wasm` store. 

This means that one can obtain a merkle proof for the total supply or balance of a token, or for a value stored in a mapping under some smart contract.

[click to return home](../../README.md)