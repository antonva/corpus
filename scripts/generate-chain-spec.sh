#!/usr/bin/env sh

PARA_ID=2000

set -e

# Generate the chain spec from the built node
./target/release/parachain-template-node build-spec > ./node/service/chain-specs/corpus.json

sed -i "s/\"para_id\": 1000/\"para_id\": $PARA_ID/g" ./node/service/chain-specs/corpus.json
sed -i "s/\"parachainId\": 1000/\"parachainId\": $PARA_ID /g" ./node/service/chain-specs/corpus.json
sed -i "s/\"relay_chain\": \"rococo-local\"/\"relay_chain\": \"rococo_local_testnet\"/g" ./node/service/chain-specs/corpus.json

# Generate a raw, distributable, chainspec with no bootnodes
./target/release/parachain-template-node build-spec \
	--chain ./node/service/chain-specs/corpus.json \
	--raw \
	--disable-default-bootnode > ./node/service/raw-chain-specs/corpus-raw-distributable.json

# Generate a raw chainspec
./target/release/parachain-template-node build-spec \
	--chain ./node/service/chain-specs/corpus.json \
	--raw > ./node/service/raw-chain-specs/corpus-raw.json
