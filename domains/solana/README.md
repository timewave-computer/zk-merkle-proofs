# Solana libraries

Note: Solana is not a priority right now, Neutron/Eth Vault First!

Solana is tricky, we might have to develop a co-processor specifically for it.
My current idea is that we could have a solana co-processor that is deployed for each application, 
in combination with an MPC that provides state roots of coprocessor readings.

Since Solana doesn't really have a trie / tree with presistent state. We can monitor state
and build our own trie. Multiple nodes could agree on the state of said trie and send commitments
on-chain. If the commitment threshold is met, Solana on-chain state root is updated and merkle openings
can be accepted.

These nodes would not have to synchronize with one another, they just each individually attest to the state at height 
E * n, where E is the epoch block range and n is the current epoch index.

## Requirements solana coprocessor
1. Query solana account state
2. Construct a merkle trie from said state - we can use the eth_trie library for simplicity
3. Have a set of operators attest to said state
4. Post ZK opening proofs on-chain and verify them against the root that is updated by those operators
5. A smart contract that maintains the root history of our custom trie and updates it when sufficiently many attestations are pushed

## Trie representation of on-chain data

The user (valence app operator) will define which accounts on Solana they are interested in.

E.g. Input: Vec<Address||Key>

For sake of simplicity we can start with just token balances.

They also define an update epoch e.g. how many blocks should pass in between state updates.

They instantiate the solana co-processor contract with a set of keys or a multisig wallet:

```json
CoprocessorConfig{
    Accounts: Vec<Address||Key>,
    EpochDuration: 10,
    Key: Share of multisig, or regular ECDSA|RSA
}
```

The coprocessors will start reading state and construct a trie from it:

```rust
trie: Trie = Trie::new()
trie.insert(Account1)
trie.insert(Account2)
trie.hash_root()
```

They will sign the root and send it to the co-processor contract.

That's all, now we have an on-chain representation of the state of the accounts on Solana that we are interested in for the Valence application.

While there are trust assumptions, they are in control of each app developer deploying the coprocessors.

We could also have the co-processors submit state to an Ethereum contract and include the root in our MAIN co-processor's Trie/Tree.

## Merkle Openings
The zk openings will work the exact same for Solana, except we verify against the Trie that is constructed by our special solana co-processor.
