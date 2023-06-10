# Web

## Generate Grpc Web

pre-requisite: install protoc and grpc-web

```bash
brew install protobuf
npm install -g protoc-gen-js protoc-gen-grpc-web
```

genrate grpc-web code:

```bash
cd land-serverless
protoc -I crates/rpc --js_out=import_style=commonjs:web/src/api --grpc-web_out=import_style=commonjs,mode=grpcweb:web/src/api crates/rpc/proto/land-rpc.proto
```
