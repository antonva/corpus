# Corpus

A quadratic voting parachain based on the 
[Cumulus](https://github.com/paritytech/cumulus/)-based Substrate node.

## Getting started

### Build
```shell
cargo build --release
```
### Run
The assumed `ParaId` is 2000 and has been tested runnig on a version of the
rococo local testnet named cambrelay which can be found [here](https://github.com/antonva/cambrelay). There's nothing special about this testnet so this should work on any relay network, there is however a stored chainspec for the cambrelay
network in this node.

```
./scripts/start-collator-node.sh
```
This will attempt to clean up the storage in `/tmp/corpus-parachain` and then start a collator node as Alice.

### Setup
Note that you generally do not need to run these as the
runtime and chainspec is included in the repository.

Also note that there is a problem with the generation of the chainspec
where it will add a bootnode even if you don't want one. Should you get an
error such as: 

```shell
💔 The bootnode you want to connect provided a different peer ID than the one you expect:...
```

Then it should be fine to just carry on, but you can remove the entry from 
the raw chainspec to get rid of it.

#### Generating chainspec
```shell
# From the root of the repository
./scripts/generate-chain-spec.sh
```

This will generate 3 separate json files that can be used to start the collator
and register a parachain. The files are located in `./node/service/chain-spec`
and `./node/service/raw-chain-spec`.

#### Generating wasm and genesis for parachains
```shell
# From the root of the repository
./scripts/generate-genesis-wasm.sh
```

This will create 2 files, the wasm runtime at genesis and the state at genesis.
Both files will be in `./node/service/genesis/`

## Using the pallets
Corpus implements two new pallets in addition to being run with Sudo at the moment.
These are `pallet-quadravote` and `pallet-votingregistry`.

### Pallet Voting Registry 
Serves the function of providing an identity for an account
in order to vote. In this simple scheme there are no meatspace validations made 
by a registrar so any account is a valid voter if they so choose. There is a runtime
adjustable amount of reserved currency reserved for being a registered voter.

Extrinsics:

- register: Registers the account which sent the transaction as a voter/proposer and reserves tokens.
- deregister: Removes the account from storage if exists and unreserves the tokens previously reserved.

Traits:

- IdentityInterface: A trait with a single function, is_identified, which takes an account and asks
`pallet_votingregistry` if it exists in storage.

### Pallet Quadravote

The quadratic voting system pallet

Extrinsics:

- create_proposal:
- withdraw_proposal:
- cast_vote:
