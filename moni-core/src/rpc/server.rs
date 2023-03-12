use super::moni_rpc_service_server::{MoniRpcService, MoniRpcServiceServer};
use std::net::SocketAddr;
use tracing::info;

#[derive(Default)]
pub struct ServiceImpl {}

#[tonic::async_trait]
impl MoniRpcService for ServiceImpl {
    async fn login_by_token(
        &self,
        _request: tonic::Request<super::LoginTokenRequest>,
    ) -> Result<tonic::Response<super::LoginTokenResponse>, tonic::Status> {
        todo!()
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
