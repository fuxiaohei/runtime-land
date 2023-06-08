# Runtime.lol

lol-serverless (runtime.lol) is a serverless platform for running webassembly (wasm) functions. It supports multiple languages, and is designed to be easy to use, and easy to deploy. It provides:

- `lol-cli`: a command line interface for deploying and managing functions
- `lol-runtime`: a runner for executing functions
- `lol-server`: a server for managing functions

**This is an experimental project, and is not ready for production use.**

## Quick Start

### Install

TODO

### Develop

TODO

### Deploy

TODO

## Self Hosting

TODO

## Auto Generated Code

use sea-orm to create model:

```bash
sea-orm-cli generate entity -u=mysql://root:@localhost/lol-serverless -o crates/core/src/model
```

use protoc to generate grpc-web code:

```bash
# install protobuf and grpc-web codegen
brew install protobuf
npm install -g protoc-gen-js protoc-gen-grpc-web
# genrate grpc-web code
protoc -I crates/rpc --js_out=import_style=commonjs:web/src/api --grpc-web_out=import_style=commonjs,mode=grpcweb:web/src/api crates/rpc/proto/lol-rpc.proto
```


## Feedback

We'd also love to hear from you. Feel free to open an issue on GitHub with your questions, feedback, or suggestions!
