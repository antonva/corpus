#!/usr/bin/env sh

set -e

# Export genesis wasm
./target/release/parachain-template-node export-genesis-wasm \
	--chain ./node/service/raw-chain-specs/corpus-raw.json > ./node/service/genesis/corpus-genesis-wasm
# Export genesis state
./target/release/parachain-template-node export-genesis-state \
	--chain ./node/service/raw-chain-specs/corpus-raw.json > ./node/service/genesis/corpus-genesis-state
