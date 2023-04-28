use super::moni_rpc_service_server::MoniRpcService;
use gravatar::{Gravatar, Rating};
use moni_lib::dao::{project, token, user};
use tracing::warn;

#[derive(Default)]
pub struct ServiceImpl {}

#[tonic::async_trait]
impl MoniRpcService for ServiceImpl {
    async fn login_email(
        &self,
        req: tonic::Request<super::LoginEmailRequest>,
    ) -> Result<tonic::Response<super::LoginResponse>, tonic::Status> {
        let login_req = req.into_inner();
        let (user, token) = match user::login_by_email(login_req.email, login_req.password).await {
            Ok(t) => t,
            Err(e) => {
                let resp = super::LoginResponse {
                    error: format!("{:?}", e),
                    code: 1,
                    data: None,
                };
                warn!("login by email failed: {:?}", e);
                return Ok(tonic::Response::new(resp));
            }
        };
        let gravatar_url = Gravatar::new(&user.email)
            .set_size(Some(400))
            .set_rating(Some(Rating::Pg))
            .image_url();
        let resp = super::LoginResponse {
            error: String::new(),
            code: 0,
            data: Some(crate::LoginResultData {
                access_token: token,
                display_name: user.display_name,
                display_email: user.email,
                avatar_url: gravatar_url.to_string(),
            }),
        };
        Ok(tonic::Response::new(resp))
    }

    async fn login_access_token(
        &self,
        _req: tonic::Request<super::LoginAccessTokenRequest>,
    ) -> Result<tonic::Response<super::LoginResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn create_project(
        &self,
        req: tonic::Request<super::CreateProjectRequest>,
    ) -> Result<tonic::Response<super::CreateProjectResponse>, tonic::Status> {
        let req = req.into_inner();
        let project = match project::create(req.name, req.description, req.language, 101).await {
            Ok(p) => p,
            Err(e) => {
                let resp = super::CreateProjectResponse {
                    error: format!("{:?}", e),
                    code: 1,
                    data: None,
                };
                warn!("create project failed: {:?}", e);
                return Ok(tonic::Response::new(resp));
            }
        };
        let resp = super::CreateProjectResponse {
            error: String::new(),
            code: 0,
            data: Some(crate::ProjectData {
                uuid: String::from("123"),
                name: project.name,
                version: String::from("0.0.1"),
            }),
        };
        Ok(tonic::Response::new(resp))
    }

    async fn list_access_tokens(
        &self,
        _req: tonic::Request<super::Empty>,
    ) -> Result<tonic::Response<super::ListAccessTokensResponse>, tonic::Status> {
        let tokens = match token::list(1).await {
            Ok(t) => t,
            Err(e) => {
                let resp = super::ListAccessTokensResponse {
                    error: format!("{:?}", e),
                    code: 1,
                    data: vec![],
                };
                warn!("list access tokens failed: {:?}", e);
                return Ok(tonic::Response::new(resp));
            }
        };
        let resp = super::ListAccessTokensResponse {
            error: String::new(),
            code: 0,
            data: tokens
                .into_iter()
                .map(|t| super::AccessTokenData {
                    name: t.name,
                    created_at: t.created_at.timestamp(),
                    expires_at: t.expired_at as i64,
                    origin: t.origin,
                })
                .collect(),
        };
        Ok(tonic::Response::new(resp))
    }
}
