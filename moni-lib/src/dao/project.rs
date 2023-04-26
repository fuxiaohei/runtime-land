use crate::db::DB;
use crate::model::project_info::{ActiveModel, Model};
use anyhow::Result;
use sea_orm::ActiveModelTrait;

pub async fn create(name: String, desc: String, language: String, owner_id: i32) -> Result<Model> {
    let now = chrono::Utc::now();
    let project = Model {
        id: 0,
        name,
        created_at: now,
        updated_at: now,
        owner_id: Some(owner_id),
        language,
        description: desc,
        prod_deploy_id: Some(0),
    };
    let active_model: ActiveModel = project.into();
    let db = DB.get().unwrap();
    let project = active_model.insert(db).await?;
    Ok(project)
}
