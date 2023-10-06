#!/usr/bin/env sh
set -e

mkdir -p "dist/docs:calculator"

(cd calculator && cargo component build --release)
(cd adder && cargo component build --release)
(cd spin-app && cargo component build --release)

cp "target/wasm32-wasi/release/calculator.wasm" "dist"
cp "target/wasm32-wasi/release/adder.wasm" "dist/docs:calculator/"

RUST_LOG=none wasm-tools compose -c config.yml -o dist/spin_app.wasm target/wasm32-wasi/release/spin_app.wasm