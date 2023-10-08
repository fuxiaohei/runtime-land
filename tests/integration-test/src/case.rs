use anyhow::Result;
use reqwest::Body;
use serde::{Deserialize, Serialize};
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
    assert_eq!(headers.get("x-served-by").unwrap(), "land-runtime");
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

pub async fn test_js_basic(wasm: &str) -> Result<()> {
    let client = reqwest::Client::new();
    let resp = client
        .get(RUMTIME_SERVER)
        .header("x-land-module", wasm)
        .send()
        .await?;
    info!("resp: {:?}", resp);
    assert_eq!(resp.status(), 200);
    let body = resp.text().await.unwrap();
    assert!(body.contains("Runtime.land JS SDK"));
    Ok(())
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct JsFetchResponse {
    pub status: String,
    pub country: String,
    pub country_code: String,
    pub region: String,
    pub region_name: String,
    pub city: String,
    pub zip: String,
    pub lat: f64,
    pub lon: f64,
    pub timezone: String,
    pub isp: String,
    pub org: String,
    #[serde(rename = "as")]
    pub as_field: String,
    pub query: String,
}

pub async fn test_js_fetch(wasm: &str) -> Result<()> {
    let client = reqwest::Client::new();
    let resp = client
        .get(RUMTIME_SERVER)
        .header("x-land-module", wasm)
        .send()
        .await?;
    info!("resp: {:?}", resp);
    assert_eq!(resp.status(), 200);
    let body: JsFetchResponse = resp.json().await.unwrap();
    assert!(body.status.contains("success"));
    Ok(())
}

pub async fn test_js_itty_router(wasm: &str) -> Result<()> {
    let proxy = reqwest::Proxy::http(RUMTIME_SERVER)?;
    let client = reqwest::Client::builder().proxy(proxy).build()?;
    let resp = client
        .get("http://example.com/hello")
        .header("x-land-module", wasm)
        .send()
        .await?;
    info!("resp: {:?}", resp);
    assert_eq!(resp.status(), 200);
    let body = resp.text().await?;
    assert_eq!(body, "Hello, World");

    let resp = client
        .post("http://example.com/foo/bar")
        .body(Body::from("Foo Bar"))
        .header("x-land-module", wasm)
        .send()
        .await?;
    info!("resp: {:?}", resp);
    assert_eq!(resp.status(), 200);
    let body = resp.text().await?;
    assert_eq!(body, "Body: Foo Bar");

    let resp = client
        .get("http://example.com/params/xyz")
        .header("x-land-module", wasm)
        .send()
        .await?;
    info!("resp: {:?}", resp);
    assert_eq!(resp.status(), 200);
    let body = resp.text().await?;
    assert_eq!(body, "value: xyz");

    Ok(())
}
