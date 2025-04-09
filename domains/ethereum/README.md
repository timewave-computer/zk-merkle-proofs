# Ethereum libraries

## Merkle Proofs
The Ethereum `merkle_lib` library exposes functions to obtain different kinds of storage and account proofs from an Ethereum node, as well as verify them against a `trusted root`.

## Account Proofs
When verifying an account proof, we must use the keccak hash of the account address as the key.
All data that is stored under the account can be verified against the account root.
The account root is what lives under the keccak hash of the account address.

Therefore we have a nested structure where we first want to verify the account storage root against the trusted
light client root and then the stored value against that account root.

[click to return home](../../README.md)