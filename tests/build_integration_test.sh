#!/usr/bin/env bash

set -e
set -o pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m' # No Color

# Print in colors - 1=green, 2=red, other=neutral
# e.g. log_print 0 "All is great"
log_print() {
    if [[ $1 == 1 ]]; then
        echo -e "${GREEN}${2}${NC}"
        elif [[ $1 == 2 ]]; then
        echo -e "${RED}${2}${NC}"
    else
        echo -e "${2}"
    fi
}

# build integration-test
log_print 1 "Building land-cli"
cargo build --release -p land-cli
log_print 1 "Building land-runtime"
cargo build --release -p land-runtime
log_print 1 "Building integration-test"
cargo build --release -p integration-test

# cp binary to top dir
cp target/release/land-cli .
cp target/release/land-runtime .
cp target/release/integration-test .
cp deploy/download-deps-binary.sh .

# package integration-test.tar.gz
log_print 1 "Packaging integration-test.tar.gz"
tar -czvf integration-test.tar.gz land-cli land-runtime integration-test download-deps-binary.sh

rm land-cli land-runtime integration-test download-deps-binary.sh