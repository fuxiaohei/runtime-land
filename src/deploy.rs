use anyhow::Result;
use land_core::metadata::Metadata;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::debug;

/// LOCAL_PROJECT_ENV_FILE is the file name of the local project env file
const LOCAL_PROJECT_ENV_FILE: &str = ".land.env";

/// CLIENT is the global http client
pub static CLIENT: OnceCell<reqwest::Client> = OnceCell::new();

/// load_project loads the project from local env or cloud
pub async fn load_project(
    project_name: Option<String>,
    meta: &Metadata,
    addr: &str,
    token: &str,
) -> Result<Project> {
    let local_project = read_project_env()?;
    if project_name.is_some() {
        // load project info from given name, local env
        let project_name = project_name.as_ref().unwrap();
        debug!("Try to load project from given name: {}", project_name);
        if local_project.is_some() && project_name == &local_project.as_ref().unwrap().name {
            return Ok(local_project.unwrap());
        }

        // if local env not found, load from cloud
        // use cloud project name to override local env
        debug!("Try to load project from cloud: {}", project_name);
        let project = query_project(addr, token, project_name.to_string()).await?;
        write_project_env(&project.project)?;
        return Ok(project.project);
    }

    // if project name not provided, load from local env
    debug!("Try to load project from local env");
    if local_project.is_some() {
        return Ok(local_project.unwrap());
    }
    debug!("Try to create new project from cloud");
    let project = create_project(meta, addr, token).await?;
    write_project_env(&project)?;
    Ok(project)
}

#[derive(Serialize, Default, Debug)]
struct CreateProjectRequest {
    language: String,
    prefix: Option<String>,
    name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Project {
    pub name: String,
    pub uuid: String,
    /*language: String,
    uuid: String,
    created_at: i64,
    updated_at: i64,
    prod_deployment: i32,
    prod_url: String,
    status: String,*/
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectOverView {
    pub project: Project,
}

/// query_project queries the project from cloud
pub async fn query_project(addr: &str, token: &str, name: String) -> Result<ProjectOverView> {
    let url = format!("{}/v2/project/{}/overview", addr, name);
    let client = CLIENT.get().unwrap();
    let resp = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;
    if !resp.status().is_success() {
        return Err(anyhow::anyhow!(
            "query project failed, status: {}",
            resp.status()
        ));
    }
    let project = resp.json::<ProjectOverView>().await?;
    Ok(project)
}

/// create_project creates a new project
pub async fn create_project(meta: &Metadata, addr: &str, token: &str) -> Result<Project> {
    let url = format!("{}/v1/project", addr);
    let data = CreateProjectRequest {
        language: meta.language.clone(),
        prefix: Some(meta.name.clone().replace('-', "")),
        name: None,
    };
    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&data)
        .send()
        .await?;
    let project = resp.json::<Project>().await?;
    Ok(project)
}

/// read_project_env reads the project name from env file
pub fn read_project_env() -> Result<Option<Project>> {
    let env_file = Path::new(LOCAL_PROJECT_ENV_FILE);
    if !env_file.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(env_file)?;
    let project = serde_json::from_str::<Project>(&content)?;
    Ok(Some(project))
}

/// write_project_env writes the project name to env file
pub fn write_project_env(project: &Project) -> Result<()> {
    let env_file = Path::new(LOCAL_PROJECT_ENV_FILE);
    let content = serde_json::to_vec(project)?;
    std::fs::write(env_file, content)?;
    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateDeployRequest {
    pub project_name: String,
    pub project_uuid: String,
    pub deploy_chunk: Vec<u8>,
    pub deploy_content_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeploymentResponse {
    pub domain: String,
    pub domain_url: String,
    pub prod_domain: String,
    pub prod_url: String,
    pub uuid: String,
}

/// create_deployment creates a new deployment
pub async fn create_deployment(
    project: Project,
    content: Vec<u8>,
    content_type: String,
    addr: &str,
    token: &str,
) -> Result<DeploymentResponse> {
    let url = format!("{}/v2/deployment", addr);
    let data = CreateDeployRequest {
        project_name: project.name,
        project_uuid: project.uuid,
        deploy_chunk: content,
        deploy_content_type: content_type,
    };
    let client = CLIENT.get().unwrap();
    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&data)
        .send()
        .await?;
    if !resp.status().is_success() {
        return Err(anyhow::anyhow!(
            "create deployment failed, status: {}, body: {}",
            resp.status(),
            resp.text().await?
        ));
    }
    let deployment = resp.json::<DeploymentResponse>().await?;
    Ok(deployment)
}

#[derive(Serialize, Deserialize, Debug)]
struct UpdateDeployRequest {
    pub project_uuid: String,
    pub deployment_uuid: String,
    pub action: String,
}

/// publish_deployment publishes the deployment
pub async fn publish_deployment(
    uuid: String,
    addr: &str,
    token: &str,
) -> Result<DeploymentResponse> {
    let url = format!("{}/v2/deployment", addr);
    let data = UpdateDeployRequest {
        project_uuid: String::new(),
        deployment_uuid: uuid,
        action: String::from("publish"),
    };
    let client = CLIENT.get().unwrap();
    let resp = client
        .put(&url)
        .json(&data)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;
    if !resp.status().is_success() {
        return Err(anyhow::anyhow!(
            "publish deployment failed, status: {}, body: {}",
            resp.status(),
            resp.text().await?
        ));
    }
    let deployment = resp.json::<DeploymentResponse>().await?;
    Ok(deployment)
}
