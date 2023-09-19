#!/usr/bin/env bash

set -e
set -o pipefail

ARCH=$1
OS=$2

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

require curl
require tar

log_print 1 "ARCH=$ARCH, OS=$OS"

download_wizer_binary() {
    log_print 0 "Downloading wizer"
    
    # if os==windows, wizer_ext is tar.xz, otherwize tar.gz
    local wizer_ext="tar.xz"
    if [[ "$OS" == "windows" ]]; then
        wizer_ext="zip"
    fi
    
    local filebase="wizer-v3.0.1-$ARCH-$OS"
    local filename="$filebase.$wizer_ext"
    local archive_url="https://github.com/bytecodealliance/wizer/releases/download/v3.0.1/$filename"
    log_print 1 "Downloading wizer: $archive_url"
    curl --progress-bar --show-error --location --fail $archive_url --output "wizer.$wizer_ext"
    
    # if wizer_ext is tar.xz, uncompress with tar -xf
    if [[ "$wizer_ext" == "tar.xz" ]]; then
        tar -xf "wizer.$wizer_ext"
    else
        unzip "wizer.$wizer_ext"
    fi
    rm -rf wizer-bin
    mv $filebase wizer-bin
}

download_wizer_binary

download_wasm_opt_binary() {
    log_print 0 "Downloading wasm-opt"
    local filename="binaryen-version_116-wasm-opt-$ARCH-$OS.tar.gz"
    local archive_url="https://runtime.land/$filename"
    log_print 1 "Downloading wasm-opt: $archive_url"
    curl --progress-bar --show-error --location --fail $archive_url --output "wasm-opt.tar.gz"
    mkdir -p wasm-opt-bin
    tar -xzf wasm-opt.tar.gz -C wasm-opt-bin
}

download_wasm_opt_binary