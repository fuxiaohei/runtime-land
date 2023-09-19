#!/usr/bin/env bash

set -e
set -o pipefail

ARCH=$1
OS=$2
VERSION=$3
TARGET_DIR=$4

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

# Function used to check if utilities are available
require() {
    if ! hash "$1" &>/dev/null; then
        log_print 2 "'$1' not found in PATH. This is required for this script to work."
        exit 1
    fi
}

log_print 0 "ARCH=$ARCH, OS=$OS, VERSION=$VERSION, TARGET_DIR=$TARGET_DIR"

EXT=""
if [[ "$OS" == "windows" ]]; then
    EXT=".exe"
fi

mkdir -p _dist
log_print 1 "Target: $TARGET_DIR/land-cli$EXT"
log_print 1 "Copying files to _dist"
cp LICENSE README.md $TARGET_DIR/land-cli$EXT _dist/
cp -R wizer-bin _dist/
cp -R wasm-opt-bin _dist/
cd _dist
tar cvzf land-cli-$VERSION-$OS-$ARCH.tar.gz LICENSE README.md wizer-bin wasm-opt-bin land-cli$EXT
log_print 1 "Done: land-cli-$VERSION-$OS-$ARCH.tar.gz"