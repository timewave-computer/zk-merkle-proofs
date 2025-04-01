# Timewave RLP
This is a minimized fork of `alloy_rlp` that exposes the necessary functionality
to decode rlp-encoded leaf nodes.

When verifying merkle proofs inside provable (zkvm) programs, this is easier to 
maintain and fix than the entirety of `alloy-rlp`.

[click to return home](../../../../README.md)