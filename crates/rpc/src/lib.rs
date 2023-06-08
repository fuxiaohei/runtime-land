use crate::rpc_service_server::RpcServiceServer;
use http::HeaderName;
use lol_core::dao;
use lol_core::model::user_token;
use std::net::SocketAddr;
use std::time::Duration;
use tonic::{Request, Status};
use tonic_web::GrpcWebLayer;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tracing::info;
use tracing::log::warn;

pub mod client;
mod server;

tonic::include_proto!("lol");

const DEFAULT_MAX_AGE: Duration = Duration::from_secs(24 * 60 * 60);
const DEFAULT_EXPOSED_HEADERS: [&str; 3] =
    ["grpc-status", "grpc-message", "grpc-status-details-bin"];
const DEFAULT_ALLOW_HEADERS: [&str; 6] = [
    "x-grpc-web",
    "content-type",
    "x-user-agent",
    "grpc-timeout",
    "authorization",
    "x-grpc-method",
];

pub(crate) struct UserContext {
    token: String,
}

impl UserContext {
    pub async fn get_token(&self) -> Result<user_token::Model, tonic::Status> {
        let token = dao::token::find(self.token.clone())
            .await
            .map_err(|e| tonic::Status::internal(format!("{:?}", e)))?;
        if token.is_none() {
            return Err(tonic::Status::unauthenticated("invalid auth token"));
        }
        Ok(token.unwrap())
    }
}

fn auth_intercept(mut req: Request<()>) -> Result<Request<()>, Status> {
    let grpc_method = match req.metadata().get("x-grpc-method") {
        Some(grpc_method) => grpc_method.to_str().unwrap(),
        None => {
            return Err(Status::unauthenticated("no grpc method"));
        }
    };

    // if grpc_method is signupEmail or loginEmail or LoginAccessToken, no need to check auth token
    if grpc_method == "loginEmail"
        || grpc_method == "loginAccessToken"
        || grpc_method == "signupEmail"
    {
        return Ok(req);
    }

    let auth_token = req.metadata().get("authorization");
    if auth_token.is_none() {
        warn!("no auth token, grpc_method:{}", grpc_method);
        return Err(Status::unauthenticated("no auth token"));
    }
    let auth_token = auth_token.unwrap().to_str().unwrap();
    let auth_token = auth_token.replace("Bearer ", "");
    let auth_token = auth_token.trim();
    req.extensions_mut().insert(UserContext {
        token: auth_token.to_string(),
    });

    Ok(req)
}

pub async fn start_server(
    addr: SocketAddr,
    is_grpc_web: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let rpc_impl = server::ServiceImpl::default();
    let svc = RpcServiceServer::with_interceptor(rpc_impl, auth_intercept);
    info!("RpcServer listening on {addr}");
    if is_grpc_web {
        let cors_layer = CorsLayer::new()
            .allow_origin(AllowOrigin::mirror_request())
            .allow_credentials(true)
            .max_age(DEFAULT_MAX_AGE)
            .expose_headers(
                DEFAULT_EXPOSED_HEADERS
                    .iter()
                    .cloned()
                    .map(HeaderName::from_static)
                    .collect::<Vec<HeaderName>>(),
            )
            .allow_headers(
                DEFAULT_ALLOW_HEADERS
                    .iter()
                    .cloned()
                    .map(HeaderName::from_static)
                    .collect::<Vec<HeaderName>>(),
            );

        info!("GRPC-Web is enabled");
        tonic::transport::Server::builder()
            .accept_http1(true)
            .layer(cors_layer)
            .layer(GrpcWebLayer::new())
            .add_service(svc)
            .serve(addr)
            .await?;
        return Ok(());
    }
    tonic::transport::Server::builder()
        .add_service(svc)
        .serve(addr)
        .await?;

    Ok(())
}
