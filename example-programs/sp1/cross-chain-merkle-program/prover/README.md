# Multi-chain merkle openings in ZK
To run the example for multi-chain merkle openings in SP1:

```bash
cargo test test_generate_proof_cross_chain_merkle_program --release -- --nocapture
```

This will leverage the keccak precompile for the SP1 zkvm and prove a batch filled with one storage proof for Ethereum,
as well as a batch of one storage proof for neutron.

Make sure your `.env` at the project root contains these values and that neutron is running.:

```bash
ETH_RPC="https://mainnet.infura.io/v3/"
INFURA="SOME_INFURA_KEY"
TEST_VECTOR_DENOM_NEUTRON="untrn"
TEST_VECTOR_HEIGHT_NEUTRON="0"
TEST_VECTOR_MERKLE_ROOT_NEUTRON="SOME_HEX_ENCODED_APP_HASH"
NEUTRON_RPC="http://localhost:26657"
```