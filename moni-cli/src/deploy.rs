use moni_lib::meta::Meta;
use std::path::Path;
use tracing::{debug, info, warn};

pub async fn deploy(meta: &mut Meta, mut project_name: String, token: String, addr: String) {
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
    info!("Project name: {}", project_name);

    let mut client = moni_rpc::client::Client::new(addr, token).await.unwrap();

    // fetch project
    client
        .fetch_project(project_name.clone(), meta.language.clone())
        .await
        .unwrap_or_else(|e| {
            warn!("fetch project failed: {:?}", e);
        });
}
