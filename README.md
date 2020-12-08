<p align="center">
  <img src="http://uat.bit.country/blackteal.png" width="130">
</p>

<p align="center">  
  <img src="https://raw.githubusercontent.com/w3f/General-Grants-Program/master/src/badge_black.svg" width="530">
</p>

<div align="center">
<h1>Bit.Country</h1>

## A Decentralized World - Create a decentralized bit country of your own.

[![Substrate version](https://img.shields.io/badge/Substrate-2.0.0-brightgreen?logo=Parity%20Substrate)](https://substrate.dev/)
[![Twitter URL](https://img.shields.io/twitter/url?style=social&url=https%3A%2F%2Ftwitter.com%2Fbitdotcountry)](https://twitter.com/bitdotcountry)
[![Medium](https://img.shields.io/badge/Medium-BitCountry-brightgreen?logo=medium)](https://medium.com/@bitcountry)

</div>

Development Note: It is still a WIP.

<!-- TOC -->

- [1. Introduction](#1-introduction)
- [2. Overview](#2-overview)
- [3. Building](#3-building)
- [4. Run](#4-run)
- [5. Development](#5-development)

<!-- /TOC -->


# 1. Introduction
Bit.Country is a decentralized world. The concept is uniquely invented and inspired by the decentralization paradigm.
Its vision is to allow anyone (especially new users to the blockchain) to create their communities, economics as Bit Country on the blockchain network.

Users can create their own countries, blocks, sections, and items as digital assets with NFTs. The UI offers both a classical web view and a 3D in-browser view of the country. The decentralized marketplace allows users to trade their digital assets with each other.

# 2. Overview

Bit.Country provides a new way to socialize with a game feel, while also being driven by real economics.

* A decentralized world - putting community first.
* An open NFT/game protocol for managing & incentivizing small-medium communities using gamification.

# 3. Build

Install Rust:

```bash
curl https://sh.rustup.rs -sSf | sh
```

Initialize your Wasm Build environment:

```bash
./scripts/init.sh
```

Build Wasm and native code:

```bash
cargo build --release
```

# 4. Run

### Single Node Development Chain

Purge any existing developer chain state:

```bash
./target/release/bitcountry-node purge-chain --dev
```

Start a development chain with:

```bash
./target/release/bitcountry-node --dev
```

Detailed logs may be shown by running the node with the following environment variables set: `RUST_LOG=debug RUST_BACKTRACE=1 cargo run -- --dev`.

### Multi-Node Local Testnet

If you want to see the multi-node consensus algorithm in action locally, then you can create a local testnet with two validator nodes for Alice and Bob, who are the initial authorities of the genesis chain that have been endowed with testnet units.

Optionally, give each node a name and expose them so they are listed on the Polkadot [telemetry site](https://telemetry.polkadot.io/#/Local%20Testnet).

You'll need two terminal windows open.

We'll start Alice's substrate node first on default TCP port 30333 with her chain database stored locally at `/tmp/alice`. The bootnode ID of her node is `QmRpheLN4JWdAnY7HGJfWFNbfkQCb6tFf4vvA6hgjMZKrR`, which is generated from the `--node-key` value that we specify below:

```bash
cargo run -- \
  --base-path /tmp/alice \
  --chain=local \
  --alice \
  --node-key 0000000000000000000000000000000000000000000000000000000000000001 \
  --telemetry-url 'wss://telemetry.polkadot.io/submit/ 0' \
  --validator
```

In the second terminal, we'll start Bob's substrate node on a different TCP port of 30334, and with his chain database stored locally at `/tmp/bob`. We'll specify a value for the `--bootnodes` option that will connect his node to Alice's bootnode ID on TCP port 30333:

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
