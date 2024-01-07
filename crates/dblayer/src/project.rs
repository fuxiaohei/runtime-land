use crate::models::project_info;
use crate::DB;
use anyhow::Result;
use land_common::MetaData;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};

#[derive(strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum CreatedByCases {
    LandCli,
}

#[derive(strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum Status {
    Active,
}

pub async fn find_by_name(owner_id: i32, name: &str) -> Result<Option<project_info::Model>> {
    let db = DB.get().unwrap();
    let project = project_info::Entity::find()
        .filter(project_info::Column::Name.eq(name))
        .filter(project_info::Column::OwnerId.eq(owner_id))
        .one(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;
    Ok(project)
}

pub async fn create(
    owner_id: i32,
    meta: &MetaData,
    create_by: CreatedByCases,
) -> Result<project_info::Model> {
    let project = find_by_name(owner_id, &meta.project.name).await?;
    if project.is_some() {
        return Err(anyhow::anyhow!("project is exist"));
    }
    let metadata_json = serde_json::to_string(meta)?;
    let project = project_info::Model {
        id: 0,
        owner_id,
        name: meta.project.name.clone(),
        language: meta.project.language.clone(),
        status: Status::Active.to_string(),
        uuid: uuid::Uuid::new_v4().to_string(),
        description: meta.project.description.clone(),
        created_by: create_by.to_string(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        deleted_at: None,
        metadata: Some(metadata_json),
    };
    let project_active_model: project_info::ActiveModel = project.into();
    let db = DB.get().unwrap();
    let project_model = project_active_model.insert(db).await?;
    Ok(project_model)
}
