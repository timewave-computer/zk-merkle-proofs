# Multi-chain merkle openings in ZK
This is a simple example of how one can verify merkle proofs (state) from different chains (Neutron, Ethereum) inside an off-chain ZK Valence program and generate a cryptographic proof that said state is valid.

To run the example for multi-chain merkle openings in SP1:

```bash
cargo test test_generate_proof_cross_chain_merkle_program --release --features zk-tests --features sp1 -- --nocapture
```

This will generate an Ethereum encoded proof that can be sent to an Ethereum smart contract linked to a Valence program, to verify and udpate cross-chain state, based on off-chain computation results.

[click to return home](../../../../README.md)