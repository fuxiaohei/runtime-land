use crate::moni_rpc_service_server::MoniRpcServiceServer;
use std::net::SocketAddr;
use tracing::info;

mod server;

tonic::include_proto!("moni");

pub async fn start_server(addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    let rpc_impl = server::ServiceImpl::default();
    let svc = MoniRpcServiceServer::new(rpc_impl);
    info!("RpcServer listening on {addr}");
    tonic::transport::Server::builder()
        .add_service(svc)
        .serve(addr)
        .await?;

    Ok(())
}
