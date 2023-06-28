use crate::db::DB;
use crate::model::project_info::{self, ActiveModel, Model};
use anyhow::Result;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};

#[derive(strum::Display)]
#[strum(serialize_all = "lowercase")]
enum ProjectStatus {
    Normal,
    Deleted,
}

pub async fn create(name: String, language: String, owner_id: i32) -> Result<Model> {
    let now = chrono::Utc::now();
    let uuid = uuid::Uuid::new_v4().to_string();
    let project = Model {
        id: 0,
        name,
        uuid,
        created_at: now,
        updated_at: now,
        owner_id,
        language,
        prod_deploy_id: 0,
        project_status: ProjectStatus::Normal.to_string(),
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

pub async fn find_by_id(owner_id: i32, project_id: i32) -> Result<Option<Model>> {
    let db = DB.get().unwrap();
    let project = project_info::Entity::find()
        .filter(project_info::Column::Id.eq(project_id))
        .filter(project_info::Column::OwnerId.eq(owner_id))
        .one(db)
        .await?;
    Ok(project)
}

pub async fn list_normal(owner_id: i32) -> Result<Vec<Model>> {
    let db = DB.get().unwrap();
    let projects = project_info::Entity::find()
        .filter(project_info::Column::OwnerId.eq(owner_id))
        .filter(project_info::Column::ProjectStatus.eq(ProjectStatus::Normal.to_string()))
        .order_by_desc(project_info::Column::UpdatedAt)
        .all(db)
        .await?;
    Ok(projects)
}

pub async fn remove(owner_id: i32, project_id: i32) -> Result<()> {
    let db = DB.get().unwrap();
    let project = project_info::Entity::find_by_id(project_id)
        .filter(project_info::Column::OwnerId.eq(owner_id))
        .one(db)
        .await?;
    if project.is_none() {
        return Err(anyhow::anyhow!("project not found"));
    }
    let mut project_model: project_info::ActiveModel = project.unwrap().into();
    project_model.project_status = Set(ProjectStatus::Deleted.to_string());
    project_model.updated_at = Set(chrono::Utc::now());
    project_model.update(db).await?;
    Ok(())
}
