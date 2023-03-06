#!/usr/bin/env bash

set -e
set -o pipefail

cmd="target/release/moni-serverless"
echo -e "build runner:"
cargo build --release

cli="target/release/moni-cli"
echo -e "build cli:"
cargo build -p moni-cli --release

echo -e "rust-basic:"
cargo build -p rust-basic --target wasm32-wasi --release && $cmd rust-basic

echo -e "rust-fetch:"
cargo build -p rust-fetch --target wasm32-wasi --release && $cmd rust-fetch
