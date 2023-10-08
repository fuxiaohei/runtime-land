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

download_wasi_snapshot_preview1_reactor_wasm() {
    log_print 0 "Downloading wasi_snapshot_preview1.reactor.wasm"
    local targetfile="crates/worker/engine/wasi_snapshot_preview1.reactor.wasm"
    if [[ -f "$targetfile" ]]; then
        log_print 1 "wasi_snapshot_preview1.reactor.wasm already exists, skip downloading"
        return
    fi
    local filename="wasi_snapshot_preview1.reactor.wasm"
    local archive_url="https://github.com/bytecodealliance/wasmtime/releases/download/v13.0.0/wasi_snapshot_preview1.reactor.wasm"
    log_print 1 "Downloading wasi_snapshot_preview1.reactor.wasm: $archive_url"
    curl --progress-bar --show-error --location --fail $archive_url --output "$filename"
    mv "$filename" "$targetfile"
}

download_wasi_snapshot_preview1_reactor_wasm

download_js_sdk_wasm() {
    log_print 0 "Downloading js-sdk.wasm"
    local targetfile="crates/worker/engine/land-js-sdk.wasm"
    if [[ -f "$targetfile" ]]; then
        log_print 1 "js-sdk.wasm already exists, skip downloading"
        return
    fi
    local filename="land-js-sdk.wasm"
    local archive_url="https://github.com/fuxiaohei/runtime-land-js/releases/download/nightly/land-js-sdk-nightly.wasm"
    log_print 1 "Downloading js-sdk.wasm: $archive_url"
    curl --progress-bar --show-error --location --fail $archive_url --output "$filename"
    mv "$filename" "$targetfile"
}

download_js_sdk_wasm
