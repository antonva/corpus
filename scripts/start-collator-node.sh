#!/usr/bin/env sh

set -e

./target/release/parachain-template-node \
	--alice \
	--collator \
	--force-authoring \
	--chain ./node/service/raw-chain-specs/corpus-raw.json \
	--base-path /tmp/parachain/alice \
	--port 40333 \
	--ws-port 8844 \
	-- \
	--execution wasm \
	--chain ./node/service/raw-chain-specs/cambrelay-relay-raw.json \
	--port 30343 \
	--ws-port 9977
