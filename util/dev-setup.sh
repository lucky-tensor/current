#!/bin/bash

# Ubuntu
apt-get update

apt install -y build-essential lld pkg-config libssl-dev libclang-dev libpq-dev

# curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

cargo install sccache

echo [build]$'\n'rustc-wrapper = $(which sccache) > $HOME/.cargo/config.toml 
