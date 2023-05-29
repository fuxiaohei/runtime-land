use super::moni_rpc_service_server::MoniRpcService;
use crate::UserContext;
use gravatar::{Gravatar, Rating};
use moni_lib::dao::{self, token, user};
use moni_lib::storage::STORAGE;
use tracing::{debug, warn};

#[derive(Default)]
pub struct ServiceImpl {}

#[tonic::async_trait]
impl MoniRpcService for ServiceImpl {
    
    #[tracing::instrument(skip(self, req))]
    async fn signup_email(
        &self,
        req: tonic::Request<super::SignupEmailRequest>,
    ) -> std::result::Result<tonic::Response<super::LoginResponse>, tonic::Status> {
        let sign_req = req.into_inner();
        let (user, token) =
            user::signup_by_email(sign_req.email.clone(), sign_req.password, sign_req.nickname)
                .await
                .map_err(|e| {
                    warn!("failed, {:?}, {:?}", sign_req.email, e);
                    tonic::Status::internal(format!("{:?}", e))
                })?;
        let gravatar_url = Gravatar::new(&user.email)
            .set_size(Some(400))
            .set_rating(Some(Rating::Pg))
            .image_url();
        debug!("success, {:?}, {:?}", sign_req.email, token.uuid);
        Ok(tonic::Response::new(super::LoginResponse {
            access_token: token.token,
            access_token_uuid: token.uuid,
            display_name: user.display_name,
            display_email: user.email,
            avatar_url: gravatar_url.to_string(),
        }))
    }

    #[tracing::instrument(skip(self, req))]
    async fn login_email(
        &self,
        req: tonic::Request<super::LoginEmailRequest>,
    ) -> Result<tonic::Response<super::LoginResponse>, tonic::Status> {
        let login_req = req.into_inner();
        let (user, token) = user::login_by_email(login_req.email.clone(), login_req.password)
            .await
            .map_err(|e| {
                warn!("failed, {:?}, {:?}", login_req.email, e);
                tonic::Status::internal(format!("{:?}", e))
            })?;
        let gravatar_url = Gravatar::new(&user.email)
            .set_size(Some(400))
            .set_rating(Some(Rating::Pg))
            .image_url();
        debug!("success, {:?}, {:?}", login_req.email, token.uuid);
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
            updated_at: project.updated_at.timestamp(),
        }))
    }

    async fn create_empty_project(
        &self,
        req: tonic::Request<super::FetchProjectRequest>,
    ) -> std::result::Result<tonic::Response<super::ProjectResponse>, tonic::Status> {
        let user_context: &UserContext = req.extensions().get().unwrap();
        let token = user_context.get_token().await?;

        let req = req.into_inner();
        let project = dao::project::create(req.name, req.language, token.owner_id)
            .await
            .map_err(|e| tonic::Status::internal(format!("{:?}", e)))?;
        Ok(tonic::Response::new(super::ProjectResponse {
            name: project.name,
            language: project.language,
            uuid: project.uuid,
            prod_deployment: project.prod_deploy_id,
            updated_at: project.updated_at.timestamp(),
        }))
    }

    async fn list_projects(
        &self,
        req: tonic::Request<super::Empty>,
    ) -> std::result::Result<tonic::Response<super::ListProjectsResponse>, tonic::Status> {
        let user_context: &UserContext = req.extensions().get().unwrap();
        let token = user_context.get_token().await?;

        let projects = dao::project::list(token.owner_id)
            .await
            .map_err(|e| tonic::Status::internal(format!("{:?}", e)))?;
        let resp = super::ListProjectsResponse {
            data: projects
                .into_iter()
                .map(|p| super::ProjectResponse {
                    name: p.name,
                    language: p.language,
                    uuid: p.uuid,
                    prod_deployment: p.prod_deploy_id,
                    updated_at: p.updated_at.timestamp(),
                })
                .collect(),
        };
        Ok(tonic::Response::new(resp))
    }

    async fn create_deployment(
        &self,
        req: tonic::Request<super::CreateDeploymentRequest>,
    ) -> std::result::Result<tonic::Response<super::DeploymentResponse>, tonic::Status> {
        let user_context: &UserContext = req.extensions().get().unwrap();
        let token = user_context.get_token().await?;

        let req = req.into_inner();

        // get project
        let project = dao::project::find(token.owner_id, req.project_name)
            .await
            .map_err(|e| tonic::Status::internal(format!("{:?}", e)))?;
        if project.is_none() {
            return Err(tonic::Status::not_found("project not found"));
        }
        let project = project.unwrap();
        if project.uuid != req.project_uuid {
            return Err(tonic::Status::not_found("project not matched"));
        }

        // create deployment
        let deployment = dao::deployment::create(
            token.owner_id,
            project.id as i32,
            req.deploy_name.clone(),
            format!("fs://{}", req.deploy_name.clone()),
        )
        .await
        .map_err(|e| tonic::Status::internal(format!("create deployment failed: {:?}", e)))?;

        // save file
        let storage_path = format!("{}/{}.wasm", project.uuid, deployment.uuid);
        let storage = STORAGE.get().unwrap();
        storage
            .write(&storage_path, req.deploy_chunk)
            .await
            .map_err(|e| tonic::Status::internal(format!("save storage failed: {:?}", e)))?;
        dao::deployment::update_storage(deployment.id as i32, storage_path.clone())
            .await
            .map_err(|e| tonic::Status::internal(format!("update storage url failed: {:?}", e)))?;
        debug!(
            "save deployment {} to {}",
            req.deploy_name.clone(),
            storage_path
        );

        let resp = super::DeploymentResponse {
            id: deployment.id as i32,
            domain: req.deploy_name,
            uuid: deployment.uuid,
            deploy_status: deployment.deploy_status,
            prod_status: deployment.prod_status,
        };
        Ok(tonic::Response::new(resp))
    }

    async fn promote_deployment(
        &self,
        req: tonic::Request<super::PromoteDeploymentRequest>,
    ) -> std::result::Result<tonic::Response<super::DeploymentResponse>, tonic::Status> {
        let user_context: &UserContext = req.extensions().get().unwrap();
        let token = user_context.get_token().await?;

        let req = req.into_inner();
        let deployment =
            dao::deployment::promote(token.owner_id, req.deploy_id as i32, req.deploy_uuid)
                .await
                .map_err(|e| {
                    tonic::Status::internal(format!("promote deployment failed: {:?}", e))
                })?;
        let resp = super::DeploymentResponse {
            id: deployment.id as i32,
            domain: deployment.name,
            uuid: deployment.uuid,
            deploy_status: deployment.deploy_status,
            prod_status: deployment.prod_status,
        };
        Ok(tonic::Response::new(resp))
    }
}
