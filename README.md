# Introduction to Valence ZK
This workspace exposes different libraries that can be used to write Valence co-programs in ZK. The idea is to move some of the math that would usually happen inside smart contracts on different chains off-chain, where anyone in the world can perform the necessary computations locally, to then submit a cryptographic proof to all of our supported networks that will verify the results against a unified state. 

In the broader context of Valence, we call this unified state the `zk-coprocessor`, which aggregates information from different [light clients](https://a16zcrypto.com/posts/article/an-introduction-to-light-clients/) into a single cryptographic commitment. By offloading some of the computations that happen inside our cross-chain Valence programs, we not only strive to optimize gas costs, but also minimize trust assumptions across  the wider blockchain ecosystem. 

Our modular design of (ZK) Valence libraries allows us to easily integrate with new zk lightclients and enable developers to choose their own security / verification stack, aside from the default stack that we provide.

## Getting Started

### Prerequisites
- Rust 1.84.0 or later
- Basic understanding of blockchain concepts and zero-knowledge proofs
- Familiarity with Merkle trees and cryptographic proofs

### Core Components
| Component | Description |
|-----------|-------------|
| Common Library | Core traits and types for Merkle proof verification |
| Chain Libraries | Chain-specific implementations of Merkle proof generation and verification |
| ZK Programs | Example programs demonstrating cross-chain operations |
| SP1 Integration | Zero-knowledge virtual machine integration for proof generation |

### Key Features

- **Cross-Chain Merkle Proofs**: Generate and verify Merkle proofs across different blockchain networks
- **Storage Proof Verification**: Verify storage slot values using Merkle proofs
- **Account Proof Verification**: Verify account state using Merkle proofs
- **Receipt Proof Verification**: Verify transaction receipts using Merkle proofs
- **Modular Design**: Easy integration with new blockchain networks and ZK proving systems

# Adding new Chains to ZK Valence
At the core of this project are chain-specific merkle proof libraries that implements the `MerkleVerifiable` generic from  [common](common/src/merkle/types.rs).
This trait must be implemented for every supported chain. Any chain can easily be added to Valence by simply implementing this trait and choosing either a combination of light clients or utilizing Valence's `zk-coprocessor`.

Ideally the `MerkleProver` trait is also implemented, as well as some helper functions to construct keys and obtain the full scope of merkle proofs.
Ultimately we want to be able to prove any state on any network.

### Implementing a New Chain

To add support for a new blockchain network, you'll need to:

1. Create a new directory in `domains/`
2. Implement the `MerkleVerifiable` trait
3. Implement the `MerkleRpcClient` trait
4. Add chain-specific proof generation and verification logic
5. Add tests for the new implementation

Example implementation structure:
```rust
pub struct NewChainMerkleProof {
    proof: Vec<Vec<u8>>,
    key: Vec<u8>,
    value: Vec<u8>,
}

impl MerkleVerifiable for NewChainMerkleProof {
    fn verify(&self, root: &[u8]) -> bool {
        // Chain-specific verification logic
    }
}
```

# ZK Valence Supported Chains
| Ethereum | Neutron |
|---|---|
| [Readme](domains/ethereum/README.md) | [Readme](domains/neutron/README.md) |

### Chain-Specific Features

| Chain | Account Proofs | Storage Proofs | Receipt Proofs |
|-------|---------------|----------------|----------------|
| Ethereum | ✅ | ✅ | ✅ |
| Neutron | ✅ | ✅ | ❌ |

# ZK Rate calculation for a Cross Chain Vault
We currently have two mock vault contracts deployed on Sepolia (Ethereum) and Pion-1 (Neutron).

| Sepolia | Pion-1 |
|---|---|
| 0x8Fbd2549Dc447d229813ef5139b1aee8a9012eb3 | neutron148w9spa5f9hcwgdy8cnejfel8ly6c2kdazuu94ja5dmy6zyet2ks6c49fd |

Both contracts have the following storage layout:

| Chain | Slot | Data |
|---|---|---|
| Sepolia | 0 | Mapping(Address->Uint256) |
| Sepolia | 1 | Uint256 |
| Pion-1 | 0 | Mapping(Address->Uint128) |
| Pion-1 | 1 | Uint128 |

Where the mapping at slot `0` represents deposit balances and the value at slot `1` represents the total amount of LP shares that have been minted.
Since this is a vault, we have a default account in each mapping that we are interested in.

| Chain | Default Account |
|---|---|
| Sepolia | 0x51df57D545074bA4b2B04b5f973Efc008A2fde6E |
| Pion-1 | neutron148w9spa5f9hcwgdy8cnejfel8ly6c2kdazuu94ja5dmy6zyet2ks6c49fd |

The balance of each account on the respective chain has been initialized to `10` and the shares have also been initialized to `10`. Therefore our total cross-chain rate for this example is `10+10/10+10=20/20= 1 `.

The values in the respective contracts can be updated by anyone. You can review and deploy the contracts yourself, they are located in `examples/contracts/CHAIN_NAME-vault-contract`.

See details about the ZK program that does the cross-chain LP token rate calculation [here](examples/sp1/vault-zk-rate-program/prover/README.md).

# Simple Examples of ZK programs
1. Cross-chain merkle openings in ZK, [here](examples/sp1/cross-chain-merkle-program/prover/README.md)

2. Executable message builder in ZK, [here](examples/sp1/cross-chain-message-builder-program/prover/README.md)

# Test Existing Libraries and Provers

To run the SP1 prover tests (generating real proofs) for all example programs:

```shell
$ cargo test --features sp1 --features zk-tests
```

To test against contracts that have been deployed on `Neutron` (pion-1) and `Ethereum` (Sepolia) testnet:

```shell
$ cargo test
```

To serve the documentation locally:
```shell
$ cargo doc --no-deps --open --features no-zkvm
```