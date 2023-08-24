use crate::{conf::process_conf, conf::CONF_VALUES, localip, server};
use anyhow::Result;
use futures_util::stream::StreamExt;
use futures_util::SinkExt;
use land_core::confdata::{RegionRecvData, RegionReportData};
use once_cell::sync::OnceCell;
use std::ops::ControlFlow;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use tracing::{debug, debug_span, error, warn, Instrument};
use tracing::{info, instrument};

/// CENTER_ADDR is the address of center server
pub static CENTER_ADDR: OnceCell<String> = OnceCell::new();

async fn build_report_data() -> RegionReportData {
    let localip = localip::IPINFO.get().unwrap().clone();
    let region = localip.region();
    let runtimes = server::get_living_runtimes().await;
    let local_conf = CONF_VALUES.lock().await;
    RegionReportData {
        localip,
        region,
        runtimes,
        conf_value_time_version: local_conf.created_at,
        time_at: chrono::Utc::now().timestamp() as u64,
        owner_id: 0,
    }
}

async fn init_ws(ws_url: String) {
    info!("connect to {}", ws_url);

    let reconnect_interval = std::time::Duration::from_secs(5);

    loop {
        let ws_stream = match connect_async(&ws_url).await {
            Ok((stream, _response)) => stream,
            Err(e) => {
                warn!("Error during handshake {:?}", e);
                info!("reconnect after {:?}", reconnect_interval);
                tokio::time::sleep(reconnect_interval).await;
                continue;
            }
        };

        debug!("connected");

        let (mut sender, mut receiver) = ws_stream.split();

        sender
            .send(Message::Ping("Hello".into()))
            .await
            .expect("Can not send!");

        let mut send_task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(2));
            loop {
                interval.tick().await;

                let report_data = build_report_data().await;
                let content = serde_json::to_vec(&report_data).unwrap();

                if sender.send(Message::Binary(content)).await.is_err() {
                    warn!("send report data failed");
                    return;
                }
            }
        });

        let mut recv_task = tokio::spawn(async move {
            while let Some(Ok(msg)) = receiver.next().await {
                if process_message(msg)
                    .instrument(debug_span!("[WS]"))
                    .await
                    .is_break()
                {
                    break;
                }
            }
        });

        tokio::select! {
            _ = (&mut send_task) => {
                debug!("send task done");
                recv_task.abort();
            },
            _ = (&mut recv_task) => {
                debug!("recv task done");
                send_task.abort();
            }
        }

        info!("reconnect after {:?}", reconnect_interval);
        tokio::time::sleep(reconnect_interval).await;
    }
}

#[instrument(name = "[WS]", skip_all)]
pub async fn init(center_url: String, token: String) {
    // set CENTER_ADDR
    let center_addr = if !center_url.starts_with("http") {
        format!("http://{}", center_url)
    } else {
        center_url.clone()
    };
    CENTER_ADDR.set(center_addr.clone()).unwrap();

    // set websocket protocol follow http protocol
    let ws_addr = if center_addr.starts_with("https://") {
        center_addr.replace("https://", "wss://")
    } else {
        center_addr.replace("http://", "ws://")
    };

    // init ws
    let ipinfo = crate::localip::IPINFO.get().unwrap();
    let ws_url = format!(
        "{}/v1/region/ws?token={}&region={}",
        ws_addr,
        token,
        ipinfo.region_ip()
    );
    init_ws(ws_url).await;
}

async fn process_message(msg: Message) -> ControlFlow<(), ()> {
    match msg {
        Message::Text(t) => {
            debug!("recv text: {:?}", t);
        }
        Message::Binary(d) => {
            let recv_data: RegionRecvData = match serde_json::from_slice(&d) {
                Ok(data) => data,
                Err(e) => {
                    error!("parse region data error: {:?}", e);
                    return ControlFlow::Continue(());
                }
            };

            process_conf(recv_data.conf_values).await;
        }
        Message::Close(c) => {
            if let Some(cf) = c {
                info!("recv close: {:?}", cf);
            } else {
                info!("recv close");
            }
            return ControlFlow::Break(());
        }
        Message::Pong(_v) => {
            info!("recv pong")
        }
        Message::Ping(_v) => {
            info!("recv ping")
        }
        Message::Frame(_) => {
            unreachable!("This is never supposed to happen")
        }
    }
    ControlFlow::Continue(())
}

/// request_file request file from center server
pub async fn request_file(download_url: &str) -> Result<reqwest::Response> {
    let resp = reqwest::get(download_url).await?;
    if !resp.status().is_success() {
        return Err(anyhow::anyhow!(
            "request file failed, status: {}, url:{}",
            resp.status(),
            download_url,
        ));
    }
    Ok(resp)
}
