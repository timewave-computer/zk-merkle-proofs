# Multi-chain merkle openings in ZK
To run the example for multi-chain merkle openings in SP1:

```bash
cargo test test_generate_proof_cross_chain_vault_program --release -- --nocapture
```

This will leverage the keccak precompile for the SP1 zkvm and prove a batch filled with one storage proof for Ethereum,
as well as a batch of one storage proof for neutron.