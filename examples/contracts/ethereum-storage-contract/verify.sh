source ../../.env
forge verify-contract $1 --chain-id 11155111 \
  --rpc-url $SEPOLIA_URL \
  --compiler-version 0.8.13 \
  --constructor-args 0x00000000000000000000000051df57d545074ba4b2b04b5f973efc008a2fde6e000000000000000000000000000000000000000000000000000000000000000a \
  --watch


# cast abi-encode "constructor(address, uint256)" 0x51df57D545074bA4b2B04b5f973Efc008A2fde6E 10
