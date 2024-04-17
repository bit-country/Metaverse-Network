#!/usr/bin/env bash

set -e

root_dir=$(git rev-parse --show-toplevel)
bin_dir="$root_dir/bin"
working_dir="$(pwd)"

provider=native
zombienet_version=v1.3.100
pdot_branch=release-polkadot-v1.6.0
asset_hub_branch=release-polkadot-v1.1.0
polkadot_tmp_dir=/tmp/polkadot
asset_hub_tmp_dir=/tmp/asset_hub

arch=$(uname -s 2>/dev/null || echo not)
if [[ $arch == "Darwin" ]]; then
    machine=macos
    elif [[ $arch == "Linux" ]]; then
    machine=linux-x64
fi

PATH=$PATH:$bin_dir

clean() {
    echo "Cleaning bin dir"
    rm -rf "$bin_dir"/*
}

make_bin_dir() {
    echo "Making bin dir"
    mkdir -p "$bin_dir"
}

fetch_zombienet() {
    # Don't fetch zombienet if it's already present in the system
    if which zombienet-$zombienet_version >/dev/null; then
        cp $(which zombienet-$zombienet_version) "$bin_dir/zombienet"
        echo "✅ zombienet-$zombienet_version"
        return
    fi
    
    if [ ! -f "$bin_dir/zombienet" ]; then
        echo "::group::Install zombienet"
        echo "Fetching zombienet..."
        curl -fL -o "$bin_dir/zombienet" "https://github.com/paritytech/zombienet/releases/download/$zombienet_version/zombienet-$machine"
        
        echo "Making zombienet executable"
        chmod +x "$bin_dir/zombienet"
        echo "::endgroup::"
        echo "✅ zombienet-$zombienet_version"
    else
        echo "✅ zombienet-$zombienet_version"
    fi
}

build_polkadot() {
    if [ -f "$bin_dir/polkadot" ]; then
        echo "✅ polkadot-$pdot_branch"
        return
    fi
    
    if [ ! -f "$polkadot_tmp_dir/$pdot_branch/target/release/polkadot" ] || [ ! -f "$polkadot_tmp_dir/$pdot_branch/target/release/polkadot-prepare-worker" ] || [ ! -f "$polkadot_tmp_dir/$pdot_branch/target/release/polkadot-execute-worker" ]; then
        echo "::group::Install polkadot."
        if [ ! -f "$polkadot_tmp_dir/$pdot_branch" ]; then
          echo "Cloning polkadot into $polkadot_tmp_dir"
          mkdir -p "$polkadot_tmp_dir"
          git clone --branch "$pdot_branch" --depth 1 https://github.com/paritytech/polkadot-sdk "$polkadot_tmp_dir/$pdot_branch" || true
        fi
        echo "Building polkadot..."
        cargo build --manifest-path "$polkadot_tmp_dir/$pdot_branch/Cargo.toml" --locked --release --features fast-runtime --bin polkadot --bin polkadot-prepare-worker --bin polkadot-execute-worker
        cp "$polkadot_tmp_dir/$pdot_branch/target/release/polkadot" "$bin_dir/polkadot"
        cp "$polkadot_tmp_dir/$pdot_branch/target/release/polkadot-prepare-worker" "$bin_dir/polkadot-prepare-worker"
        cp "$polkadot_tmp_dir/$pdot_branch/target/release/polkadot-execute-worker" "$bin_dir/polkadot-execute-worker"
        echo "::endgroup::"
        echo "✅ polkadot-$pdot_branch"
    else
        cp "$polkadot_tmp_dir/$pdot_branch/target/release/polkadot" "$bin_dir/polkadot"
        cp "$polkadot_tmp_dir/$pdot_branch/target/release/polkadot-prepare-worker" "$bin_dir/polkadot-prepare-worker"
        cp "$polkadot_tmp_dir/$pdot_branch/target/release/polkadot-execute-worker" "$bin_dir/polkadot-execute-worker"
        echo "✅ polkadot-$pdot_branch"
    fi
}

build_asset_hub() {
    if [ -f "$bin_dir/asset-hub" ]; then
        echo "✅ asset-hub-$pdot_branch"
        return
    fi
    
    if [ ! -f "$asset_hub_tmp_dir/$asset_hub_branch/target/release/polkadot-parachain" ]; then
        echo "::group::Install AssetHub."
        echo "Cloning AssetHub into $asset_hub_tmp_dir"
        mkdir -p "$asset_hub_tmp_dir"
        git clone --branch "$asset_hub_branch" --depth 1 https://github.com/paritytech/polkadot-sdk "$asset_hub_tmp_dir/$asset_hub_branch" || true
        echo "Building AssetHub..."
        cargo build --manifest-path "$asset_hub_tmp_dir/$asset_hub_branch/Cargo.toml" --release --locked --bin polkadot-parachain
        cp "$asset_hub_tmp_dir/$asset_hub_branch/target/release/polkadot-parachain" "$bin_dir/asset-hub"
        echo "::endgroup::"
        echo "✅ asset-hub-$asset_hub_branch"
    else
        cp "$asset_hub_tmp_dir/$asset_hub_branch/target/release/polkadot-parachain" "$bin_dir/asset-hub"
        echo "✅ asset-hub-$asset_hub_branch"
    fi
}

build_meatverse_node() {
    echo "::group::Building Metaverse node.."
    time cargo build --release  --features with-metaverse-runtime
    echo "::endgroup::"
    cp "$root_dir/target/release/metaverse-node" "$bin_dir/metaverse-node"
    
    echo "✅ Metaverse node built"
    cp "$root_dir/target/release/metaverse-node" "$bin_dir/metaverse-node"
    
    echo Current version of Metaverse node:
    "$bin_dir/metaverse-node" -V
}

build_pioneer_node() {
    echo "::group::Building Pioneer node.."
    time cargo build --release  --features with-pioneer-runtime
    echo "::endgroup::"
    cp "$root_dir/target/release/metaverse-node" "$bin_dir/pioneer-node"
    echo "✅ Pioneer node built"
    cp "$root_dir/target/release/metaverse-node" "$bin_dir/pioneer-node"
    
    echo Current version of Pioneer node:
    "$bin_dir/pioneer-node" -V
}

setup_basic() {
    make_bin_dir
    fetch_zombienet
    build_polkadot
    build_asset_hub
}

setup_metaverse() {
    make_bin_dir
    fetch_zombienet
    build_polkadot
    build_asset_hub
    build_meatverse_node
}

setup_pioneer() {
    make_bin_dir
    fetch_zombienet
    build_polkadot
    build_asset_hub
    build_pioneer_node
}

spawn_basic() {
    setup_basic
    echo "Spawning zombienet using provider: $provider..."
    zombienet --provider="$provider" spawn $root_dir/scripts/zombienet/basic-config.toml
}

spawn_metaverse() {
    setup_metaverse
    echo "Spawning zombienet using provider: $provider..."
    zombienet --provider="$provider" spawn $root_dir/scripts/zombienet/mnet-metaverse.toml
}

spawn_pioneer() {
    setup_pioneer
    echo "Spawning zombienet using provider: $provider..."
    zombienet --provider="$provider" spawn $root_dir/scripts/zombienet/mnet-pioneer.toml
}

case "$1" in
    "setup_basic")
        setup_basic
    ;;
    "setup_metaverse")
        setup_metaverse
    ;;
    "setup_pioneer")
        setup_pioneer
    ;;
    "spawn_basic")
        spawn_basic
    ;;
    "spawn_metaverse")
        spawn_metaverse
    ;;
    "spawn_pioneer")
        spawn_pioneer
    ;;
    *)
        echo "Enter an appropriate command"
        exit 1
    ;;
esac
