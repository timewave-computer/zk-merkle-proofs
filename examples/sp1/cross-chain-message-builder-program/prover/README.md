# Multi-chain merkle openings in ZK
This is a simple example for building an executable Ethereum message inside a ZK Valence program. While not particularily useful as-is, this concept can be combined with other examples, like the `cross-chain-merkle-program`, to construct a set of on-chain calls in our off-chain execution environment.

```bash
cargo test test_generate_proof_cross_chain_message_builder_program --features zk-tests --features sp1 --release -- --nocapture
```

This will generate an Ethereum encoded proof that can be sent to an Ethereum smart contract linked to a Valence program, to verify and udpate cross-chain state, based on off-chain computation results.

[click to return home](../../../../README.md)