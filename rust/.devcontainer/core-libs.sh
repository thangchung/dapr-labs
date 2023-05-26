#!/usr/bin/env bash
set -euo pipefail

apt-get update && apt-get install -y \
    software-properties-common \
    bash \
    git \
    curl \
    npm \
    build-essential libssl-dev pkg-config glibc-source \
    ca-certificates

# Install Rust
curl https://sh.rustup.rs -sSf | bash -s -- -y
PATH="/root/.cargo/bin:${PATH}"
# RUN rustup target add wasm32-wasi
cargo install sea-orm-cli
