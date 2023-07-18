# runtime.land

`runtime.land` is a tiny serverless runtime for [WebAssembly](https://webassembly.org/) modules. It is designed to run small applications and functions written in any language that compiles to WebAssembly in sandboxed environments.

## Build local image

```bash
docker build -t land-runtime:v0.1.0-b3 -f deploy/runtime.Dockerfile .
```

## Start Docker Compose

start region node with `land-edge` and two `land-runtime` instances (mock as load-balanced runtime nodes).

```bash
docker compose -f deploy/docker-compose/region.yaml up
```
