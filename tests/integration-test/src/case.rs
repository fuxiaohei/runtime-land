use anyhow::Result;
use reqwest::Body;
use tracing::info;

static RUMTIME_SERVER: &str = "http://127.0.0.1:7909";

pub async fn test_rust_basic(wasm: &str) -> Result<()> {
    let client = reqwest::Client::new();
    let resp = client
        .get(RUMTIME_SERVER)
        .header("x-land-module", wasm)
        .send()
        .await?;
    info!("resp: {:?}", resp);
    assert_eq!(resp.status(), 200);
    let headers = resp.headers();
    assert_eq!(headers.get("x-request-url").unwrap(), "/");
    assert_eq!(headers.get("x-request-method").unwrap(), "GET");
    assert_eq!(headers.get("x-served-by").unwrap(), "land-edge");
    Ok(())
}

pub async fn test_rust_fetch(wasm: &str) -> Result<()> {
    let client = reqwest::Client::new();
    let resp = client
        .get(RUMTIME_SERVER)
        .header("x-land-module", wasm)
        .send()
        .await?;
    info!("resp: {:?}", resp);
    assert_eq!(resp.status(), 200);
    let body = resp.text().await?;
    assert!(body.contains("Rust Programming Language"));
    Ok(())
}

pub async fn test_rust_router(wasm: &str) -> Result<()> {
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}/hello", RUMTIME_SERVER))
        .header("x-land-module", wasm)
        .send()
        .await?;
    info!("resp: {:?}", resp);
    assert_eq!(resp.status(), 200);
    let body = resp.text().await?;
    assert_eq!(body, "Hello, World");

    let resp = client
        .post(format!("{}/foo/bar", RUMTIME_SERVER))
        .body(Body::from("Foo Bar"))
        .header("x-land-module", wasm)
        .send()
        .await?;
    info!("resp: {:?}", resp);
    assert_eq!(resp.status(), 200);
    let body = resp.text().await?;
    assert_eq!(body, "Foo Bar, BodySize: 7");

    let resp = client
        .get(format!("{}/params/xyz", RUMTIME_SERVER))
        .header("x-land-module", wasm)
        .send()
        .await?;
    info!("resp: {:?}", resp);
    assert_eq!(resp.status(), 200);
    let body = resp.text().await?;
    assert_eq!(body, "value: xyz");

    Ok(())
}
