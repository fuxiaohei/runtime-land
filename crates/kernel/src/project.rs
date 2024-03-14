use anyhow::{anyhow, Result};
use land_dao::models::playground::Model as PlaygroundModel;
use land_dao::models::project::Model as ProjectModel;
use land_dao::playground;
use land_dao::project::{self, CreatedBy};

/// show_single gets a project and playground by project name
pub async fn show_single(
    name: String,
    user_id: i32,
) -> Result<(ProjectModel, Option<PlaygroundModel>)> {
    let p = project::get_by_name(name, Some(user_id)).await?;
    if p.is_none() {
        return Err(anyhow!("Project not found"));
    }
    let p = p.unwrap();
    let mut py: Option<PlaygroundModel> = None;
    if p.created_by == CreatedBy::Playground.to_string() {
        py = playground::get_by_project(user_id, p.id).await?;
    }
    Ok((p, py))
}

/// delete deletes a project by id, with playground and deployment
pub async fn delete(user_id: i32, name: String) -> Result<()> {
    let p = project::get_by_name(name, Some(user_id)).await?;
    if p.is_none() {
        return Err(anyhow!("Project not found"));
    }
    let p = p.unwrap();
    project::delete(user_id, p.id).await?;
    playground::delete_by_project(user_id, p.id).await?;
    Ok(())
}
