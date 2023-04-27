# Moni-Web

## Generate Grpc Web

pre-requisite: install protoc and grpc-web

```bash
brew install protobuf
npm install -g protoc-gen-js protoc-gen-grpc-web
```

genrate grpc-web code:

```bash
cd moni-serverless
protoc -I moni-lib/rpc --js_out=import_style=commonjs:moni-web/src/api --grpc-web_out=import_style=commonjs,mode=grpcweb:moni-web/src/api moni-lib/rpc/proto/moni-rpc.proto
```
