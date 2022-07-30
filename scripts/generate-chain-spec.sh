#!/usr/bin/env sh

set -e

# Generate the chain spec from the built node
./target/release/parachain-template-node build-spec > ./node/service/chain-specs/corpus.json

# Generate a raw, distributable, chainspec with no bootnodes 
./target/release/parachain-template-node build-spec \
	--chain ./node/service/chain-specs/corpus.json \
	--raw \
	--disable-default-bootnode > ./node/service/raw-chain-specs/corpus-raw-distributable.json

# Generate a raw chainspec
./target/release/parachain-template-node build-spec \
	--chain ./node/service/chain-specs/corpus.json \
	--raw > ./node/service/raw-chain-specs/corpus-raw.json
