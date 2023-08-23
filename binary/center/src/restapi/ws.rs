use crate::region::conf::CONF_VALUES;
use crate::region::REGIONS;
use anyhow::Result;
use axum::{
    extract::{
        connect_info::ConnectInfo,
        ws::{Message, WebSocket},
        Query, WebSocketUpgrade,
    },
    http::StatusCode,
    response::Response,
};
use futures_util::stream::StreamExt;
use futures_util::SinkExt;
use land_core::confdata::RegionReportData;
use land_dao::{user, user_token};
use once_cell::sync::OnceCell;
use serde::Deserialize;
use std::net::SocketAddr;
use std::ops::ControlFlow;
use tokio::sync::watch;
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

    let (tx, mut rx) = watch::channel(0);

    let (mut sender, mut receiver) = socket.split();

    let mut send_task = tokio::spawn(async move {
        while rx.changed().await.is_ok() {
            let send_data = crate::region::build_recv_data().await.unwrap();
            let bytes = serde_json::to_vec(&send_data).unwrap();
            if sender.send(Message::Binary(bytes)).await.is_ok() {
                debug!("Send values ok");
            } else {
                warn!("Send failed");
                break;
            }
        }
    });

    let mut recv_task = tokio::spawn(async move {
        let mut cnt = 0;
        while let Some(Ok(msg)) = receiver.next().await {
            cnt += 1;
            let (trigger_send, ops) = process_message(msg, region.clone(), addr, owner_id)
                .await
                .unwrap();
            if ops.is_break() {
                break;
            }
            if trigger_send {
                let ts = chrono::Utc::now().timestamp() as u64;
                if tx.send(ts).is_err() {
                    warn!("send error");
                    break;
                }
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
                Ok(b) => debug!("Received {} messages", b),
                Err(b) => warn!("Error receiving messages {:?}", b)
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
) -> Result<(bool, ControlFlow<(), ()>)> {
    let mut trigger_send = false;
    match msg {
        Message::Text(t) => {
            debug!("recv text: {:?}", t);
        }
        Message::Binary(d) => {
            let mut region_data: RegionReportData = serde_json::from_slice(&d)
                .map_err(|e| {
                    info!("parse region data error: {:?}", e);
                    anyhow::anyhow!("parse region data error: {:?}", e)
                })
                .unwrap();

            let conf_time_version = region_data.conf_value_time_version;

            region_data.owner_id = owner_id;
            region_data.time_at = chrono::Utc::now().timestamp() as u64;
            REGIONS.lock().await.insert(region.clone(), region_data);

            let conf_value = CONF_VALUES.lock().await;
            if conf_value.created_at > conf_time_version {
                trigger_send = true;
            }
        }
        Message::Close(c) => {
            if let Some(cf) = c {
                info!("recv close: {:?}, addr: {:?}", cf, addr);
            } else {
                info!("recv close, addr: {:?}", addr);
            }
            return Ok((false, ControlFlow::Break(())));
        }
        Message::Pong(v) => {
            debug!("recv pong: {:?}", v)
        }
        Message::Ping(v) => {
            debug!("recv ping: {:?}", v)
        }
    }
    Ok((trigger_send, ControlFlow::Continue(())))
}

/// REGION_TOKEN is token for region
static REGION_TOKEN: OnceCell<String> = OnceCell::new();

// REGION_GLOBAL_TOKEN_OWNER_ID is owner id of admin user token
const REGION_GLOBAL_TOKEN_OWNER_ID: i32 = -1;

// REGION_GLOBAL_TOKEN_ENV_ID is global token owner id for env token
const REGION_GLOBAL_TOKEN_ENV_ID: i32 = -2;

async fn validate_token(token: String) -> Result<i32> {
    let region_token = REGION_TOKEN.get_or_init(|| {
        std::env::var("REGION_TOKEN")
            .unwrap_or_else(|_| "".to_string())
            .to_string()
    });
    if region_token.eq(&token) {
        info!("token use env region token");
        return Ok(REGION_GLOBAL_TOKEN_ENV_ID);
    }
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
