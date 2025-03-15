# Solana libraries
Solana is tricky, we might have to develop a co-processor specifically for it.
My current idea is that we could have a co-processor that is deployed for each application, 
in combination with an MPC that provides state roots of coprocessor readings.

Since Solana doesn't really have a trie / tree with presistent state. We can monitor state
and build our own trie. Multiple nodes could agree on the state of said trie and send commitments
on-chain. If the commitment threshold is met, Solana on-chain state root is updated and merkle openings
can be accepted.

## Requirements solana coprocessor
1. Query solana account state
2. Construct a merkle trie from said state - we can use the eth_trie library for simplicity
3. Have a set of operators attest to said state
4. Post ZK opening proofs on-chain and verify them against the root that is updated by those operators