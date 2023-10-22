# land-sdk

`land-sdk` provides api for Runtime.land to run faas function project with http trigger.

## Usage

Add this to your `Cargo.toml`:

```toml
[package]
name = "rust-hello-world"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = "1.0.75"
http = "0.2.9"
land-sdk = "0.1.4"
wit-bindgen = "0.13.0"

[lib]
crate-type = ["cdylib"] # target wasm32-wasi
```

## Example

```rust
use land_sdk::http::{Body, Error, Request, Response};
use land_sdk::http_main;

#[http_main]
pub fn handle_request(req: Request) -> Result<Response, Error> {
    let url = req.uri().clone();
    let method = req.method().to_string().to_uppercase();
    Ok(http::Response::builder()
        .status(200)
        .header("X-Request-Url", url.to_string())
        .header("X-Request-Method", method)
        .body(Body::from("Hello Runtime.land!!"))
        .unwrap())
}
```

## License

Apache-2.0
