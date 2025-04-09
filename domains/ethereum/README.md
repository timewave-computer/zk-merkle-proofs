# Ethereum Libraries

This directory contains Ethereum-specific implementations for working with Merkle proofs and interacting with Ethereum nodes.

## Merkle Proofs

The `merkle_lib` library provides a comprehensive set of tools for working with Ethereum Merkle proofs. It supports:

- Account proofs
- Storage proofs
- Receipt proofs

### Implementation Details

The library is built on top of the following components:
- `timewave_rlp`: For RLP encoding/decoding
- `timewave_trie`: For Merkle Patricia Trie operations
- `keccak`: For Ethereum's keccak256 hashing

### Proof Types

#### Account Proofs
When verifying an account proof, we must use the keccak hash of the account address as the key.
All data that is stored under the account can be verified against the account root.
The account root is what lives under the keccak hash of the account address.

Therefore we have a nested structure where we first want to verify the account storage root against the trusted
light client root and then the stored value against that account root.

#### Storage Proofs
Storage proofs allow verification of specific storage slots within an account's storage trie.
Each storage slot is identified by its keccak256 hash.

#### Receipt Proofs
Receipt proofs enable verification of transaction receipts, which contain information about the execution of transactions
including status, gas used, and logs.

### Features

1. `no-zkvm`: Standard implementation with full Ethereum node interaction capabilities
   - Includes RPC client support
   - Full alloy types support
   - Async runtime support

2. `sp1`: Zero-knowledge virtual machine optimized implementation
   - Uses SP1-optimized keccak implementation
   - Suitable for ZK proof generation

### Usage

The library provides two main proof types:
- `EthereumMerkleProof`: Standard proof type with hashed keys
- `EthereumRawMerkleProof`: Proof type with raw (unhashed) keys

Both types implement the `MerkleVerifiable` trait, allowing verification against trusted roots.

[click to return home](../../README.md)