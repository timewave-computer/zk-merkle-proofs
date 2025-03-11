> [!WARNING]
> This library is under heavy development and has not been audited

# Opening proofs for the Zk Coprocessor
This workspace exposes different libraries that can be used to write Valence ZK programs.
At the core of this project are two fundamental libraries that should be implemented for every supported network.:

- a library for merkle proofs that implements the `MerkleVerifiable` trait from `common/merkle/types.rs`
- a library that implements the `GenericMessage` trait from `TODO`

# Supported Domains
[Ethereum](domains/ethereum/README.md)

[Neutron](domains/neutron/README.md)

# Example ZK Programs that depend on our Libraries
1. Cross-chain merkle openings in ZK, [here](example-programs/sp1/cross-chain-merkle-program/prover/README.md)
2. Executable message builder in ZK, [here](example-programs/sp1/cross-chain-message-builder-program/prover/README.md)
