.PHONY: build, build-server, build-runtime, build-cli, build-web, test-wit-v2

test-wit-v2:
	@cargo build --release --target wasm32-wasi -p wit-v2-guest
	@cargo run -p wit-v2-host
	@cargo run -p wit-v2-gen

build-server:
	@echo "Building server..."
	@cargo build --release

build-runtime:
	@echo "Building runtime..."
	@cargo build --release -p lol-runtime

build-cli:
	@echo "Building CLI..."
	@cargo build --release -p lol-cli

build-web:
	@echo "Building web..."
	@cd web && npm install && npm run build

build: build-server build-runtime build-cli build-web

