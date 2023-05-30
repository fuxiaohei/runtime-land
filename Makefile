.PHONY: build, build-core, build-runtime, build-cli, build-web, test-wit-v2

test-wit-v2:
	@cargo build --release --target wasm32-wasi -p wit-v2-guest
	@cargo run -p wit-v2-host
	@cargo run -p wit-v2-gen

build-core:
	@echo "Building moni-serverless..."
	@cargo build --release

build-runtime:
	@echo "Building moni-serverless runtime..."
	@cargo build --release -p moni-runtime

build-cli:
	@echo "Building moni-serverless CLI..."
	@cargo build --release -p moni-cli

build-web:
	@echo "Building moni-serverless web..."
	@cd moni-web && npm install && npm run build

build: build-core build-runtime build-cli build-web

