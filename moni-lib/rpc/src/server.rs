use super::moni_rpc_service_server::MoniRpcService;

#[derive(Default)]
pub struct ServiceImpl {}

#[tonic::async_trait]
impl MoniRpcService for ServiceImpl {
    async fn create_function(
        &self,
        _req: tonic::Request<super::CreateFunctionRequest>,
    ) -> Result<tonic::Response<super::CreateFunctionResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("Not implemented"))
    }
}
