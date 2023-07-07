#!/usr/bin/env bash

set -e
set -o pipefail

# cargo install wasm-tools
cargo build -p rust-test --target wasm32-wasi --release
wasm-tools component new --adapt wasi_snapshot_preview1=crates/worker/engine/wasi_snapshot_preview1.reactor.wasm  -o tests/rust_test.component.wasm target/wasm32-wasi/release/rust_test.wasm