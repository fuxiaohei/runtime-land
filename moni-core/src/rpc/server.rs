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
                let response = super::LoginTokenResponse { jwt_token: token };
                Ok(tonic::Response::new(response))
            }
            Err(e) => {
                error!("login_by_token error: {}, token: {}", e, &token);
                Err(tonic::Status::internal("internal error".to_string()))
            }
        }
    }

    async fn create_function(
        &self,
        req: tonic::Request<super::CreateFunctionRequest>,
    ) -> Result<tonic::Response<super::CreateFunctionResponse>, tonic::Status> {
        self.verify_token(&req).await?;
        let req = req.into_inner();
        match dao::function_data::create(
            &req.name,
            1,
            &req.md5,
            &req.description,
            "rust",
            "to-location",
        )
        .await
        {
            Ok(_) => {
                let response = super::CreateFunctionResponse::default();
                Ok(tonic::Response::new(response))
            }
            Err(e) => {
                error!("create_function error: {}", e);
                Err(tonic::Status::internal("internal error".to_string()))
            }
        }
    }
}

impl ServiceImpl {
    pub async fn verify_token<T>(&self, req: &tonic::Request<T>) -> Result<(), tonic::Status> {
        let auth = match req.metadata().get("authorization") {
            Some(t) => t,
            _ => return Err(tonic::Status::unauthenticated("authorization required")),
        };
        // spit two part from auth string by ":"
        let values: Vec<_> = auth
            .to_str()
            .unwrap()
            .strip_prefix("Bearer ")
            .unwrap()
            .split(':')
            .collect();
        if values.len() != 2 {
            return Err(tonic::Status::unauthenticated("authorization format error"));
        }
        // verify token
        match dao::user_token::verify_jwt_token(values[0], values[1]).await {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("verify_token error: {}", e);
                Err(tonic::Status::unauthenticated("authorization invalid"))
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
