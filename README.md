> ![WARNING]
> This library is under heavy development and has not been audited

# Opening proofs for the Zk Coprocessor
This workspace exposes different libraries that can be used to write Valence ZK programs.
At the core of this project are two fundamental libraries that should be implemented for every supported network.:

- a library for merkle proofs that implements the `MerkleVerifiable` trait from `common/merkle/types.rs`
- a library that implements the `GenericMessage` trait from `TODO`

# Developer Notes
# Supported proof types
Obtain storage proofs for 

[Ethereum](ethereum/README.md)

[Neutron](neutron/README.md)

# Example ZK Programs that depend on our Libraries
Multi-chain merkle openings in ZK, [here](example-programs/sp1/multi-chain-merkle-guest/prover/README.md)
