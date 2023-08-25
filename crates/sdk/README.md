# land-sdk

`land-sdk` provides api for Runtime.land to run faas function project with http trigger.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
anyhow = "1.0.75"
http = "0.2.9"
land-sdk = "0.1.0-rc.2"
wit-bindgen = "0.10.0"

[lib]
crate-type = ["cdylib"] # target wasm32-wasi
```

## Example

```rust
use land_sdk::http::{Body, Request, Response};
use land_sdk::http_main;

#[http_main]
pub fn handle_request(req: Request) -> Response {
    let url = req.uri().clone();
    let method = req.method().to_string().to_uppercase();
    http::Response::builder()
        .status(200)
        .header("X-Request-Url", url.to_string())
        .header("X-Request-Method", method)
        .body(Body::from("Hello Runtime.land!!"))
        .unwrap()
}
```

## License

Apache-2.0
