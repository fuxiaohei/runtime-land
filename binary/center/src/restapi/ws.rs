use axum::extract::ws::{Message, WebSocket};
use axum::extract::{connect_info::ConnectInfo, Query, WebSocketUpgrade};
use axum::response::IntoResponse;
use futures_util::stream::StreamExt;
use futures_util::SinkExt;
use serde::Deserialize;
use std::net::SocketAddr;
use tracing::{debug, info, warn};

#[derive(Debug, Deserialize)]
pub struct WsQuery {
    region: String,
    token: String,
}

#[tracing::instrument(name = "[WS]", skip_all)]
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Query(region): Query<WsQuery>,
) -> impl IntoResponse {
    debug!("ws handler,  {:?}", region);
    debug!("{addr} connected.");
    ws.on_upgrade(move |socket| handle_socket(socket, addr))
}

#[tracing::instrument(name = "[WS]", skip(socket))]
async fn handle_socket(mut socket: WebSocket, addr: SocketAddr) {
    debug!("Connected");

    if socket.send(Message::Ping(vec![1, 2, 3])).await.is_ok() {
        debug!("Ping ok");
    } else {
        warn!("Ping failed");
        return;
    }

    let (mut sender, mut receiver) = socket.split();

    let mut send_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
        loop {
            interval.tick().await;
            if sender.send(Message::Ping(vec![1, 2, 3])).await.is_ok() {
                debug!("Ping ok");
            } else {
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
        rv_a = (&mut send_task) => {
            match rv_a {
                Ok(_) => println!("Messages sent"),
                Err(a) => println!("Error sending messages {:?}", a)
            }
            recv_task.abort();
        },
        rv_b = (&mut recv_task) => {
            match rv_b {
                Ok(b) => println!("Received {} messages", b),
                Err(b) => println!("Error receiving messages {:?}", b)
            }
            send_task.abort();
        }
    }

    info!("Disconnected.")
}
