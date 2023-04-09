#!/usr/bin/env bash

set -e
set -o pipefail

cargo build -p rust-impl --target wasm32-wasi
wasm-tools component new --adapt wasi_snapshot_preview1=moni-runtime/engine/wasi_snapshot_preview1.reactor.wasm  -o tests/data/rust_impl.component.wasm target/wasm32-wasi/debug/rust_impl.wasm
