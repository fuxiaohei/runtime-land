use super::moni_rpc_service_server::MoniRpcService;
use crate::UserContext;
use gravatar::{Gravatar, Rating};
use moni_lib::dao::{self, project, token, user};
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
        let (user, token) = user::login_by_email(login_req.email, login_req.password)
            .await
            .map_err(|e| tonic::Status::internal(format!("{:?}", e)))?;
        let gravatar_url = Gravatar::new(&user.email)
            .set_size(Some(400))
            .set_rating(Some(Rating::Pg))
            .image_url();
        Ok(tonic::Response::new(super::LoginResponse {
            access_token: token.token,
            access_token_uuid: token.uuid,
            display_name: user.display_name,
            display_email: user.email,
            avatar_url: gravatar_url.to_string(),
        }))
    }

    async fn login_access_token(
        &self,
        req: tonic::Request<super::LoginAccessTokenRequest>,
    ) -> Result<tonic::Response<super::LoginResponse>, tonic::Status> {
        let token_req = req.into_inner();
        let (user, token) = user::login_by_access_token(token_req.access_token)
            .await
            .map_err(|e| tonic::Status::internal(format!("{:?}", e)))?;
        Ok(tonic::Response::new(super::LoginResponse {
            access_token: token.token,
            access_token_uuid: token.uuid,
            display_name: user.display_name,
            display_email: user.email,
            avatar_url: String::new(),
        }))
    }

    async fn create_project(
        &self,
        req: tonic::Request<super::CreateProjectRequest>,
    ) -> Result<tonic::Response<super::CreateProjectResponse>, tonic::Status> {
        let req = req.into_inner();
        let project = match project::create(req.name, req.language, 101).await {
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
        req: tonic::Request<super::Empty>,
    ) -> Result<tonic::Response<super::ListAccessTokensResponse>, tonic::Status> {
        let user_context: &UserContext = req.extensions().get().unwrap();
        let tk_value = user_context.get_token().await?;

        let tokens = token::list(tk_value.owner_id)
            .await
            .map_err(|e| tonic::Status::internal(format!("{:?}", e)))?;
        let resp = super::ListAccessTokensResponse {
            data: tokens
                .into_iter()
                .map(|t| super::AccessTokenData {
                    name: t.name,
                    created_at: t.created_at.timestamp(),
                    updated_at: t.updated_at.timestamp(),
                    expires_at: t.expired_at as i64,
                    origin: t.origin,
                    uuid: t.uuid,
                    value: None,
                })
                .collect(),
        };
        Ok(tonic::Response::new(resp))
    }

    async fn create_access_token(
        &self,
        req: tonic::Request<super::CreateAccessTokenRequest>,
    ) -> Result<tonic::Response<super::CreateAccessTokenResponse>, tonic::Status> {
        let user_context: &UserContext = req.extensions().get().unwrap();
        let tk_value = user_context.get_token().await?;

        let req = req.into_inner();
        let tk = token::create(
            tk_value.owner_id,
            req.name,
            "dashboard".to_string(),
            365 * 24 * 3600,
        )
        .await
        .map_err(|e| tonic::Status::internal(format!("{:?}", e)))?;
        let resp = super::CreateAccessTokenResponse {
            data: Some(super::AccessTokenData {
                name: tk.name,
                created_at: tk.created_at.timestamp(),
                updated_at: tk.updated_at.timestamp(),
                expires_at: tk.expired_at as i64,
                origin: tk.origin,
                uuid: tk.uuid,
                value: Some(tk.token),
            }),
        };
        Ok(tonic::Response::new(resp))
    }

    async fn remove_access_token(
        &self,
        req: tonic::Request<super::RemoveAccessTokenRequest>,
    ) -> std::result::Result<tonic::Response<super::NoDataResponse>, tonic::Status> {
        let user_context: &UserContext = req.extensions().get().unwrap();
        let tk_value = user_context.get_token().await?;

        let req = req.into_inner();
        token::remove(tk_value.owner_id, req.token_uuid)
            .await
            .map_err(|e| tonic::Status::internal(format!("{:?}", e)))?;
        Ok(tonic::Response::new(super::NoDataResponse {
            code: 0,
            error: String::new(),
        }))
    }

    async fn fetch_project(
        &self,
        req: tonic::Request<super::FetchProjectRequest>,
    ) -> std::result::Result<tonic::Response<super::ProjectResponse>, tonic::Status> {
        let user_context: &UserContext = req.extensions().get().unwrap();
        let token = user_context.get_token().await?;

        let project = dao::project::find(token.owner_id, req.into_inner().name)
            .await
            .map_err(|e| tonic::Status::internal(format!("{:?}", e)))?;
        if project.is_none() {
            return Err(tonic::Status::not_found("project not found"));
        }
        let project = project.unwrap();
        Ok(tonic::Response::new(super::ProjectResponse {
            name: project.name,
            language: project.language,
            uuid: project.uuid,
            prod_deployment: project.prod_deploy_id,
            update_at: project.updated_at.timestamp(),
        }))
    }
}
