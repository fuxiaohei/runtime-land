use clap::Parser;
use tracing::{debug, debug_span, warn, Instrument};

mod localip;
mod server;

#[derive(Parser, Debug)]
#[clap(name = "land-edge", version = land_core::version::get())]
struct Cli {
    #[clap(long, env("HTTP_ADDR"), default_value("127.0.0.1:7899"))]
    pub http_addr: String,
    #[clap(long, env("CENTER_ADDR"), default_value("http://127.0.0.1:7777"))]
    pub center_addr: String,
    #[clap(long, env("CENTER_TOKEN"))]
    pub center_token: String,
}

#[derive(Debug, serde::Serialize)]
struct SyncData {
    pub localip: localip::IpInfo,
    pub region: String,
}

async fn sync_interval(center_addr: String, center_token: String) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
    let url = format!("{}/v1/region/sync", center_addr);
    let auth = format!("Bearer {}", center_token);
    loop {
        interval.tick().await;

        let localip = localip::IPINFO.get().unwrap().clone();
        let region = localip.region();
        let sync_data = SyncData { localip, region };

        let client = reqwest::Client::new();
        let res = match client
            .post(&url)
            .json(&sync_data)
            .header(reqwest::header::AUTHORIZATION, auth.clone())
            .send()
            .await
        {
            Ok(res) => res,
            Err(e) => {
                warn!("request error: {}", e.to_string());
                continue;
            }
        };

        // if no auth
        if res.status() == reqwest::StatusCode::UNAUTHORIZED {
            warn!("no auth");
            continue;
        }

        // if not ok
        if !res.status().is_success() {
            warn!("request error: {}", res.status());
            continue;
        }

        // TODO: handle response

        debug!("sync interval response: {:?}", res);
    }
}

#[tokio::main]
async fn main() {
    land_core::trace::init();

    let args = Cli::parse();
    debug!("Load args: {:?}", args);

    localip::init().await.expect("init localip failed");

    // spawn sync internal task
    tokio::spawn(
        sync_interval(args.center_addr, args.center_token).instrument(debug_span!("[SYNC]")),
    );

    server::start(args.http_addr.parse().unwrap())
        .instrument(debug_span!("[SERVER]"))
        .await
        .unwrap();
}
