.PHONY: build, build-core, build-runtime, build-cli, build-web

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

