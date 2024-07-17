use crate::{
    models::playground,
    now_time,
    projects::{self, Language},
    DB,
};
use anyhow::Result;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, QueryOrder};
pub type Status = projects::Status;

#[derive(strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum Visibility {
    Public,
    Private,
}

/// create creates a new playground
pub async fn create(
    owner_id: i32,
    project_id: i32,
    language: Language,
    source: String,
    visible: bool,
) -> Result<playground::Model> {
    let uuid = uuid::Uuid::new_v4().to_string();
    let p = playground::Model {
        id: 0,
        owner_id,
        project_id,
        uuid,
        language: language.to_string(),
        source,
        version: String::new(),
        status: Status::Active.to_string(),
        visiblity: if visible {
            Visibility::Public.to_string()
        } else {
            Visibility::Private.to_string()
        },
        created_at: now_time(),
        deleted_at: None,
    };
    let mut active_model = p.into_active_model();
    active_model.id = Default::default();
    let p = active_model.insert(DB.get().unwrap()).await?;
    Ok(p)
}

/// get_by_project gets a playground by project
pub async fn get_by_project(project_id: i32) -> Result<Option<playground::Model>> {
    let db = DB.get().unwrap();
    let p = playground::Entity::find()
        .filter(playground::Column::ProjectId.eq(project_id))
        .filter(playground::Column::Status.eq(Status::Active.to_string()))
        .order_by_desc(playground::Column::Id)
        .one(db)
        .await?;
    Ok(p)
}
