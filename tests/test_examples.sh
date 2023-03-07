#!/usr/bin/env bash

set -e
set -o pipefail

pwd=$(pwd)

cmd="$pwd/target/release/moni-serverless"
echo -e "build runner:"
cargo build --release

cli="$pwd/target/release/moni-cli"
echo -e "build cli:"
cargo build -p moni-cli --release

echo -e "rust-basic:"
(cd examples/rust-basic && $cli build) && $cmd rust-basic

echo -e "rust-fetch:"
(cd examples/rust-fetch && $cli build) && $cmd rust-fetch

echo -e "rust-router:"
(cd examples/rust-router && $cli build) 
$cmd rust-router --url=/hello
$cmd rust-router --url=/foo/bar
$cmd rust-router --url=/params/666

echo -e "rust-kv:"
(cd examples/rust-kv && $cli build) && $cmd rust-kv
