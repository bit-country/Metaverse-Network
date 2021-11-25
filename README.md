<p align="center">
  <img src="https://github.com/bit-country/Metaverse-Network/blob/master/node/res/bm-logo.png?raw=true" width="200">
</p>

<p align="center">  
  <img src="https://raw.githubusercontent.com/w3f/General-Grants-Program/master/src/badge_black.svg" width="530">
</p>

<div align="center">
<h1>[Bit.Country] Metaverse.Network</h1>

## Start your own metaverse. An Ethereum-compatible Network for Metaverses & Games

[![Substrate version](https://img.shields.io/badge/Substrate-3.0.0-brightgreen?logo=Parity%20Substrate)](https://substrate.dev/)
[![Twitter URL](https://img.shields.io/twitter/url?style=social&url=https%3A%2F%2Ftwitter.com%2Fbitdotcountry)](https://twitter.com/bitdotcountry)
[![Medium](https://img.shields.io/badge/Medium-Metaverse-brightgreen?logo=medium)](https://medium.com/@metaverse)

</div>

Development Note: It is still a WIP.

<!-- TOC -->

- [1. Introduction](#1-introduction)
- [2. Overview](#2-overview)
- [3. Building](#3-building)
- [4. Run](#4-run)

<!-- /TOC -->

# 1. Introduction

Metaverse Network is an EVM-enabled blockchain network for user-created metaverses and games.

Everyone can start their own metaverse for their people with the 3D world, NFTs, play-to-earn & build communities to
earn, and takes community engagement to a new dimension on web3.0.

# 2. Build

Install Rust and Wasm build environment:

```bash
make init
```

Build Wasm and native code:

```bash
make build
```

# 3. Run

### Start a Dev Metaverse Network Chain

Start a dev Metaverse Chain

```bash
make run-dev
```

Detailed logs may be shown by running the node with the following environment variables
set: `RUST_LOG=debug RUST_BACKTRACE=1 cargo run -- --dev`.

This will spin up a Development Metaverse Chain with Alice and Bob as initial authorities.

If you would like to run multi-node manually then you can use the Multi-Node Dev Testnet setup

### Multi-Node Metaverse Network Chain

If you want to see the multi-node consensus algorithm in action locally, then you can create a local testnet with two
validator nodes for Alice and Bob, who are the initial authorities of the genesis chain that have been endowed with
testnet units.

Optionally, give each node a name and expose them so they are listed on the
Polkadot [telemetry site](https://telemetry.polkadot.io/#/Local%20Testnet).

You'll need two terminal windows open.

We'll start Alice's substrate node first on default TCP port 30333 with her chain database stored locally
at `/tmp/alice`. The bootnode ID of her node is `QmRpheLN4JWdAnY7HGJfWFNbfkQCb6tFf4vvA6hgjMZKrR`, which is generated
from the `--node-key` value that we specify below:

```bash
cargo run -- \
  --base-path /tmp/alice \
  --chain=local \
  --alice \
  --node-key 0000000000000000000000000000000000000000000000000000000000000001 \
  --telemetry-url 'wss://telemetry.polkadot.io/submit/ 0' \
  --validator
```

In the second terminal, we'll start Bob's substrate node on a different TCP port of 30334, and with his chain database
stored locally at `/tmp/bob`. We'll specify a value for the `--bootnodes` option that will connect his node to Alice's
bootnode ID on TCP port 30333:

```bash
cargo run -- \
  --base-path /tmp/bob \
  --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/QmRpheLN4JWdAnY7HGJfWFNbfkQCb6tFf4vvA6hgjMZKrR \
  --chain=local \
  --bob \
  --port 30334 \
  --telemetry-url 'wss://telemetry.polkadot.io/submit/ 0' \
  --validator
```
