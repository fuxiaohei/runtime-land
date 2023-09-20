#!/usr/bin/env bash

set -e
set -o pipefail

tar -xzvf land-cli-nightly-linux-x64_64.tar.gz

export SDK_VERSION='{ git = "https://github.com/fuxiaohei/runtime-land" }'

LAND_CLI=$(pwd)/land-cli
echo "LAND_CLI=$LAND_CLI"

mkdir -p dist

build_template() {
    local template=$1
    echo "Building template: $template"
    $LAND_CLI init "$template-demo" --template "$template"
    (cd "$template-demo" && $LAND_CLI build)
    local outputname="$template-demo.component.wasm"
    local outputpath="${outputname//-/_}"
    local outputfile="$template-demo/target/wasm32-wasi/release/$outputpath"
    cp "$outputfile" "dist/$template.component.wasm"
    echo "Done: $template, output: dist/$template.component.wasm"
}

build_template "rust-basic"
build_template "rust-fetch"
build_template "rust-router"
build_template "js-basic"

# build templates-wasm.tar.gz from dist
tar -czvf land-cli-nightly-templates-wasm.tar.gz dist