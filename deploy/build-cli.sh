#!/usr/bin/env bash

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

get_arch() {
    local arch=$(uname -m)
    case $arch in
        amd64) arch="x86_64" ;;
        x86_64) arch="x86_64" ;;
        aarch64) arch="aarch64" ;;
        riscv64) arch="riscv64" ;;
        arm64) arch="aarch64" ;; # This is for the macOS M1 ARM chips
        *)
            log_print 2 "The system architecture (${arch}) is not yet supported by this script."
            exit 1
        ;;
    esac
    echo "$arch"
}

get_os() {
    OS=$(uname | tr '[:upper:]' '[:lower:]')
    case "$OS" in
        darwin) OS='macos' ;;
        linux) OS='linux' ;;
        freebsd) OS='freebsd' ;;
        # mingw*) OS='windows';;
        # msys*) OS='windows';;
        *)
            log_print 2 "The OS (${OS}) is not supported by this script."
            exit 1
        ;;
    esac
}

get_os

# ARCH if not set, use get_arch
ARCH=$1
ARCH=${ARCH:-$(get_arch)}

# VERSION if not set, use "nightly"
VERSION=$2
VERSION=${VERSION:-nightly}

# TARGET_DIR if not set, use "target/release"
TARGET_DIR=$3
TARGET_DIR=${TARGET_DIR:-target/release}

# EXTRA_ARGS
EXTRA_ARGS=$4

log_print 0 "Checking for system os and architecture..."

log_print 1 "VERSION=$VERSION, TARGET_DIR=$TARGET_DIR, ARCH=$ARCH, OS=$OS, EXTRA_ARGS=$EXTRA_ARGS"

set -e
set -o pipefail

# Function used to check if utilities are available
require() {
    if ! hash "$1" &>/dev/null; then
        log_print 2 "'$1' not found in PATH. This is required for this script to work."
        exit 1
    fi
}

require curl
require tar
require uname

./deploy/download-wasm-deps.sh

log_print 0 "Build land-cli..."
cargo build -p land-cli --release $EXTRA_ARGS

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

# if os=windows, use 7z to package zip
if [[ "$OS" == "windows" ]]; then
    log_print 1 "Packaging zip"
    7z a -tzip land-cli-$VERSION-$OS-$ARCH.zip LICENSE README.md wizer-bin wasm-opt-bin land-cli$EXT
    log_print 1 "Done: land-cli-$VERSION-$OS-$ARCH.zip"
    exit 0
fi
log_print 1 "Packaging tar.gz"
tar cvzf land-cli-$VERSION-$OS-$ARCH.tar.gz LICENSE README.md wizer-bin wasm-opt-bin land-cli$EXT
log_print 1 "Done: land-cli-$VERSION-$OS-$ARCH.tar.gz"
