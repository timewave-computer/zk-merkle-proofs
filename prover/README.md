# ZK Opening Proofs

# Run Tests
To generate a ZK storage proof for the supply of the USDT contract on Ethereum, run:

```bash
cargo test test_generate_proof --release -- --nocapture
```

This will leverage the keccak precompile for the SP1 zkvm.
Note that the precompile adds some overhead and is more efficient for larger proof batches