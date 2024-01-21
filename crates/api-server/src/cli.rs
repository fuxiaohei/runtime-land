use crate::AppError;
use anyhow::anyhow;
use axum::{extract::Path, Json};
use land_dblayer::models::user_info;
use land_dblayer::{deployment, project, user};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, info_span, warn, Instrument};

/// LoginResponse is the response for /cli/login
#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub user_token: String,
    pub user_name: String,
    pub user_uuid: String,
    pub user_email: String,
}

pub async fn login(Path(token): Path<String>) -> Result<Json<LoginResponse>, AppError> {
    let user_token = user::find_token_by_value(&token).await?;
    if user_token.is_none() {
        warn!("token is not exist, value: {}", token);
        return Err(anyhow!("token is not exist").into());
    }
    let user_token = user_token.unwrap();
    // get current user info
    let user = user::find_by_id(user_token.owner_id).await?;
    if user.is_none() {
        warn!("user is not exist, id: {}", user_token.owner_id);
        return Err(anyhow!("user is not exist").into());
    }
    let user = user.unwrap();
    // create a new token for cli-accesss for this user
    let now_ts = chrono::Utc::now().timestamp();
    let new_token = user::create_token(
        user_token.owner_id,
        &format!(
            "cli-access-{}-{}-{}",
            user_token.owner_id, user_token.id, now_ts
        ),
        3600 * 24 * 365,
        user::TokenCreatedByCases::CliAccess,
    )
    .await?;
    let resp = LoginResponse {
        user_token: new_token.value,
        user_name: user.name,
        user_uuid: user.uuid,
        user_email: user.email,
    };
    debug!("login success, resp: {:?}", resp);
    Ok(Json(resp))
}

/// DeployRequest is the request for /cli/deploy
#[derive(Debug, Serialize, Deserialize)]
pub struct DeployRequest {
    pub metadata: land_common::MetaData,
    pub bundle: Vec<u8>,
    pub bundle_md5: String,
    pub user_token: String,
    pub user_uuid: String,
}

async fn check_cli_token(user_token: &str, user_uuid: &str) -> anyhow::Result<user_info::Model> {
    // validate user_token
    let token = user::find_token_by_value(user_token).await?;
    if token.is_none() {
        warn!("token is not exist, value: {}", user_token);
        return Err(anyhow!("token is not exist"));
    }
    let token = token.unwrap();
    if token.created_by != user::TokenCreatedByCases::CliAccess.to_string() {
        warn!("token is not cli-access, value: {}", user_token);
        return Err(anyhow!("token is not cli-access"));
    }
    let user = user::find_by_uuid(user_uuid).await?;
    if user.is_none() {
        warn!("user is not exist, uuid: {}", user_uuid);
        return Err(anyhow!("user is not exist"));
    }
    Ok(user.unwrap())
}

/// DeployResponse is the response for /cli/deploy
#[derive(Debug, Serialize, Deserialize)]
pub struct DeployResponse {
    pub visit_url: String,
    pub deploy_id: i32,
}

