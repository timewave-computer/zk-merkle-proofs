> ![WARNING]
> This library is under heavy development and has not been audited

# Opening proofs for the Zk Coprocessor
This library exposes an interface and implementations to prove the verification of network and account storage proofs in ZKVMs.
We are especially interested in Neutron and Ethereum for the time being, but the library will be extended to support different networks
and perhaps layers.

We implement a `MerkleProver` interface for different `domains`. Domains can be either account abstractions (valence accounts) or any other
network that has a triestore and data availability layer for merkle proofs.

The inputs to the ZK programs (regardless of the proving backend) are standardized to this format:

```rust
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MerkleProof {
    // a list of serialized nodes
    // the last node should be the queried value on ethereum
    pub nodes: Vec<Vec<u8>>,
    // on neutron the value is supplied seperately
    pub value: Option<Vec<u8>>,
    // the key that we query
    pub key: Vec<u8>,
    // target domain
    pub domain: Domain,
    // serialized trie root
    pub root: Vec<u8>,
}
```

see [here](common/types.rs).

# Developer Notes
## Optimizations
- todo: use borsh instead of serde

# Supported proof types
Obtain storage proofs for 

[Ethereum](ethereum/README.md)

[Neutron](neutron/README.md)

# ZK opening proof
Read more [here](prover/README.md)
