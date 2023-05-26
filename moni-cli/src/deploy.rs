use moni_lib::meta::Meta;
use moni_rpc::client::Client;
use moni_rpc::{DeploymentResponse, ProjectResponse};
use std::path::Path;
use tracing::{debug, info, warn};

pub async fn deploy(
    meta: &mut Meta,
    mut project_name: String,
    token: String,
    addr: String,
    is_production: bool,
) {
    println!("deploy: {:?}", meta);
    let output = meta.get_output();
    debug!("output: {:?}", output);

    // if output file is not exist, suggest to run build command
    if !Path::new(&output).exists() {
        warn!("output file not found, \nplease run `moni-cli build`");
        return;
    }
    if project_name.is_empty() {
        project_name = meta.get_project_name();
    }
    if project_name.is_empty() {
        project_name = meta.generate_project_name();
    }
    info!("Fetching Project '{project_name}'");

    // fetch project info
    let mut client = moni_rpc::client::Client::new(addr, token).await.unwrap();
    let project = fetch_project(&mut client, project_name, meta.language.clone())
        .await
        .unwrap();

    // upload wasm file to project
    let wasm_binary = std::fs::read(output).unwrap();
    info!(
        "Uploading assets to project '{project_name}', size: {size} KB",
        project_name = project.name,
        size = wasm_binary.len() / 1024,
    );
    let deployment = create_deploy(&mut client, &project, wasm_binary, is_production)
        .await
        .unwrap();

    info!(
        "Deployed to project '{project_name}', deploy domain: {deploy_name}",
        project_name = project.name,
        deploy_name = deployment.domain,
    );
}

async fn fetch_project(
    client: &mut Client,
    project_name: String,
    language: String,
) -> Option<ProjectResponse> {
    // fetch project
    let mut project = client
        .fetch_project(project_name.clone(), language.clone())
        .await
        .unwrap_or_else(|e| {
            warn!("fetch project failed: {:?}", e);
            return None;
        });
    // if project is not exist, create empty project with name
    if project.is_none() {
        info!("Project not found, create '{project_name}' project");
        project = client
            .create_project(project_name.clone(), language.clone())
            .await
            .unwrap_or_else(|e| {
                warn!("create project failed: {:?}", e);
                return None;
            });
        info!(
            "Project '{project_name}' created",
            project_name = project_name
        );
    }
    project
}

async fn create_deploy(
    client: &mut Client,
    project: &ProjectResponse,
    binary: Vec<u8>,
    is_production: bool,
) -> Option<DeploymentResponse> {
    let response = client
        .create_deployment(
            project.name.clone(),
            project.uuid.clone(),
            binary,
            "application/wasm".to_string(),
            is_production,
        )
        .await
        .unwrap_or_else(|e| {
            warn!("create deployment failed: {:?}", e);
            return None;
        });
    response
}
