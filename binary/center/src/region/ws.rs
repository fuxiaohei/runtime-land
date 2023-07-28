use anyhow::Result;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{connect_info::ConnectInfo, Query, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::Response;
use futures_util::stream::StreamExt;
use futures_util::SinkExt;
use land_dao::{user, user_token};
use serde::Deserialize;
use std::net::SocketAddr;
use std::ops::ControlFlow;
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
    Query(query): Query<WsQuery>,
) -> Result<Response, StatusCode> {
    debug!("{addr} connected.");
    let owner_id = match validate_token(query.token).await {
        Ok(id) => id,
        Err(e) => {
            warn!("validate token error: {:?}", e);
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    Ok(ws.on_upgrade(move |socket| handle_socket(socket, query.region, addr, owner_id)))
}

#[tracing::instrument(name = "[WS]", skip(socket))]
async fn handle_socket(mut socket: WebSocket, region: String, addr: SocketAddr, owner_id: i32) {
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
            if process_message(msg, region.clone(), addr, owner_id)
                .await
                .is_break()
            {
                break;
            }
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

async fn process_message(
    msg: Message,
    region: String,
    addr: SocketAddr,
    owner_id: i32,
) -> ControlFlow<(), ()> {
    match msg {
        Message::Text(t) => {
            debug!("recv text: {:?}", t);
        }
        Message::Binary(d) => {
            let mut region_data: super::RegionData = serde_json::from_slice(&d)
                .map_err(|e| {
                    info!("parse region data error: {:?}", e);
                    anyhow::anyhow!("parse region data error: {:?}", e)
                })
                .unwrap();
            region_data.owner_id = owner_id;
            region_data.time_at = chrono::Utc::now().timestamp() as u64;
            super::REGIONS
                .lock()
                .await
                .insert(region.clone(), region_data);
        }
        Message::Close(c) => {
            if let Some(cf) = c {
                info!("recv close: {:?}, addr: {:?}", cf, addr);
            } else {
                info!("recv close, addr: {:?}", addr);
            }
            return ControlFlow::Break(());
        }
        Message::Pong(v) => {
            debug!("recv pong: {:?}", v)
        }
        Message::Ping(v) => {
            debug!("recv ping: {:?}", v)
        }
    }
    ControlFlow::Continue(())
}

// set specific value to show global ownership for token
const REGION_GLOBAL_TOKEN_OWNER_ID: i32 = -1;

async fn validate_token(token: String) -> Result<i32> {
    let (token, user) = user_token::find_by_value_with_active_user(token).await?;
    if token.created_by != user_token::CreatedByCases::Edgehub.to_string() {
        info!("token created by not edgehub");
        anyhow::bail!("token created by not edgehub")
    }
    if user.role == user::Role::Admin.to_string() {
        return Ok(REGION_GLOBAL_TOKEN_OWNER_ID);
    }
    Ok(token.owner_id)
}
