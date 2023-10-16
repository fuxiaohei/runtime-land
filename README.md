<p align="center"><img src="docs/logo-v2.png" width="120" height="120"/></p>
<h1 align="center">runtime.land</h1>

`runtime.land` is a tiny serverless runtime for [WebAssembly](https://webassembly.org/) modules. It is designed to run small applications and functions written in any language that compiles to WebAssembly in sandboxed environments.

<p align="center">
  <a href="https://github.com/fuxiaohei/runtime-land/actions/workflows/release.yaml">
    <img alt="GitHub Workflow Status" src="https://img.shields.io/github/actions/workflow/status/fuxiaohei/runtime-land/release.yaml?label=Release">
  </a>
  <a href="https://github.com/fuxiaohei/runtime-land/releases/latest">
    <img alt="GitHub release (with filter)" src="https://img.shields.io/github/v/release/fuxiaohei/runtime-land?label=Release">
  </a>
  <a href="https://github.com/fuxiaohei/runtime-land/blob/main/LICENSE">
    <img alt="GitHub" src="https://img.shields.io/github/license/fuxiaohei/runtime-land?color=427ece&label=License">
  </a>
</p>

## Features

- Provides a simple `runtime` as `land-runtime` for WebAssembly modules
- Supports `Rust` and `JavaScript` that compiles to WebAssembly
- Provides a command line tool `land-cli` to manage local projects and deploy to remote `land-runtime` nodes
- Provides a center component `land-center` to manage `land-runtime` nodes and handle requests from `land-cli`.

### Architecture

### Current Status

`runtime.land` is currently in **developing**. It is not recommended to use it in business cases.

- `land-cli` is almost done with basic features: `init`, `build`,`serve`, `deploy`.
- `land-center` is developing and deploying on [zeabur.com](https://zeabur.com).
- `land-runtime` is almost done to run WebAssembly modules. There are _3 runtimes_ deployed on different servers that same as AWS t3.small instance and load-balanced by `Cloudflare`.

### Language Support

`land-runtime` provides a runner to run WebAssembly modules.

| Language | SDK | Status | Features |
| -------- | ------ | ------ | ------ |
| Rust     | [land-sdk](https://crates.io/crates/land-sdk) | ✅ | HTTP Trigger, HTTP Router, Fetch HTTP request |
| JavaScript | [runtime-land-js](https://github.com/fuxiaohei/runtime-land-js) | ✅ | Fetch API with Request/Response <br/> Base64 Encoding <br/> TextEncoding <br/> Web Streams |
| Golang | planning | ❌ | |
| Python | planning | ❌ | |

To get more information about multi-language support, check our [documentation](https://runtime.land/docs/category/languages-guide).

### Usage and Documentation

Usage instructions and documentation for `runtime.land` is available at [https://runtime.land/docs/intro/](https://runtime.land/docs/intro/).

## Development

todo!

## License

`runtime.land` is licensed under the Apache 2.0 License. See [LICENSE](LICENSE) for the full license text.
