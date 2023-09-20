#!/usr/bin/env bash

set -e
set -o pipefail

# build integration-test
cargo build --release -p land-cli
cargo build --release -p land-runtime
cargo build --release -p integration-test

# mv binary to top dir
mv target/release/land-cli .
mv target/release/land-runtime .
mv target/release/integration-test .
mv deploy/download-deps-binary.sh .

# package integration-test.tar.gz
tar -czvf integration-test.tar.gz land-cli land-runtime integration-test download-deps-binary.sh