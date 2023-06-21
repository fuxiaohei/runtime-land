use land_core::meta::Meta;
use land_restful::client::Client;
use std::path::Path;
use tracing::{debug, warn};

pub async fn deploy(
    meta: &mut Meta,
    mut project_name: String,
    token: String,
    addr: String,
    is_production: bool,
) {
    debug!("deploy: {:?}", meta);

    let output = meta.get_output();
    debug!("output: {:?}", output);

    // if output file is not exist, suggest to run build command
    if !Path::new(&output).exists() {
        warn!("output file not found, \nplease run `land-cli build`");
        return;
    }
    if project_name.is_empty() {
        project_name = meta.get_project_name();
    }
    if project_name.is_empty() {
        project_name = meta.generate_project_name();
    }
    println!("Fetching Project '{project_name}'");

    // fetch project info
    let client = Client::new(addr, token);
    let mut project = client
        .fetch_project(project_name.clone(), meta.language.clone())
        .await
        .expect("Fetch project '{project_name}' failed");
    // if project is not exist, create a new one
    if project.is_none() {
        println!("Project '{project_name}' not found, creating project");
        let project2 = client
            .create_project(project_name.clone(), meta.language.clone())
            .await
            .expect("Create project '{project_name}' failed");
        project = Some(project2);
    }
    let project = project.unwrap();
    debug!("project: {:?}", project);

    // prepare wasm binary
    let wasm_binary = std::fs::read(output).unwrap();
    println!(
        "Uploading assets to project '{project_name}', size: {size} KB",
        project_name = project.name,
        size = wasm_binary.len() / 1024,
    );
    let mut deployment = client
        .create_deploy(
            project.name.clone(),
            project.uuid.clone(),
            wasm_binary,
            "application/wasm".to_string(),
        )
        .await
        .expect("Create deploy failed");
    debug!("deployment: {:?}", deployment);
    println!("Deployed to project '{}' success\n", project.name,);

    if is_production {
        deployment = client
            .publish_deploy(deployment.id, deployment.uuid)
            .await
            .expect("Publish deploy failed");
        debug!("production deployment: {:?}", deployment);

        println!("Deploy to Production");
        println!("View at:");
        println!("- {}", deployment.prod_url);
        return;
    }

    println!("View at:");
    println!("- {}", deployment.domain_url);
}
