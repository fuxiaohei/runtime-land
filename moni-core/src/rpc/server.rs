use super::moni_rpc_service_server::{MoniRpcService, MoniRpcServiceServer};
use crate::dao;
use std::net::SocketAddr;
use tracing::{error, info};

#[derive(Default)]
pub struct ServiceImpl {}

#[tonic::async_trait]
impl MoniRpcService for ServiceImpl {
    async fn login_by_token(
        &self,
        request: tonic::Request<super::LoginTokenRequest>,
    ) -> Result<tonic::Response<super::LoginTokenResponse>, tonic::Status> {
        let token = request.into_inner().token;
        match dao::user_token::create_jwt_token(&token).await {
            Ok(token) => {
                let mut response = super::LoginTokenResponse::default();
                response.jwt_token = token;
                Ok(tonic::Response::new(response))
            }
            Err(e) => {
                error!("login_by_token error: {}, token: {}", e, &token);
                Err(tonic::Status::internal("internal error".to_string()))
            }
        }
    }
}

pub async fn start(addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    let rpc_impl = ServiceImpl::default();
    let svc = MoniRpcServiceServer::new(rpc_impl);
    info!("RpcServer listening on {addr}");
    tonic::transport::Server::builder()
        .add_service(svc)
        .serve(addr)
        .await?;

    Ok(())
}
