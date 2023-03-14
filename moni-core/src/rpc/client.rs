use super::moni_rpc_service_client::MoniRpcServiceClient;
use tonic::{codegen::InterceptedService, transport::Channel};
use tonic::{metadata::MetadataValue, service::Interceptor, Request, Status};

/// new_client create a new rpc client with token
pub async fn new_client_with_token(
    addr: String,
    token: String,
    jwt_token: String,
) -> Result<
    MoniRpcServiceClient<InterceptedService<Channel, ClientTokenInterceptor>>,
    Box<dyn std::error::Error>,
> {
    let channel = Channel::from_shared(addr)?.connect().await?;
    let client = MoniRpcServiceClient::with_interceptor(
        channel,
        ClientTokenInterceptor { token, jwt_token },
    );
    Ok(client)
}

/// new client without token
pub async fn new_client(
    addr: String,
) -> Result<MoniRpcServiceClient<Channel>, Box<dyn std::error::Error>> {
    let channel = Channel::from_shared(addr)?.connect().await?;
    let client = MoniRpcServiceClient::new(channel);
    Ok(client)
}

/// ClientTokenInterceptor is a interceptor to add jwt token to request
pub struct ClientTokenInterceptor {
    token: String,
    jwt_token: String,
}

impl Interceptor for ClientTokenInterceptor {
    fn call(&mut self, mut req: Request<()>) -> Result<Request<()>, Status> {
        let token_value = format!("Bearer {}:{}", self.token, self.jwt_token);
        let token: MetadataValue<_> = token_value.parse().unwrap();
        req.metadata_mut().insert("Authorization", token);
        Ok(req)
    }
}
