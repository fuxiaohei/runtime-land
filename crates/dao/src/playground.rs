use anyhow::{anyhow, Result};
use sea_orm::{
    sea_query::Expr, ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter,
};
use std::str::FromStr;

use crate::{models::playground, now_time, project::Language, DB};

#[derive(strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum Status {
    Active,
    Deleted,
}

#[derive(strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum Visibility {
    Public,
    Private,
}

/// update_source updates a playground source
pub async fn update_source(
    user_id: i32,
    project_id: i32,
    source: String,
) -> Result<playground::Model> {
    let current = get_by_project(user_id, project_id).await?;
    if current.is_none() {
        return Err(anyhow!("Playground not found"));
    }
    let current = current.unwrap();
    let new_playground = create(
        user_id,
        project_id,
        Language::from_str(&current.language)?,
        source,
        current.visiblity == Visibility::Public.to_string(),
    )
    .await?;

    // delete old one
    let current_active_model = current.into_active_model();
    current_active_model
        .delete(DB.get().unwrap())
        .await
        .map_err(|e| anyhow!(e))?;
    Ok(new_playground)
}

/// create creates a new playground
pub async fn create(
    user_id: i32,
    project_id: i32,
    language: Language,
    source: String,
    visible: bool,
) -> Result<playground::Model> {
    let uuid = uuid::Uuid::new_v4().to_string();
    let p = playground::Model {
        id: 0,
        user_id,
        project_id,
        uuid,
        language: language.to_string(),
        source,
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
pub async fn get_by_project(user_id: i32, project_id: i32) -> Result<Option<playground::Model>> {
    let db = DB.get().unwrap();
    let p = playground::Entity::find()
        .filter(playground::Column::UserId.eq(user_id))
        .filter(playground::Column::ProjectId.eq(project_id))
        .filter(playground::Column::Status.eq(Status::Active.to_string()))
        .one(db)
        .await?;
    Ok(p)
}

/// get_by_id gets a playground by id
pub async fn get_by_id(id: i32) -> Result<Option<playground::Model>> {
    let db = DB.get().unwrap();
    let p = playground::Entity::find()
        .filter(playground::Column::Id.eq(id))
        .filter(playground::Column::Status.eq(Status::Active.to_string()))
        .one(db)
        .await?;
    Ok(p)
}

/// delete_by_project sets playgroud of project to deleted status
pub async fn delete_by_project(user_id: i32, project_id: i32) -> Result<()> {
    let db = DB.get().unwrap();
    playground::Entity::update_many()
        .filter(playground::Column::UserId.eq(user_id))
        .filter(playground::Column::ProjectId.eq(project_id))
        .col_expr(
            playground::Column::Status,
            Expr::value(Status::Deleted.to_string()),
        )
        .col_expr(playground::Column::DeletedAt, Expr::value(now_time()))
        .exec(db)
        .await?;
    Ok(())
}
