#!/usr/bin/env bash

set -e
set -o pipefail

# cargo install wasm-tools
cargo build -p rust-basic --target wasm32-wasi --release
wasm-tools component new --adapt wasi_snapshot_preview1=moni-runtime/engine/wasi_snapshot_preview1.reactor.wasm  -o tests/data/rust_impl.component.wasm target/wasm32-wasi/release/rust_basic.wasm
