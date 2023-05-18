use super::moni_rpc_service_client::MoniRpcServiceClient;
use tonic::codegen::InterceptedService;
use tonic::metadata::MetadataValue;
use tonic::service::Interceptor;
use tonic::transport::Channel;
use tonic::{Request, Status};

/// ClientTokenInterceptor is a interceptor to add jwt token to request
pub struct ClientTokenInterceptor {
    token: String,
}
impl Interceptor for ClientTokenInterceptor {
    fn call(&mut self, mut req: Request<()>) -> Result<Request<()>, Status> {
        let token_value = format!("Bearer {}", self.token);
        let token: MetadataValue<_> = token_value.parse().unwrap();
        req.metadata_mut().insert("authorization", token);

        let grpc_method: MetadataValue<_> = "moni-cli".parse().unwrap();
        req.metadata_mut().insert("x-grpc-method", grpc_method);
        Ok(req)
    }
}

pub struct Client {
    client: MoniRpcServiceClient<InterceptedService<Channel, ClientTokenInterceptor>>,
}

impl Client {
    pub async fn new(addr: String, token: String) -> Result<Self, Box<dyn std::error::Error>> {
        let channel = Channel::from_shared(addr)?.connect().await?;
        let client =
            MoniRpcServiceClient::with_interceptor(channel, ClientTokenInterceptor { token });
        Ok(Client { client })
    }

    pub async fn fetch_project(
        &mut self,
        name: String,
        language: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let req = tonic::Request::new(super::FetchProjectRequest { name, language });
        let resp = self.client.fetch_project(req).await?;
        println!("RESPONSE={:?}", resp);
        Ok(())
    }
}
