# Corpus

A quadratic voting parachain based on the 
[Cumulus](https://github.com/paritytech/cumulus/)-based Substrate node.

## Getting started

### Build
```rust
cargo build --release
```

### Generating chainspec
```shell
# From the root of the repository
./scripts/generate-chain-spec.sh
```

This will generate 3 separate json files that can be used to start the collator
and register a parachain. The files are located in `./node/service/chain-spec`
and `./node/service/raw-chain-spec`.

### Generating wasm and genesis for parachains
```shell
# From the root of the repository
./scripts/generate-genesis-wasm.sh
```

This will create 2 files, the wasm runtime at genesis and the state at genesis.
Both files will be in `./node/service/genesis/`
