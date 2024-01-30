<div align="center">
<h1>Continuum Node Operator</h1>

## A guide for Node Operators to run Continuum Node

[![Substrate version](https://img.shields.io/badge/Substrate-3.0.0-brightgreen?logo=Parity%20Substrate)](https://substrate.dev/)
[![Twitter URL](https://img.shields.io/twitter/url?style=social&url=https%3A%2F%2Ftwitter.com%2Fbitdotcountry)](https://twitter.com/bitdotcountry)
[![Medium](https://img.shields.io/badge/Medium-Metaverse-brightgreen?logo=medium)](https://medium.com/@metaverse)

</div>

Development Note: It is still a WIP.

<!-- TOC -->

- [1. Running an Archive Node](#1-introduction)
- [2. Overview](#2-overview)
- [3. Building](#3-building)
- [4. Run](#4-run)

<!-- /TOC -->

# 1. Running an Archive Node

An archive node stores the history of past blocks. Most of times, an archive node is used as RPC endpoint. RPC plays a
vital role on our network: it connects users and dApps to the blockchain through WebSocket and HTTP endpoints. Here is a
public RPC endpoint for Continuum RPC Node

[wss://continuum-rpc-1.metaverse.network/wss](wss://continuum-rpc-1.metaverse.network/wss)

**DApp projects** need to run their own RPC node as archive to the retrieve necessary blockchain data and not to rely on
public infrastructure. Public endpoints respond slower because of the large amount of users connected and are rate
limited.

## Requirements

### Machine

- Storage space will increase as the network grows.
- Archive nodes may require a larger server, depending on the amount and frequency of data requested by a dApp.

<Tabs>
<TabItem value="continuum" label="Continuum" default>

| Component | Requirement |
|---|---|
| System | Ubuntu 20.04 |
| CPU | 8 cores |
| Memory | 16 GB |
| Hard Disk | 500 GB SSD (NVMe preferable) |

</TabItem>
</Tabs>

### Ports

The Continuum node runs in parachain configuration, meaning they will listen at different ports by default for both the
parachain and the embedded relay chain.

|Description| Parachain Port | Relaychain Port | Custom Port Flag |
|---|---|---|---|
| P2P | 30333 | 30334 | `--port` |
| RPC | 9944 | 9945 | `--rpc-port` |
| Prometheus | 9615 | 9616 | `--prometheus-port` |

For all types of nodes, ports `30333` and `30334` need to be opened for incoming traffic at the Firewall.
**Collator nodes should not expose WS and RPC ports to the public.**

---

## Installation

There are 2 different ways to run an Continuum node:

Using [Binary](#2-binary) - run the node from binary file and set it up as systemd service

Using [Docker](/docs/build/nodes/archive-node/docker) - run the node within a Docker container

# 2. Building from source

Window users: please use
[this tutorial](https://substrate.dev/docs/en/knowledgebase/getting-started/windows-users)
to setup your build environment

Linux-based machine

**Clone MNet Blockchain code**

Go to [BitCountry team repo](https://github.com/bit-country/Metaverse-Network), clone the repo from correct commit hash.

```git
# clone the code locally

git clone https://github.com/bit-country/Metaverse-Network.git

# change directory

cd Metaverse-Network

# select correct `continuum-release-0.0.2` branch

git checkout continuum-release-0.0.2
```

\*\*Install Make

```bash
sudo apt-get install build-essential
```

**Install Rust**

```bash
curl https://sh.rustup.rs -sSf | sh
```

**Build environment**

````bash
make init
``1

After initializing you can then start building by using the cargo command:

```sh
cargo build --release --features=with-continuum-runtime
````

In case your build fails, please use this command first:

```sh
sudo apt install cmake git clang libclang-dev build-essential
```

Once the build has finished you will have the metaverse-node binary available in the target/release folder. You can
start a node for Continuum Chain from the root of the directory like so:

```sh
./target/release/metaverse-node --chain continuum --bootnodes /ip4/13.239.118.231/tcp/30344/p2p/12D3KooW9rDqyS5S5F6oGHYsmFjSdZdX6HAbTD88rPfxYfoXJdNU --name 'your_node_name' --telemetry-url 'wss://telemetry.polkadot.io/submit/ 0'
```

# 3. Using Docker

We publish the latest version to the
[Docker Hub](https://hub.docker.com/repository/docker/bitcountry/tewai-node/tags?page=1&ordering=last_updated)
that can be pulled and ran locally to connect to the network. In order to do this first make sure that you have Docker
installed locally.
[How to install and run Docker](https://docs.docker.com/engine/install/)

#### Downloading the docker image

```sh
docker pull bitcountry/continuum-collator-node:latest
```

#### Running the docker image

You can test if the docker image is running by using the following command, but the node id and the chain data will be
deleted after you shut down the docker container:

```sh
docker run bitcountry/continuum-collator-node:latest --chain continuum
```

Now, it's time to set up your node to connect to Continuum Bootnode, you need to choose which folder you would like to
store your chain data. Ensure the folder exists and you have write permission for the folder.

Assuming the example path you want to use locally is
_path/to/continuumDb/continuum-node_, the command would be:

```sh
docker run --network=host -v /continuumDb/continuum-node:/bitcountry-db bitcountry/continuum-collator-node:latest -d /bitcountry-db --chain continuum --bootnodes /ip4/13.239.118.231/tcp/30344/p2p/12D3KooW9rDqyS5S5F6oGHYsmFjSdZdX6HAbTD88rPfxYfoXJdNU --telemetry-url 'wss://telemetry.polkadot.io/submit/ 0'
```
