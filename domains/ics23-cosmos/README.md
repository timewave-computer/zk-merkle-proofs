# Ics23-Cosmos libraries

## Merkle Proofs
The Neutron `merkle_lib` library exposes functions to obtain different kinds of storage and account proofs from an Neutron node, as well as verify them against a `trusted root`.

In the case of Neutron the scope of these proofs includes, but is not limited to the `bank` and `wasm` store. 

This means that one can obtain a merkle proof for the total supply or balance of a token, or for a value stored in a mapping under some smart contract.

>[!NOTE]
> Most (if not all) Cosmos Chains should have the same key value bank and wasm store 
> as Neutron. Therefore the NeutronKey and MerkleLibrary can be re-used!

[click to return home](../../README.md)