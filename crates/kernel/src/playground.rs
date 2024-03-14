use anyhow::{anyhow, Result};
use land_dao::models::deployment::Model as DeploymentModel;
use land_dao::models::playground::Model as PlaygroundModel;
use land_dao::models::project::Model as ProjectModel;
use land_dao::project::Language;
use land_dao::{deployment, playground, project};
use tracing::info;

/// get_by_project gets a project, playground and deployment by project name
pub async fn get_by_project(
    name: String,
    user_id: i32,
) -> Result<(ProjectModel, PlaygroundModel, Option<DeploymentModel>)> {
    let p = project::get_by_name(name, Some(user_id)).await?;
    if p.is_none() {
        return Err(anyhow!("Project not found"));
    }
    let p = p.unwrap();
    let py = playground::get_by_project(user_id, p.id).await?;
    if py.is_none() {
        return Err(anyhow!("Playground not found"));
    }
    let dp = deployment::get_by_project(user_id, p.id).await?;
    Ok((p, py.unwrap(), dp))
}

/// save_source updates playground source
pub async fn save_source(name: String, user_id: i32, source: String) -> Result<()> {
    let p = project::get_by_name(name, Some(user_id)).await?;
    if p.is_none() {
        return Err(anyhow!("Project not found"));
    }
    let p = p.unwrap();
    let py = playground::update_source(user_id, p.id, source).await?;

    let dp = deployment::get_by_project(user_id, p.id).await?;
    if dp.is_none() {
        return Err(anyhow!("Deployment not created"));
    }
    let dp = dp.unwrap();

    // make deployment uploading status
    let task_id = uuid::Uuid::new_v4().to_string();
    deployment::mark_status(
        dp.id,
        deployment::DeployStatus::Uploading,
        "uploading".to_string(),
        Some(task_id),
        None,
    )
    .await?;

    // send deploy task to background
    super::tasks::send_deploy_task(super::tasks::DeployTask {
        deploy_id: dp.id,
        playground_id: py.id,
        project_id: p.id,
    })
    .await;
    info!("Playground source updated: {}", p.name);

    Ok(())
}

/// create creates a new playground
pub async fn create(
    user_id: i32,
    language: Language,
    description: String,
    source: String,
) -> Result<String> {
    let p = project::create_by_playground(user_id, language.clone(), description).await?;
    let py = playground::create(user_id, p.id, language, source, false).await?;
    let dp = deployment::create(user_id, p.id, p.domain).await?;
    // send deploy task to background
    super::tasks::send_deploy_task(super::tasks::DeployTask {
        deploy_id: dp.id,
        playground_id: py.id,
        project_id: p.id,
    })
    .await;
    info!("Playground created: {}", p.name);
    Ok(p.name)
}
