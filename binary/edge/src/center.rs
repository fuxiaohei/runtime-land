use crate::server::RuntimeData;
use futures_util::stream::StreamExt;
use futures_util::SinkExt;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use tracing::{debug, warn};
use tracing::{info, instrument};
use crate::localip;
use std::collections::HashMap;


#[derive(Debug, serde::Serialize)]
struct SyncData {
    pub localip: localip::IpInfo,
    pub region: String,
    pub runtimes: HashMap<String, RuntimeData>,
}

#[instrument(name = "[WS]", skip_all)]
pub async fn init(addr: String, token: String) {
    let ipinfo = crate::localip::IPINFO.get().unwrap();
    let url = format!(
        "ws://{}/v1/region/ws?token={}&region={}",
        addr,
        token,
        ipinfo.region_ip()
    );

    loop {
        debug!("connect to {}", url);

        let ws_stream = match connect_async(&url).await {
            Ok((stream, _response)) => stream,
            Err(e) => {
                warn!("Error during handshake {:?}", e);
                info!("reconnect after 3s");
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
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
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
            loop {
                interval.tick().await;

                let localip = localip::IPINFO.get().unwrap().clone();
                let region = localip.region();
                let runtimes = server::RUNTIMES.lock().unwrap().clone();
                let sync_data = SyncData {
                    localip,
                    region,
                    runtimes,
                };
                let data = serde_json::to_vec(&sync_data).unwrap();

                if !sender.send(Message::Binary(data)).await.is_ok() {
                    warn!("Ping failed");
                    return;
                }
            }
        });

        let mut recv_task = tokio::spawn(async move {
            let mut cnt = 0;
            while let Some(Ok(msg)) = receiver.next().await {
                cnt += 1;
                debug!("recv: {:?}", msg);
            }
            cnt
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

        info!("reconnect after 3s");

        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    }
}
