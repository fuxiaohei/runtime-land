use anyhow::Result;
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
