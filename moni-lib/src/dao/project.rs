use crate::db::DB;
use crate::model::project_info::{self, ActiveModel, Model};
use anyhow::Result;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder};

pub async fn create(name: String, language: String, owner_id: i32) -> Result<Model> {
    let now = chrono::Utc::now();
    let uuid = uuid::Uuid::new_v4().to_string();
    let project = Model {
        id: 0,
        name,
        uuid,
        created_at: now,
        updated_at: now,
        owner_id: Some(owner_id),
        language,
        prod_deploy_id: Some(0),
    };
    let active_model: ActiveModel = project.into();
    let db = DB.get().unwrap();
    let project = active_model.insert(db).await?;
    Ok(project)
}

pub async fn find(owner_id: i32, name: String) -> Result<Option<Model>> {
    let db = DB.get().unwrap();
    let project = project_info::Entity::find()
        .filter(project_info::Column::Name.eq(name))
        .filter(project_info::Column::OwnerId.eq(owner_id))
        .one(db)
        .await?;
    Ok(project)
}

pub async fn list(owner_id: i32) -> Result<Vec<Model>> {
    let db = DB.get().unwrap();
    let projects = project_info::Entity::find()
        .filter(project_info::Column::OwnerId.eq(owner_id))
        .order_by_desc(project_info::Column::UpdatedAt)
        .all(db)
        .await?;
    Ok(projects)
}
