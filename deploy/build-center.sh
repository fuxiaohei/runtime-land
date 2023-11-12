#!/usr/bin/env bash

set -e pipefail

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
require uname

log_print 0 "Checking for system os and architecture..."

get_arch() {
    ARCH=$(uname -m)
    case $ARCH in
        amd64) ARCH="x86_64" ;;
        x86_64) ARCH="x86_64" ;;
        aarch64) ARCH="aarch64" ;;
        riscv64) ARCH="riscv64" ;;
        arm64) ARCH="aarch64" ;; # This is for the macOS M1 ARM chips
        *)
            log_print 2 "The system architecture (${ARCH}) is not yet supported by this installation script."
            exit 1
        ;;
    esac
    # echo "ARCH = $ARCH"
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
            log_print 2 "The OS (${OS}) is not supported by this installation script."
            exit 1
        ;;
    esac
}

get_arch
get_os

log_print 0 "OS = $OS, ARCH = $ARCH"

rustup component add rustfmt
rustup target add wasm32-wasi

./deploy/download-wasm-deps.sh

build_tailwindcss() {
    log_print 0 "Downloading tailwindcss cli"
    local ext=""
    if [[ "$OS" == "windows" ]]; then
        ext=".exe"
    fi
    local arch_name="$ARCH"
    # convert aarch64 to arm64
    if [[ "$arch_name" == "aarch64" ]]; then
        arch_name="arm64"
    fi
    # convert x86_64 to x64
    if [[ "$arch_name" == "x86_64" ]]; then
        arch_name="x64"
    fi
    local binaryname="tailwindcss$ext"
    local downloadurl="https://github.com/tailwindlabs/tailwindcss/releases/download/v3.3.5/tailwindcss-$OS-$arch_name"
    log_print 1 "Downloading tailwindcss cli: $downloadurl"
    curl --progress-bar --show-error --location --fail $downloadurl --output "$binaryname"
    chmod +x "$binaryname"
    
    log_print 0 "Build templates css"
    ./"$binaryname" -c ./binary/center/templates/tailwind.config.js  -i ./binary/center/templates/css/input.css -o ./binary/center/templates/css/main.css --minify
}

build_tailwindcss

log_print 0 "Build land-center..."
cargo build --release -p land-center

log_print 0 "Build land-cli..."
cargo build --release -p land-cli

log_print 0 "Create temp dir to build templates wasm"
TEMPDIR=$(mktemp -d)
WORKDIR=$(pwd)

log_print 0 "TEMPDIR=$TEMPDIR, WORKDIR=$WORKDIR"

cp target/release/land-cli $TEMPDIR/land-cli
cd $TEMPDIR

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

export SDK_VERSION='{ git = "https://github.com/fuxiaohei/runtime-land" }'

LAND_CLI=$(pwd)/land-cli
echo "LAND_CLI=$LAND_CLI"

mkdir -p templates-wasm

build_template() {
    local template=$1
    echo "Building template: $template"
    $LAND_CLI init "$template-demo" --template "$template"
    (cd "$template-demo" && $LAND_CLI build)
    local outputname="$template-demo.component.wasm"
    local outputpath="${outputname//-/_}"
    local outputfile="$template-demo/target/wasm32-wasi/release/$outputpath"
    cp "$outputfile" "templates-wasm/$template.component.wasm"
    echo "Done: $template, output: templates-wasm/$template.component.wasm"
}

build_template "rust-hello-world"
build_template "rust-fetch"
build_template "rust-router"
build_template "js-hello-world"
build_template "js-fetch"
build_template "js-itty-router"

# build templates-wasm.tar.gz from dist
tar -czvf templates-wasm.tar.gz templates-wasm
cp templates-wasm.tar.gz $WORKDIR