pub async fn deploy(Json(payload): Json<DeployRequest>) -> Result<Json<DeployResponse>, AppError> {
    // check md5
    let check_md5 = format!("{:x}", md5::compute(&payload.bundle));
    if check_md5 != payload.bundle_md5 {
        warn!(
            "bundle md5 not match, check_md5: {}, payload.bundle_md5: {}",
            check_md5, payload.bundle_md5
        );
        return Err(anyhow!("bundle md5 not match").into());
    }
    debug!("bundle size: {} KB", payload.bundle.len() / 1024);

    // validate user_token
    let user = check_cli_token(&payload.user_token, &payload.user_uuid).await?;

    // use meta data to create new project
    let mut project = project::find_by_name(user.id, &payload.metadata.project.name).await?;
    if project.is_none() {
        let p2 =
            project::create(user.id, &payload.metadata, project::CreatedByCases::LandCli).await?;
        project = Some(p2);
    }
    let project = project.unwrap();
    debug!("find project: {:?}", project);

    // get project testing deployment
    let deploy =
        deployment::find_by_project(project.id, deployment::DeploymentType::Testing).await?;
    let mut trace_uuid = String::new();
    let mut old_deploy_id = 0;
    if deploy.is_some() {
        let deploy = deploy.unwrap();
        trace_uuid = deploy.trace_uuid;
        old_deploy_id = deploy.id;
    }
    // create new deployment and set old deployment to replaced
    let new_deploy = deployment::create(
        project.id,
        user.id,
        &project.name,
        &trace_uuid,
        &payload.bundle_md5,
        payload.bundle.len() as i32,
        "application/gzip",
    )
    .await?;

    debug!(
        "create new_deploy: {:?}, old_deploy:{:?}",
        new_deploy, old_deploy_id
    );

    // build save storage path
    let save_storage_path = format!("{}/{}.tar.gz", project.uuid, new_deploy.name);
    debug!("save_storage_path: {:?}", save_storage_path);
    // use tokio task to upload storage
    tokio::spawn(
        async move {
            let deploy_id = new_deploy.id;
            let span = info_span!("upload-storage", path = &save_storage_path, deploy_id);
            let global_storage = land_dblayer::storage::GLOBAL.lock().await;
            let res = global_storage
                .write(&save_storage_path, payload.bundle)
                .instrument(span)
                .await;
            match res {
                Ok(_) => {
                    info!(storage_path = &save_storage_path, "success");
                    deployment::make_deploy_success(deploy_id, old_deploy_id, &save_storage_path)
                        .await
                        .unwrap();
                }
                Err(e) => {
                    warn!("failed, err: {}", e);
                    deployment::make_deploy_failed(deploy_id).await.unwrap();
                }
            }
        }
        .instrument(info_span!("upload-storage")),
    );

    let (domain_suffix, domain_protocol) = land_dblayer::settings::get_domain_settings().await?;
    let resp = DeployResponse {
        visit_url: format!(
            "{}://{}.{}",
            domain_protocol, new_deploy.name, domain_suffix
        ),
        deploy_id: new_deploy.id,
    };

    info!("deploy success, resp: {:?}", resp);
    Ok(Json(resp))
}

/// DeployRequest is the request for /cli/deploy
#[derive(Debug, Serialize, Deserialize)]
pub struct DeployCheckRequest {
    pub deploy_id: i32,
    pub user_token: String,
    pub user_uuid: String,
}

/// DeployCheckResponse is the response for /cli/deploy
#[derive(Debug, Serialize, Deserialize)]
pub struct DeployCheckResponse {
    pub visit_url: String,
    pub status: String,
    pub deploy_uuid: String,
}

pub async fn deploy_check(
    Json(payload): Json<DeployCheckRequest>,
) -> Result<Json<DeployCheckResponse>, AppError> {
    // validate user_token
    let user = check_cli_token(&payload.user_token, &payload.user_uuid).await?;
    let deploy = deployment::find_by_id(payload.deploy_id).await?;
    if deploy.is_none() {
        warn!("deploy is not exist, uuid: {}", payload.deploy_id);
        return Err(anyhow!("deploy is not exist").into());
    }
    let deploy = deploy.unwrap();
    if deploy.owner_id != user.id {
        warn!(
            "deploy owner_id not match, deploy.owner_id: {}, user.id: {}",
            deploy.owner_id, user.id
        );
        return Err(anyhow!("deploy owner_id not match").into());
    }

    let (domain_suffix, domain_protocol) = land_dblayer::settings::get_domain_settings().await?;

    let resp = DeployCheckResponse {
        visit_url: format!("{}://{}.{}", domain_protocol, deploy.name, domain_suffix),
        deploy_uuid: deploy.trace_uuid,
        status: deploy.deploy_status,
    };
    info!("deploy check, resp: {:?}", resp);
    Ok(Json(resp))
}
